use std::{fmt, marker::PhantomData};

pub trait Index {
    fn new(index: usize) -> Self;
    fn index(self) -> usize;
}

pub struct IndexMap<K, V> {
    inner: Vec<V>,
    marker: PhantomData<K>,
}

impl<K: Index, V> IndexMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
            marker: PhantomData,
        }
    }

    pub fn repeat(f: impl Fn() -> V, len: usize) -> Self {
        let mut inner = Vec::with_capacity(len);
        for _ in 0..len {
            inner.push(f());
        }

        Self {
            inner,
            marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn push(&mut self, value: V) -> K {
        let index = K::new(self.inner.len());
        self.inner.push(value);
        index
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.inner.get(key.index())
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(key.index())
    }

    pub fn keys(&self) -> impl DoubleEndedIterator<Item = K> {
        (0..self.inner.len()).map(K::new)
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.inner
            .iter()
            .enumerate()
            .map(|(index, value)| (K::new(index), value))
    }
}

impl<K: Index, V> std::ops::Index<K> for IndexMap<K, V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        std::ops::Index::index(&self.inner, index.index())
    }
}

impl<K: Index, V> std::ops::IndexMut<K> for IndexMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        std::ops::IndexMut::index_mut(&mut self.inner, index.index())
    }
}

impl<K: Index + fmt::Debug, V: fmt::Debug> fmt::Debug for IndexMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

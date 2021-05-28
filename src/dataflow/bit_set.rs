#[derive(PartialEq, Eq)]
pub struct BitSet<T> {
    bits: Box<[u8]>,
    marker: std::marker::PhantomData<T>,
}

impl<T> BitSet<T> {
    pub fn new(len: usize) -> Self {
        let (mut bit_len, rem) = pos_and_offset(len);

        if rem != 0 {
            bit_len += 1;
        }

        Self {
            bits: vec![0u8; bit_len].into_boxed_slice(),
            marker: Default::default(),
        }
    }

    pub fn insert(&mut self, index: usize) {
        let (pos, offset) = pos_and_offset(index);
        self.bits[pos] |= 1 << offset;
    }

    pub fn remove(&mut self, index: usize) {
        let (pos, offset) = pos_and_offset(index);
        self.bits[pos] &= !(1 << offset);
    }

    pub fn union(&mut self, other: &Self) {
        for (self_bit, &other_bit) in self.bits.iter_mut().zip(other.bits.iter()) {
            *self_bit |= other_bit;
        }
    }

    pub fn intersection(&mut self, other: &Self) {
        for (self_bit, &other_bit) in self.bits.iter_mut().zip(other.bits.iter()) {
            *self_bit &= other_bit;
        }
    }

    pub fn difference(&mut self, other: &Self) {
        for (self_bit, &other_bit) in self.bits.iter_mut().zip(other.bits.iter()) {
            *self_bit &= !other_bit;
        }
    }

    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = bool> {
        let mut pos = 0;
        let mut offset = 0;

        std::iter::from_fn(move || {
            let item = (self.bits.get(pos)? >> offset) & 1 == 1;

            if offset == 7 {
                offset = 0;
                pos += 1;
            } else {
                offset += 1;
            }

            Some(item)
        })
    }
}

fn pos_and_offset(index: usize) -> (usize, usize) {
    (index / 8, index % 8)
}

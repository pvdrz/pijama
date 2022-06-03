use crate::mir::{Local, Rvalue};

pub enum Statement {
    Assign { lhs: Local, rhs: Rvalue },
}

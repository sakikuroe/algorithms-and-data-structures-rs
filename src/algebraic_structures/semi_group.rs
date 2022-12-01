use std::cmp;

pub trait SemiGroup {
    type S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub struct MinSemiGroup;

impl SemiGroup for MinSemiGroup {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::min(*a, *b)
    }
}

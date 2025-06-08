use std::cmp;

pub trait SemiGroup {
    type S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
}

pub struct MinSemiGroup;

impl SemiGroup for MinSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::min(*a, *b)
    }
}

pub struct MaxSemiGroup;

impl SemiGroup for MaxSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        cmp::max(*a, *b)
    }
}

pub struct AddSemiGroup;

impl SemiGroup for AddSemiGroup {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}

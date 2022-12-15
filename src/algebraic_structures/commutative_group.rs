pub trait CommutativeGroup {
    type S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
    fn id() -> Self::S;
    fn inv(a: &Self::S) -> Self::S;
}

pub struct AddGroup;

impl CommutativeGroup for AddGroup {
    type S = isize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
    fn id() -> Self::S {
        0
    }
    fn inv(a: &Self::S) -> Self::S {
        -a
    }
}
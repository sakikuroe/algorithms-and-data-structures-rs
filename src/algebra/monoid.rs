use super::semi_group::SemiGroup;

pub trait Monoid: SemiGroup {
    fn id() -> Self::S;
}

pub struct MinMonoid;

impl SemiGroup for MinMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::min(*a, *b)
    }
}

impl Monoid for MinMonoid {
    fn id() -> Self::S {
        std::i64::MAX
    }
}

pub struct MaxMonoid;

impl SemiGroup for MaxMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::max(*a, *b)
    }
}

impl Monoid for MaxMonoid {
    fn id() -> Self::S {
        std::i64::MIN
    }
}
pub struct AddMonoid;

impl SemiGroup for AddMonoid {
    type S = i64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}

impl Monoid for AddMonoid {
    fn id() -> Self::S {
        0
    }
}

pub struct XorMonoid;

impl SemiGroup for XorMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a ^ *b
    }
}

impl Monoid for XorMonoid {
    fn id() -> Self::S {
        0
    }
}

pub struct AndMonoid;

impl SemiGroup for AndMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a & *b
    }
}

impl Monoid for AndMonoid {
    fn id() -> Self::S {
        std::u64::MAX
    }
}

pub struct OrMonoid;

impl SemiGroup for OrMonoid {
    type S = u64;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a | *b
    }
}

impl Monoid for OrMonoid {
    fn id() -> Self::S {
        0
    }
}

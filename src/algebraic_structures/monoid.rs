use crate::algorithms::number_theory::gcd_lcm::{gcd, lcm};

pub trait Monoid {
    type S;
    fn op(a: &Self::S, b: &Self::S) -> Self::S;
    fn id() -> Self::S;
}

pub struct MinMonoid;

impl Monoid for MinMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::min(*a, *b)
    }
    fn id() -> Self::S {
        std::usize::MAX
    }
}

pub struct MaxMonoid;

impl Monoid for MaxMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        std::cmp::max(*a, *b)
    }
    fn id() -> Self::S {
        0
    }
}

pub struct AddMonoid;

impl Monoid for AddMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
    fn id() -> Self::S {
        0
    }
}

pub struct MulMonoid;

impl Monoid for MulMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a * *b
    }
    fn id() -> Self::S {
        1
    }
}

pub struct XorMonoid;

impl Monoid for XorMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a ^ *b
    }
    fn id() -> Self::S {
        0
    }
}

pub struct GcdMonoid;

impl Monoid for GcdMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        gcd(*a, *b)
    }
    fn id() -> Self::S {
        0
    }
}

pub struct LcmMonoid;

impl Monoid for LcmMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        lcm(*a, *b)
    }
    fn id() -> Self::S {
        1
    }
}

pub struct AndMonoid;

impl Monoid for AndMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a & *b
    }
    fn id() -> Self::S {
        std::usize::MAX
    }
}

pub struct OrMonoid;

impl Monoid for OrMonoid {
    type S = usize;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        *a | *b
    }
    fn id() -> Self::S {
        0
    }
}

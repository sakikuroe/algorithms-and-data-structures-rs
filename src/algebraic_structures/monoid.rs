use crate::{algorithms::number_theory::gcd_lcm::{gcd, lcm}, data_structures::modint::ModInt};

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

#[derive(Clone)]
pub struct LinearFunction {
    a: ModInt,
    b: ModInt,
}

pub struct LinearFunctionMonoid;

impl Monoid for LinearFunctionMonoid {
    type S = LinearFunction;
    fn id() -> Self::S {
        LinearFunction {
            a: ModInt::new(1),
            b: ModInt::new(0),
        }
    }
    fn op(f2: &Self::S, f1: &Self::S) -> Self::S {
        LinearFunction {
        a: f1.a * f2.a,
            b: f2.a * f1.b + f2.b,
        }
    }
}

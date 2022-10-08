use crate::algebraic_structures::complex_number::{distance, ComplexNumber};

#[derive(Debug)]
pub struct Circle {
    pub p: ComplexNumber<f64>,
    pub r: f64,
}

impl Circle {
    pub fn new(x: f64, y: f64, r: f64) -> Self {
        Circle {
            p: ComplexNumber::new(x, y),
            r,
        }
    }
}

pub fn is_intersect(ca: &Circle, cb: &Circle) -> bool {
    ca.r + cb.r >= distance(ca.p, cb.p)
}

pub fn is_contact(ca: &Circle, cb: &Circle) -> bool {
    ca.r + cb.r == distance(ca.p, cb.p)
}

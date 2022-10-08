use crate::{
    algebraic_structures::complex_number::ComplexNumber, algorithms::number_theory::gcd_lcm::gcd,
};

pub fn cross(a: ComplexNumber<isize>, b: ComplexNumber<isize>) -> isize {
    a.re() * b.im() - a.im() * b.re()
}

pub fn get_convex_hull(cs: &Vec<ComplexNumber<isize>>) -> Vec<ComplexNumber<isize>> {
    let mut cs = cs.clone();
    cs.sort_by_key(|c| (c.re, c.im));
    let mut upper = vec![cs[cs.len() - 1], cs[cs.len() - 2]];
    let mut bottom = vec![cs[0], cs[1]];

    for i in 2..cs.len() {
        while upper.len() >= 2
            && cross(
                upper[upper.len() - 1] - upper[upper.len() - 2],
                cs[cs.len() - 1 - i] - upper[upper.len() - 1],
            ) <= 0
        {
            upper.pop();
        }
        upper.push(cs[cs.len() - 1 - i]);
        while bottom.len() >= 2
            && cross(
                bottom[bottom.len() - 1] - bottom[bottom.len() - 2],
                cs[i] - bottom[bottom.len() - 1],
            ) <= 0
        {
            bottom.pop();
        }
        bottom.push(cs[i]);
    }

    bottom
        .into_iter()
        .skip(1)
        .chain(&mut upper.into_iter().skip(1))
        .collect::<Vec<_>>()
}

pub fn number_of_points_on_the_line_segment(
    a: ComplexNumber<isize>,
    b: ComplexNumber<isize>,
) -> usize {
    let dx = (a.re() - b.re()).abs() as usize;
    let dy = (a.im() - b.im()).abs() as usize;
    gcd(dx, dy) + 1
}

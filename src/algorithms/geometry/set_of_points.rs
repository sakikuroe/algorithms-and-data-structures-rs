//! verified by
//! - AtCoder | [AtCoder Beginner Contest 207 D - Congruence Points](https://atcoder.jp/contests/abc207/tasks/abc207_d), ([submittion](https://atcoder.jp/contests/abc207/submissions/34978473))

use crate::{
    algebraic_structures::complex_number::{cross_product, inner_product, ComplexNumber},
    algorithms::string::z_algorithm::string_search_cyclic,
};

pub fn matches_except_for_translation_of_axes_and_rotations(
    ab: &Vec<(isize, isize)>,
    cd: &Vec<(isize, isize)>,
) -> bool {
    if ab.len() != cd.len() {
        return false;
    }

    if ab.len() == 1 {
        return true;
    }

    let ab = ab
        .clone()
        .into_iter()
        .map(|(a, b)| ComplexNumber::new(a * (ab.len() as isize), b * (ab.len() as isize)))
        .collect::<Vec<_>>();
    let g = {
        let gn = ab
            .iter()
            .cloned()
            .fold(ComplexNumber::new(0, 0), |ca, cb| ca + cb);
        ComplexNumber::new(gn.re() / (ab.len() as isize), gn.im() / (ab.len() as isize))
    };
    let mut ab = ab
        .into_iter()
        .filter(|c| *c != g)
        .map(|c| c - g)
        .collect::<Vec<_>>();
    ab.sort_by(|ca, cb| ca.abs_cmp(&cb));
    ab.sort_by(|ca, cb| ca.argument_cmp(&cb).unwrap());

    let cd = cd
        .clone()
        .into_iter()
        .map(|(c, d)| ComplexNumber::new(c * (cd.len() as isize), d * (cd.len() as isize)))
        .collect::<Vec<_>>();
    let g = {
        let gn = cd
            .iter()
            .cloned()
            .fold(ComplexNumber::new(0, 0), |ca, cb| ca + cb);
        ComplexNumber::new(gn.re() / (cd.len() as isize), gn.im() / (cd.len() as isize))
    };
    let mut cd = cd
        .into_iter()
        .filter(|c| *c != g)
        .map(|c| c - g)
        .collect::<Vec<_>>();
    cd.sort_by(|ca, cb| ca.abs_cmp(&cb));
    cd.sort_by(|ca, cb| ca.argument_cmp(&cb).unwrap());

    if ab.len() != cd.len() {
        return false;
    }

    let mut u = vec![];
    let mut v = vec![];
    for i in 0..ab.len() {
        u.push(inner_product(ab[i], ab[i]));
        u.push(inner_product(ab[i], ab[(i + 1) % ab.len()]));
        u.push(cross_product(ab[i], ab[(i + 1) % ab.len()]));

        v.push(inner_product(cd[i], cd[i]));
        v.push(inner_product(cd[i], cd[(i + 1) % cd.len()]));
        v.push(cross_product(cd[i], cd[(i + 1) % cd.len()]));
    }

    string_search_cyclic(&u, &v)
        .into_iter()
        .filter(|i| i % 3 == 0)
        .count()
        > 0
}

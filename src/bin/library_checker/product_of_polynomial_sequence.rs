// Library Checker: Product of Polynomial Sequence
// https://judge.yosupo.jp/problem/product_of_polynomial_sequence

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::fps;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut polynomials = Vec::with_capacity(n);
    let mut degree_sum = 0_usize;
    for _ in 0..n {
        let d = io.u32() as usize;
        degree_sum += d;

        let mut coeffs = Vec::with_capacity(d + 1);
        for _ in 0..=d {
            coeffs.push(io.u32());
        }
        polynomials.push(fps::FPS::new(coeffs));
    }

    let product = fps::FPS::product(polynomials, degree_sum);

    for i in 0..=degree_sum {
        io.writeln(product.get(i));
    }

    io.flush();
}

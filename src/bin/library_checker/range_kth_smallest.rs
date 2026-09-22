// Library Checker: Range Kth Smallest
// https://judge.yosupo.jp/problem/range_kth_smallest

use anmitsu::ds::wavelet_matrix::WaveletMatrix;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();
    let n = io.u32() as usize;
    let q = io.u32() as usize;
    let mut a = vec![0; n];
    for x in a.iter_mut() {
        *x = io.u32() as usize;
    }
    let wavelet_matrix = WaveletMatrix::new(&a);

    for _ in 0..q {
        let l = io.u32() as usize;
        let r = io.u32() as usize;
        let k = io.u32() as usize;
        let answer = wavelet_matrix.get_kth_smallest(l..r, .., k).unwrap();
        io.writeln(answer as u32);
    }
    io.flush();
}

// AtCoder: ABC234 D - Prefix K-th Max
// https://atcoder.jp/contests/abc234/tasks/abc234_d

use anmitsu::ds::wavelet_matrix::WaveletMatrix;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();
    let n = io.u32() as usize;
    let k = io.u32() as usize;
    let values = (0..n).map(|_| io.u64() as usize).collect::<Vec<_>>();
    let wavelet_matrix = WaveletMatrix::new(&values);

    for right in k..=n {
        let answer = wavelet_matrix.get_kth_largest(..right, .., k - 1).unwrap();
        io.writeln(answer as u64);
    }
    io.flush();
}

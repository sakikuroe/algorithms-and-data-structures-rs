// AtCoder: Typical90 017 - Crossing Segments
// https://atcoder.jp/contests/typical90/tasks/typical90_q

use anmitsu::ds::wavelet_matrix::WaveletMatrix;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();
    let _n = io.u32();
    let m = io.u32() as usize;
    let mut segments = (0..m)
        .map(|_| (io.u32() as usize, io.u32() as usize))
        .collect::<Vec<_>>();
    segments.sort_unstable_by_key(|&(_, right)| right);

    let rights = segments.iter().map(|&(_, right)| right).collect::<Vec<_>>();
    let lefts = segments.iter().map(|&(left, _)| left).collect::<Vec<_>>();
    let wavelet_matrix = WaveletMatrix::new(&lefts);
    let answer = segments
        .iter()
        .map(|&(left, right)| {
            let index_start = rights.partition_point(|&value| value <= left);
            let index_end = rights.partition_point(|&value| value < right);
            wavelet_matrix.count(index_start..index_end, ..left)
        })
        .sum::<usize>();

    io.writeln(answer as u64);
    io.flush();
}

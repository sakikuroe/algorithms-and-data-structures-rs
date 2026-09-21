// AtCoder: ABC405 F - Chord Crossing
// https://atcoder.jp/contests/abc405/tasks/abc405_f

use anmitsu::ds::wavelet_matrix::WaveletMatrix;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();
    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let mut mates = vec![0; n + 1];
    for _ in 0..m {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        mates[a / 2] = b;
        mates[b / 2] = a;
    }
    let wavelet_matrix = WaveletMatrix::new(&mates);

    let q = io.u32() as usize;
    for _ in 0..q {
        let c = io.u32() as usize;
        let d = io.u32() as usize;
        let inside = c.div_ceil(2)..d.div_ceil(2);
        let answer =
            wavelet_matrix.count(inside.clone(), d + 1..) + wavelet_matrix.count(inside, ..c);
        io.writeln(answer as u64);
    }
    io.flush();
}

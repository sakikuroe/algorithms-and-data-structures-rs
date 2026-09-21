// AtCoder: ABC241 D - Sequence Query
// https://atcoder.jp/contests/abc241/tasks/abc241_d

use anmitsu::ds::wavelet_matrix::WaveletMatrix;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();
    let q = io.u32() as usize;
    let mut queries = Vec::with_capacity(q);
    let mut inserted_values = Vec::new();

    for _ in 0..q {
        let kind = io.u32() as usize;
        let x = io.u64() as usize;
        let k = if kind == 1 { 0 } else { io.u32() as usize };
        if kind == 1 {
            inserted_values.push(x);
        }
        queries.push((kind, x, k));
    }

    let wavelet_matrix = WaveletMatrix::new(&inserted_values);
    let mut length = 0;
    for (kind, x, k) in queries {
        match kind {
            1 => length += 1,
            2 => {
                let answer = wavelet_matrix.get_kth_largest(..length, ..=x, k - 1);
                match answer {
                    Some(value) => io.writeln(value as u64),
                    None => io.writeln(-1_i32),
                }
            }
            3 => {
                let answer = wavelet_matrix.get_kth_smallest(..length, x.., k - 1);
                match answer {
                    Some(value) => io.writeln(value as u64),
                    None => io.writeln(-1_i32),
                }
            }
            _ => unreachable!(),
        }
    }
    io.flush();
}

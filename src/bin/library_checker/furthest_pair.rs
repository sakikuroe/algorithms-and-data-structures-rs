//! Library Checker の最遠点対を、元添字を返す公開 API で求めます。
//! 問題: https://judge.yosupo.jp/problem/furthest_pair

use anmitsu::{
    geometry::{convex_polygon, primitives},
    io::fastio,
};

/// 同一座標が重複しても、入力の異なる二添字を返します。
fn main() {
    let mut io = fastio::Fastio::new();
    let cases = io.u32() as usize;
    let mut output = String::new();
    for _ in 0..cases {
        let n = io.u32() as usize;
        let points = (0..n)
            .map(|_| primitives::Point2 {
                x: io.i64(),
                y: io.i64(),
            })
            .collect::<Vec<_>>();
        let (a, b) =
            convex_polygon::furthest_pair(&points).expect("input must contain at least two points");
        output.push_str(&format!("{a} {b}\n"));
    }
    print!("{output}");
}

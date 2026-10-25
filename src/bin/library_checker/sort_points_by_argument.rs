//! Library Checker の偏角整列を、原点を受け付ける比較関数で解きます。
//! 問題: https://judge.yosupo.jp/problem/sort_points_by_argument

use anmitsu::{
    geometry::{argument, primitives},
    io::fastio,
};

/// 座標の値や距離をタイブレークに追加せず、偏角だけで整列します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let mut points = (0..n)
        .map(|_| primitives::Vector2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    points.sort_by(argument::argument_cmp);
    let mut output = String::new();
    for point in points {
        output.push_str(&format!("{} {}\n", point.x, point.y));
    }
    print!("{output}");
}

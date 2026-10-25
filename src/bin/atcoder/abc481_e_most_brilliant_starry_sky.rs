//! ABC481 E の半平面を、方向列の半周の検索で数えます。
//! 問題: https://atcoder.jp/contests/abc481/tasks/abc481_e

use anmitsu::{
    geometry::{argument, direction, primitives},
    io::fastio,
};

/// 同方向や周回の管理を角度区間に任せ、半平面の最大点数を出力します。
fn main() {
    let mut io = fastio::Fastio::new();
    let cases = io.u32() as usize;
    let mut output = String::new();
    for _ in 0..cases {
        let n = io.u32() as usize;
        let order = argument::PolarOrder::new((0..n).map(|_| {
            direction::Direction::from_vector(primitives::Vector2 {
                x: io.i64(),
                y: io.i64(),
            })
        }));
        // 境界を点の直前へ回した半周を各方向について調べます。
        let answer = order
            .directions()
            .iter()
            .map(|&start| order.ccw_range(start..start.opposite()).len())
            .max()
            .unwrap_or(0);
        output.push_str(&format!("{answer}\n"));
    }
    print!("{output}");
}

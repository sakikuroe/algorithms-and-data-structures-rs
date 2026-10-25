//! ABC139 F の部分集合を、偏角順で連続するベクトル和として調べます。
//! 問題: https://atcoder.jp/contests/abc139/tasks/abc139_f

use anmitsu::{
    geometry::{argument, direction, primitives},
    io::fastio,
};

/// 方向順の元添字とベクトル演算を接続し、最大距離を求めます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let vectors = (0..n)
        .map(|_| primitives::Vector2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    let order = argument::PolarOrder::new(
        vectors
            .iter()
            .copied()
            .map(direction::Direction::from_vector),
    );
    let ids = order.indices();
    let mut best = 0_u128;
    for first in 0..ids.len() {
        let mut sum = primitives::Vector2 { x: 0_i64, y: 0_i64 };
        // 最大となる部分集合は偏角順の連続部分として列挙できます。
        for offset in 0..ids.len() {
            sum += vectors[ids[(first + offset) % ids.len()]];
            best = best.max(sum.norm_squared());
        }
    }
    println!("{:.15}", (best as f64).sqrt());
}

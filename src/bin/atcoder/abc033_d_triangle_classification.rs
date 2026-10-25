//! ABC033 D の直角・鈍角を、整数方向による角度区間で数えます。
//! 問題: https://atcoder.jp/contests/abc033/tasks/abc033_d

use anmitsu::{
    geometry::{argument, direction, primitives},
    io::fastio,
};
use std::ops;

/// 各頂点の角を数え、三角形全体から鋭角三角形を求めます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let points = (0..n)
        .map(|_| primitives::Point2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    let mut right = 0_u64;
    let mut obtuse = 0_u64;
    for &origin in &points {
        let order = argument::PolarOrder::new(
            points
                .iter()
                .map(|&p| direction::Direction::between(origin, p)),
        );
        for &start in order.directions() {
            let quarter = start.rotate_quarter_turns(1);
            // 同じ方向の区間と開区間を使い、誤差なしで 90 度と鈍角を分けます。
            right += order.ccw_range(quarter..=quarter).len() as u64;
            obtuse += order
                .ccw_range((
                    ops::Bound::Excluded(quarter),
                    ops::Bound::Excluded(start.opposite()),
                ))
                .len() as u64;
        }
    }
    let total = n as u64 * (n - 1) as u64 * (n - 2) as u64 / 6;
    println!("{} {right} {obtuse}", total - right - obtuse);
}

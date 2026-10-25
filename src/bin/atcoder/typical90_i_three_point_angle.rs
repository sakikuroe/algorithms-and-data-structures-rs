//! 典型90 009 の角度最大化を、逆方向を挟む二候補から調べます。
//! 問題: https://atcoder.jp/contests/typical90/tasks/typical90_i

use anmitsu::{
    geometry::{argument, direction, primitives},
    io::fastio,
};

/// 各点からの方向を整列し、180 度に最も近い候補を調べます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let points = (0..n)
        .map(|_| primitives::Point2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    let mut answer = 0.0_f64;
    for &origin in &points {
        // 自分自身には方向がないため、元添字を保ったまま分離されます。
        let order = argument::PolarOrder::new(
            points
                .iter()
                .map(|&p| direction::Direction::between(origin, p)),
        );
        let directions = order.directions();
        for &base in directions {
            // 円周の前後候補から選べば、逆方向に最も近い角度を得られます。
            for position in order
                .bracketing_positions(base.opposite())
                .into_iter()
                .flatten()
            {
                let other = directions[position];
                answer = answer.max(base.angle_to(other));
            }
        }
    }
    println!("{:.15}", answer.to_degrees());
}

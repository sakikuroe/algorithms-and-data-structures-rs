//! ABC225 E の可視区間を、方向を端点とする区間選択へ帰着します。
//! 問題: https://atcoder.jp/contests/abc225/tasks/abc225_e

use anmitsu::{
    geometry::{direction, primitives},
    io::fastio,
};

/// 終方向の早い順に、内部が重ならない区間を選びます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let mut intervals = (0..n)
        .map(|_| {
            let x = io.i64();
            let y = io.i64();
            let start = direction::Direction::from_vector(primitives::Vector2 { x, y: y - 1 })
                .expect("lower endpoint must be nonzero");
            let end = direction::Direction::from_vector(primitives::Vector2 { x: x - 1, y })
                .expect("upper endpoint must be nonzero");
            (end, start)
        })
        .collect::<Vec<_>>();
    intervals.sort_unstable();
    let mut previous = None;
    let mut answer = 0;
    for (end, start) in intervals {
        // 共通する端点は可視性を妨げないので、等しい方向も受け付けます。
        if previous.is_none_or(|last| last <= start) {
            answer += 1;
            previous = Some(end);
        }
    }
    println!("{answer}");
}

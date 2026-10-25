//! ABC248 E の直線を、基準点と無向方向の組で集約します。
//! 問題: https://atcoder.jp/contests/abc248/tasks/abc248_e

use anmitsu::{
    geometry::{direction, primitives},
    io::fastio,
};
use std::collections;

/// 直線上で元添字が最小の点だけが、その直線を数えます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let k = io.u32() as usize;
    let points = (0..n)
        .map(|_| primitives::Point2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    if k == 1 {
        println!("Infinity");
        return;
    }
    let mut answer = 0;
    for (i, &origin) in points.iter().enumerate() {
        let mut lines = collections::HashMap::new();
        for (j, &point) in points.iter().enumerate() {
            if let Some(ray) = direction::Direction::between(origin, point) {
                let entry = lines
                    .entry(ray.unoriented())
                    .or_insert((0_usize, usize::MAX));
                entry.0 += 1;
                entry.1 = entry.1.min(j);
            }
        }
        answer += lines
            .values()
            .filter(|&&(count, first)| count + 1 >= k && i < first)
            .count();
    }
    println!("{answer}");
}

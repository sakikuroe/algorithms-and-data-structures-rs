//! Library Checker の凸包を、元添字を返す構築関数で求めます。
//! 問題: https://judge.yosupo.jp/problem/static_convex_hull

use anmitsu::{
    geometry::{convex_polygon, primitives},
    io::fastio,
};

/// 空や共線の点群も同じ公開 API へ渡します。
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
        let hull = convex_polygon::convex_hull_indices(
            &points,
            convex_polygon::CollinearPoints::EndpointsOnly,
        );
        output.push_str(&format!("{}\n", hull.len()));
        for index in hull {
            let point = points[index];
            output.push_str(&format!("{} {}\n", point.x, point.y));
        }
    }
    print!("{output}");
}

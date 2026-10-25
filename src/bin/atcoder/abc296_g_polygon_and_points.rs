//! ABC296 G の問い合わせを、正規化した凸多角形の内外判定で解きます。
//! 問題: https://atcoder.jp/contests/abc296/tasks/abc296_g

use anmitsu::{
    geometry::{convex_polygon, predicates, primitives},
    io::fastio,
};

/// 凸な境界を登録し、頂点・辺上と内部・外部を区別します。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u32() as usize;
    let vertices = (0..n)
        .map(|_| primitives::Point2 {
            x: io.i64(),
            y: io.i64(),
        })
        .collect::<Vec<_>>();
    let polygon = convex_polygon::ConvexPolygon::try_from_boundary(&vertices)
        .expect("input boundary must be convex");
    let queries = io.u32() as usize;
    let mut output = String::new();
    for _ in 0..queries {
        let point = primitives::Point2 {
            x: io.i64(),
            y: io.i64(),
        };
        let answer = match polygon.locate(point) {
            predicates::Location::Inside => "IN",
            predicates::Location::Boundary => "ON",
            predicates::Location::Outside => "OUT",
        };
        output.push_str(answer);
        output.push('\n');
    }
    print!("{output}");
}

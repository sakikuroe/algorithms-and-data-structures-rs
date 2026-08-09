// Library Checker: Tree Diameter
// https://judge.yosupo.jp/problem/tree_diameter

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        let c = io.i64();
        g.add_undirected_edge(a, b, c);
    }

    let (diameter, path) = g.try_tree_diameter_path().unwrap();

    // testlib ベースのチェッカーはトークン単位 (空白・改行を区別しない) で
    // 読み取るため、各値を1行ずつ出力すればよい。
    io.writeln(diameter);
    io.writeln(path.len() as u32);
    for v in &path {
        io.writeln(*v as u32);
    }

    io.flush();
}

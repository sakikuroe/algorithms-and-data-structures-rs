// Library Checker: Minimum Spanning Tree
// https://judge.yosupo.jp/problem/minimum_spanning_tree

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    // ペイロードとして (重み, 元の辺番号) を持たせ、採用された辺から元の入力での
    // 番号を復元できるようにする。
    let mut g = Graph::new(n);
    for i in 0..m {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        let w = io.u32();
        g.add_edge(a, b, (w, i as u32));
    }

    let mst = g.minimum_spanning_forest_by(|&(w, _id)| w);

    // 合計重みは u32 の範囲を超えうるため、u64 に拡張してから加算する。
    let total_weight = mst.edges().map(|(_, _, &(w, _))| w as u64).sum::<u64>();
    io.writeln(total_weight);
    for (_, _, &(_, id)) in mst.edges() {
        io.writeln(id);
    }

    io.flush();
}

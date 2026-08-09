// Baekjoon: 15480 - LCA와 쿼리
// https://www.acmicpc.net/problem/15480

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let u = io.u32() as usize - 1;
        let v = io.u32() as usize - 1;
        g.add_undirected_edge(u, v, ());
    }

    // クエリごとに根が変わるため、あらかじめ適当な頂点 (0) を根に前処理して
    // おき、各クエリでは Hld::lca の root 引数だけを差し替えて答える。
    let hld = g.try_hld(0).unwrap();

    let m = io.u32() as usize;
    for _ in 0..m {
        let r = io.u32() as usize - 1;
        let u = io.u32() as usize - 1;
        let v = io.u32() as usize - 1;
        io.writeln((hld.lca(r, u, v) + 1) as u32);
    }

    io.flush();
}

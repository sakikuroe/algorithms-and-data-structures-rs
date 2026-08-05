// Library Checker: Lowest Common Ancestor
// https://judge.yosupo.jp/problem/lca

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let mut g = Graph::new(n);
    for i in 1..n {
        let p = io.u32() as usize;
        g.add_undirected_edge(p, i, ());
    }

    let hld = g.try_hld(0).unwrap();

    for _ in 0..q {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        io.writeln(hld.lca(0, u, v) as u32);
    }

    io.flush();
}

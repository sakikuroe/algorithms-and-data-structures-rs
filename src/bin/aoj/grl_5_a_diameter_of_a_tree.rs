// AOJ: GRL_5_A - Diameter of a Tree
// https://onlinejudge.u-aizu.ac.jp/courses/library/5/GRL/5/GRL_5_A

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let s = io.u32() as usize;
        let t = io.u32() as usize;
        let w = io.u32() as u64;
        g.add_undirected_edge(s, t, w);
    }

    io.writeln(g.try_tree_diameter().unwrap());

    io.flush();
}

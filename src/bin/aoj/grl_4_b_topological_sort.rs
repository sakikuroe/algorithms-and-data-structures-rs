// AOJ: GRL_4_B - Topological Sort
// https://onlinejudge.u-aizu.ac.jp/problems/GRL_4_B

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let v = io.u32() as usize;
    let e = io.u32() as usize;

    let mut g = Graph::new(v);
    for _ in 0..e {
        let s = io.u32() as usize;
        let t = io.u32() as usize;
        g.add_edge(s, t, ());
    }

    let order = g.topological_sort().unwrap();
    for u in order {
        io.writeln(u as u32);
    }

    io.flush();
}

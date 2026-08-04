// Library Checker: Strongly Connected Components
// https://judge.yosupo.jp/problem/scc

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..m {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        g.add_edge(a, b, ());
    }

    let groups = g.scc().groups();

    io.writeln(groups.len() as u32);
    for group in groups {
        io.writeln(group.len() as u32);
        for v in group {
            io.writeln(v as u32);
        }
    }

    io.flush();
}

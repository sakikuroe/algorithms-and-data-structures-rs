// Library Checker: Cycle Detection (Directed)
// https://judge.yosupo.jp/problem/cycle_detection

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut g = Graph::new(n);
    for i in 0..m {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        g.add_edge(u, v, i as u32);
    }

    match g.find_cycle() {
        Some(cycle) => {
            io.writeln(cycle.len() as u32);
            for e in cycle {
                io.writeln(e);
            }
        }
        None => {
            io.writeln(-1_i32);
        }
    }

    io.flush();
}

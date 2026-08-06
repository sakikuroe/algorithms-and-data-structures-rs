// AtCoder: ABC075 C - Bridge
// https://atcoder.jp/contests/abc075/tasks/abc075_c

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..m {
        let a = io.u32() as usize - 1;
        let b = io.u32() as usize - 1;
        g.add_undirected_edge(a, b, ());
    }

    let bridge_count = g.low_link().bridges().len();
    io.writeln(bridge_count as u32);

    io.flush();
}

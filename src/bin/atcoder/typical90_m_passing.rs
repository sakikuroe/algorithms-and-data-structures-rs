// AtCoder: Typical90 013 - Passing
// https://atcoder.jp/contests/typical90/tasks/typical90_m

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..m {
        let a = io.usize1();
        let b = io.usize1();
        let c = io.u64();
        g.add_undirected_edge(a, b, c);
    }

    // 頂点 k を経由する 1 -> N の最短時間は、1 から k までの最短時間と、
    // k から N までの最短時間の和で表せる。それぞれを別々の Dijkstra 法で
    // まとめて求めておく。
    let from_start = g.dijkstra(&[(0, 0_u64)]);
    let from_goal = g.dijkstra(&[(n - 1, 0_u64)]);
    for k in 0..n {
        io.writeln(from_start.distance(k).unwrap() + from_goal.distance(k).unwrap());
    }

    io.flush();
}

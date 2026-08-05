// AtCoder: 競プロ典型 90 問 039 - Tree Distance (Typical 90 AM)
// https://atcoder.jp/contests/typical90/tasks/typical90_am

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let a = io.u32() as usize - 1;
        let b = io.u32() as usize - 1;
        g.add_undirected_edge(a, b, ());
    }

    let cd = g.try_centroid_decomposition().unwrap();

    // 重心ごとに、部分問題全体でのペアの寄与を加算したうえで、同じ枝に
    // 属するペア (重心を経由しない) の寄与を差し引く。
    let mut total: i64 = 0;
    cd.for_each_component(&g, |_centroid, whole, branches| {
        let whole_sum: i64 = whole.iter().map(|&d| d as i64).sum();
        total += (whole.len() as i64 - 1) * whole_sum;
        for branch in branches {
            let branch_sum: i64 = branch.iter().map(|&d| d as i64).sum();
            total -= (branch.len() as i64 - 1) * branch_sum;
        }
    });

    io.writeln(total);
    io.flush();
}

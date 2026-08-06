// Library Checker: Assignment Problem
// https://judge.yosupo.jp/problem/assignment

use anmitsu::graph::min_cost_flow_graph::MinCostFlowGraph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let a = (0..n)
        .map(|_| (0..n).map(|_| io.i64()).collect::<Vec<i64>>())
        .collect::<Vec<Vec<i64>>>();

    // 行 i を左側の頂点、列 j を右側の頂点とする完全二部グラフを作り、
    // 各行から各列への辺にコスト a[i][j] を持たせる。すべての行・列を
    // ちょうど1回ずつ使う割り当ては、始点から終点への流量 n の最小費用流に
    // 一致する。
    let source = 0;
    let left = |i: usize| 1 + i;
    let right = |j: usize| 1 + n + j;
    let sink = 1 + 2 * n;

    let mut g = MinCostFlowGraph::<i64>::new(2 + 2 * n);
    for i in 0..n {
        g.add_edge(source, left(i), 1, 0);
    }
    for j in 0..n {
        g.add_edge(right(j), sink, 1, 0);
    }
    let edge_id = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| g.add_edge(left(i), right(j), 1, a[i][j]))
                .collect::<Vec<usize>>()
        })
        .collect::<Vec<Vec<usize>>>();

    let (_, cost) = g.min_cost_flow(source, sink);

    // 各行がちょうど1本、流量1の辺を持つはずであり、それが割り当てられた
    // 列にあたる。
    let assignment = (0..n)
        .map(|i| {
            (0..n)
                .find(|&j| g.get_edge(edge_id[i][j]).4 == 1)
                .expect("each row must be assigned to exactly one column")
        })
        .collect::<Vec<usize>>();

    io.writeln(cost);
    for p in assignment {
        io.writeln(p as u32);
    }

    io.flush();
}

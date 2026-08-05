// AtCoder: 競プロ典型 90 問 039 - Tree Distance (Typical 90 AM)
// https://atcoder.jp/contests/typical90/tasks/typical90_am

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;
use std::collections::VecDeque;

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

    // CentroidDecomposition が公開するのは重心木上の parent/level のみであり、
    // 「現在の部分問題に属する頂点集合」や「取り除き済み頂点を避けた探索」は
    // 提供されない。そのため、重心ごとに祖先 (すでに重心として取り除かれた
    // 頂点) の集合を自前で求め、それを避けて手動で BFS を行う。
    //
    // dist/branch_of は全頂点分を使い回し、触れた頂点だけを都度リセットする
    // ことで、重心分解全体を通じた計算量を O(N log N) に保つ。
    let mut blocked = vec![false; n];
    let mut dist: Vec<i64> = vec![-1; n];
    let mut branch_of: Vec<usize> = vec![usize::MAX; n];
    let mut total: i64 = 0;

    for c in 0..n {
        // c の祖先 (すでに重心として取り除かれた頂点) を集める。
        let mut ancestors = Vec::new();
        let mut cur = cd.parent(c);
        while let Some(p) = cur {
            blocked[p] = true;
            ancestors.push(p);
            cur = cd.parent(p);
        }

        // c を起点に、blocked を避けて BFS する。合わせて、各頂点が c の
        // どの直接隣接頂点 (枝) から辿り着いたかを branch_of に記録し、
        // 同じ枝内のペアを後で識別できるようにする。
        let mut component = vec![c];
        dist[c] = 0;
        let mut queue = VecDeque::new();
        queue.push_back(c);
        while let Some(u) = queue.pop_front() {
            for (v, _) in g.edges(u) {
                if !blocked[v] && dist[v] == -1 {
                    dist[v] = dist[u] + 1;
                    branch_of[v] = if u == c { v } else { branch_of[u] };
                    component.push(v);
                    queue.push_back(v);
                }
            }
        }

        // c を含む部分問題全体でのペアの寄与を加算したうえで、同じ枝に
        // 属するペア (c を経由しない) の寄与を差し引く。
        let m = component.len() as i64;
        let whole_sum: i64 = component.iter().map(|&v| dist[v]).sum();
        total += (m - 1) * whole_sum;

        let mut branch_count: Vec<(i64, i64)> = Vec::new(); // (頂点数, 距離の総和)
        let mut branch_index: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for &v in &component {
            if v == c {
                continue;
            }
            let idx = *branch_index.entry(branch_of[v]).or_insert_with(|| {
                branch_count.push((0, 0));
                branch_count.len() - 1
            });
            branch_count[idx].0 += 1;
            branch_count[idx].1 += dist[v];
        }
        for (bm, bsum) in branch_count {
            total -= (bm - 1) * bsum;
        }

        // 次の重心の処理に備え、このラウンドで触れた分だけ状態を戻す。
        for v in component {
            dist[v] = -1;
            branch_of[v] = usize::MAX;
        }
        for p in ancestors {
            blocked[p] = false;
        }
    }

    io.writeln(total);
    io.flush();
}

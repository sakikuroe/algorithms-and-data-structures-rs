// AtCoder: ABC209 D - Collision
// https://atcoder.jp/contests/abc209/tasks/abc209_d

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let a = io.u32() as usize - 1;
        let b = io.u32() as usize - 1;
        g.add_undirected_edge(a, b, ());
    }

    // 木は必ず二部グラフであるため、2彩色すれば全頂点対の距離の偶奇が
    // O(1) で求められる。距離が偶数ならば同じ色 (Town)、奇数ならば
    // 異なる色 (Road) になる。
    let coloring = g.bipartite_coloring().unwrap();

    for _ in 0..q {
        let c = io.u32() as usize - 1;
        let d = io.u32() as usize - 1;
        if coloring[c] == coloring[d] {
            for ch in "Town".chars() {
                io.write(ch);
            }
        } else {
            for ch in "Road".chars() {
                io.write(ch);
            }
        }
        io.write('\n');
    }

    io.flush();
}

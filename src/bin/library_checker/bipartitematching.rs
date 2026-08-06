// Library Checker: Matching on Bipartite Graph
// https://judge.yosupo.jp/problem/bipartitematching

use anmitsu::graph::bipartite_matching::BipartiteGraph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let l = io.u32() as usize;
    let r = io.u32() as usize;
    let m = io.u32() as usize;

    let mut g = BipartiteGraph::new(l, r);
    for _ in 0..m {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        g.add_edge(a, b);
    }

    let matching = g.hopcroft_karp();

    io.writeln(matching.size() as u32);
    for (u, v) in matching.pairs() {
        // Fastio の数値用 write は writeln と同様に必ず改行を伴うため、
        // "u v" を1行にまとめず、それぞれ別の行に出力する
        // (testlib のチェッカーは空白・改行の違いを区別しない)。
        io.writeln(u as u32);
        io.writeln(v as u32);
    }

    io.flush();
}

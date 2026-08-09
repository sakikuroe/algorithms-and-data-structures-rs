// Library Checker: Jump on Tree
// https://judge.yosupo.jp/problem/jump_on_tree

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        g.add_undirected_edge(a, b, ());
    }

    // jump は2頂点間のパス上の移動であり、木上のパス自体は根の取り方に
    // よらず一意に定まるため、根はどこでもよく、ここでは頂点0を選ぶ。
    let hld = g.try_hld(0).unwrap();

    for _ in 0..q {
        let s = io.u32() as usize;
        let t = io.u32() as usize;
        let i = io.u32() as usize;
        match hld.jump(s, t, i) {
            Some(v) => io.writeln(v as u32),
            None => io.writeln(-1_i32),
        }
    }

    io.flush();
}

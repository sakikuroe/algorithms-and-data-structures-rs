// AtCoder: Typical90 021 - Come Back in One Piece
// https://atcoder.jp/contests/typical90/tasks/typical90_u

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
        g.add_edge(a, b, ());
    }

    // x -> y, y -> x の両方が存在するのは、x と y が同じ強連結成分に属する場合に
    // 限る。各成分のサイズ s から、そこに属する2頂点の組の個数 s*(s-1)/2 を求め、
    // 全成分について足し合わせる。
    let scc = g.scc();
    let ans = scc
        .groups()
        .iter()
        .map(|group| {
            let s = group.len() as u64;
            s * (s - 1) / 2
        })
        .sum::<u64>();
    io.writeln(ans);

    io.flush();
}

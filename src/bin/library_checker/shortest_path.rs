// Library Checker: Shortest Path
// https://judge.yosupo.jp/problem/shortest_path

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let s = io.u32() as usize;
    let t = io.u32() as usize;

    let mut g = Graph::new(n);
    for _ in 0..m {
        let a = io.u32() as usize;
        let b = io.u32() as usize;
        let c = io.u64();
        g.add_edge(a, b, c);
    }

    let dijkstra = g.dijkstra(&[(s, 0_u64)]);
    match dijkstra.path_to(t) {
        Some(path) => {
            // ジャッジのチェッカーは testlib ベースであり、トークンを空白・改行の
            // 区別なく読み取るため、1 つずつ改行して出力してもよい。
            io.writeln(dijkstra.distance(t).unwrap());
            io.writeln((path.len() - 1) as u32);
            for edge in path.windows(2) {
                io.writeln(edge[0] as u32);
                io.writeln(edge[1] as u32);
            }
        }
        None => {
            io.writeln(-1_i32);
        }
    }

    io.flush();
}

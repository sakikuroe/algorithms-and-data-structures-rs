// Library Checker: Eulerian Trail (Directed)
// https://judge.yosupo.jp/problem/eulerian_trail_directed

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

/// 文字列トークンを、空白を挟まずにそのまま出力バッファへ書き込む。
fn write_token(io: &mut Fastio, token: &str) {
    for c in token.chars() {
        io.write(c);
    }
}

fn main() {
    let mut io = Fastio::new();

    let t = io.u32();
    for _ in 0..t {
        let n = io.u32() as usize;
        let m = io.u32() as usize;

        let mut g = Graph::new(n);
        for i in 0..m {
            let a = io.u32() as usize;
            let b = io.u32() as usize;
            g.add_edge(a, b, i as u32);
        }

        match g.eulerian_path() {
            Some((vertices, edge_ids)) => {
                write_token(&mut io, "Yes");
                io.write('\n');
                for v in vertices {
                    io.writeln(v as u32);
                }
                for &e in &edge_ids {
                    io.writeln(*e);
                }
            }
            None => {
                write_token(&mut io, "No");
                io.write('\n');
            }
        }
    }

    io.flush();
}

// AOJ: ALDS1_11_B - Depth First Search
// https://onlinejudge.u-aizu.ac.jp/problems/ALDS1_11_B

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「1行に複数の数値をスペース区切りで並べる」形式には使えない。
// char の書き込みだけは改行を伴わないため、数値を文字列化して1文字ずつ書き込むことで
// 同一行への複数値出力を実現する。
fn write_line(io: &mut Fastio, values: &[u32]) {
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            io.write(' ');
        }
        for c in v.to_string().chars() {
            io.write(c);
        }
    }
    io.write('\n');
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let mut g = Graph::new(n);
    for _ in 0..n {
        let u = io.usize1();
        let k = io.u32() as usize;
        for _ in 0..k {
            let v = io.usize1();
            g.add_edge(u, v, ());
        }
    }

    let forest = g.dfs_forest();
    for v in 0..n {
        write_line(
            &mut io,
            &[
                v as u32 + 1,
                forest.discover_time(v) as u32,
                forest.finish_time(v) as u32,
            ],
        );
    }

    io.flush();
}

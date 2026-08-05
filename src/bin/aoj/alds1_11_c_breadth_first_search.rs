// AOJ: ALDS1_11_C - Breadth First Search
// https://onlinejudge.u-aizu.ac.jp/problems/ALDS1_11_C

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「1行に複数の数値をスペース区切りで並べる」形式には使えない。
// char の書き込みだけは改行を伴わないため、数値を文字列化して1文字ずつ書き込むことで
// 同一行への複数値出力を実現する。
fn write_line(io: &mut Fastio, values: &[i64]) {
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

    let bfs = g.bfs(&[0]);
    for v in 0..n {
        let dist = bfs.distance(v).map(|d| d as i64).unwrap_or(-1);
        write_line(&mut io, &[v as i64 + 1, dist]);
    }

    io.flush();
}

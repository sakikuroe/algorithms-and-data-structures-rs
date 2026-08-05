// AOJ: GRL_1_B - Single Source Shortest Path
// https://onlinejudge.u-aizu.ac.jp/problems/GRL_1_B

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「INF のような文字列と数値を同じ行に並べる」形式には使えない。
// char の書き込みだけは改行を伴わないため、あらかじめ文字列化したトークン列を
// 1文字ずつ書き込むことで、任意の内容を1行にまとめて出力する。
fn write_line(io: &mut Fastio, tokens: &[String]) {
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            io.write(' ');
        }
        for c in token.chars() {
            io.write(c);
        }
    }
    io.write('\n');
}

fn main() {
    let mut io = Fastio::new();

    let v = io.u32() as usize;
    let e = io.u32() as usize;
    let r = io.u32() as usize;

    let mut g = Graph::new(v);
    for _ in 0..e {
        let s = io.u32() as usize;
        let t = io.u32() as usize;
        let d = io.i64();
        g.add_edge(s, t, d);
    }

    let bf = g.bellman_ford(&[(r, 0_i64)]);

    // bellman_ford は r を始点として緩和を行うため、affected は r から到達可能な
    // 負閉路の影響を受ける頂点にのみ立つ。よって、これが1つでもあれば
    // 「r から到達可能な負閉路が存在する」ことになる。
    if (0..v).any(|u| bf.is_affected_by_negative_cycle(u)) {
        write_line(&mut io, &["NEGATIVE CYCLE".to_string()]);
    } else {
        for u in 0..v {
            let token = match bf.distance(u) {
                Some(d) => d.to_string(),
                None => "INF".to_string(),
            };
            write_line(&mut io, &[token]);
        }
    }

    io.flush();
}

// AOJ: GRL_1_C - All Pairs Shortest Path
// https://onlinejudge.u-aizu.ac.jp/problems/GRL_1_C

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

    let mut g = Graph::new(v);
    for _ in 0..e {
        let s = io.u32() as usize;
        let t = io.u32() as usize;
        let d = io.i64();
        g.add_edge(s, t, d);
    }

    // Johnson 法は疎グラフ向けのアルゴリズムであるが、正しさの検証としては
    // Floyd-Warshall 法と同じ全点対最短路の問題をそのまま使う。
    match g.johnson(0_i64) {
        Some(johnson) => {
            for i in 0..v {
                let tokens = (0..v)
                    .map(|j| match johnson.distance(i, j) {
                        Some(d) => d.to_string(),
                        None => "INF".to_string(),
                    })
                    .collect::<Vec<String>>();
                write_line(&mut io, &tokens);
            }
        }
        None => {
            write_line(&mut io, &["NEGATIVE CYCLE".to_string()]);
        }
    }

    io.flush();
}

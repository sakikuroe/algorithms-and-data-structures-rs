// SPOJ: QTREE2 - Query on a Tree II
// https://www.spoj.com/problems/QTREE2/

use anmitsu::algebra::monoid::AddMonoid;
use anmitsu::graph::graph::Graph;
use anmitsu::graph::hld_path_query::HldPathQuery;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let t = io.u32() as usize;
    for _ in 0..t {
        let n = io.u32() as usize;
        let edges = (0..n.saturating_sub(1))
            .map(|_| {
                let a = io.usize1();
                let b = io.usize1();
                let c = io.i64();
                (a, b, c)
            })
            .collect::<Vec<(usize, usize, i64)>>();

        let mut g = Graph::new(n);
        for &(a, b, _) in &edges {
            g.add_undirected_edge(a, b, ());
        }
        let hld = g.try_hld(0).unwrap();
        // 頂点の初期値は使われない (根は対応する辺を持たず、他の頂点は直後の
        // set_edge で辺のコストに上書きされるため)。この問題に辺の更新は
        // 無いが、コストの取得には HldPathQuery の辺クエリをそのまま使う。
        let mut path_query = HldPathQuery::<AddMonoid>::new(&hld, &vec![0_i64; n]);
        for &(a, b, c) in &edges {
            path_query.set_edge(a, b, c);
        }

        loop {
            let command = io.chars().iter().collect::<String>();
            match command.as_str() {
                "DONE" => break,
                "DIST" => {
                    let a = io.usize1();
                    let b = io.usize1();
                    io.writeln(path_query.fold_edge_path(a, b));
                }
                "KTH" => {
                    let a = io.usize1();
                    let b = io.usize1();
                    let k = io.u32() as usize;
                    // k は 1-indexed (a 自身が1番目) であり、hld.jump は a から
                    // 0-indexed の歩数を取るため、k-1 歩進んだ頂点を answer とする。
                    let answer = hld.jump(a, b, k - 1).unwrap();
                    io.writeln(answer as u32 + 1);
                }
                _ => unreachable!("未知のコマンド: {command}"),
            }
        }

        io.write('\n');
    }

    io.flush();
}

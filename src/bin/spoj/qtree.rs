// SPOJ: QTREE - Query on a Tree
// https://www.spoj.com/problems/QTREE/

use anmitsu::algebra::monoid::MaxMonoid;
use anmitsu::graph::graph::Graph;
use anmitsu::graph::hld_path_query::HldEdgePathQuery;
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
        let mut path_query = HldEdgePathQuery::<MaxMonoid>::new(&hld);
        for &(a, b, c) in &edges {
            path_query.set_edge(a, b, c);
        }

        loop {
            let command = io.chars().iter().collect::<String>();
            match command.as_str() {
                "DONE" => break,
                "CHANGE" => {
                    let i = io.u32() as usize;
                    let cost = io.i64();
                    let (a, b, _) = edges[i - 1];
                    path_query.set_edge(a, b, cost);
                }
                "QUERY" => {
                    let a = io.usize1();
                    let b = io.usize1();
                    io.writeln(path_query.fold_edge_path(a, b));
                }
                _ => unreachable!("未知のコマンド: {command}"),
            }
        }
    }

    io.flush();
}

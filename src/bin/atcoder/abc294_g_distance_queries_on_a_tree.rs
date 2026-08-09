// AtCoder: ABC294-G - Distance Queries on a Tree
// https://atcoder.jp/contests/abc294/tasks/abc294_g

use anmitsu::algebra::monoid::AddMonoid;
use anmitsu::graph::graph::Graph;
use anmitsu::graph::hld_path_query::HldEdgePathQuery;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let edges = (0..n.saturating_sub(1))
        .map(|_| {
            let u = io.usize1();
            let v = io.usize1();
            let w = io.i64();
            (u, v, w)
        })
        .collect::<Vec<(usize, usize, i64)>>();

    let mut g = Graph::new(n);
    for &(u, v, _) in &edges {
        g.add_undirected_edge(u, v, ());
    }
    let hld = g.try_hld(0).unwrap();
    let mut path_query = HldEdgePathQuery::<AddMonoid>::new(&hld);
    for &(u, v, w) in &edges {
        path_query.set_edge(u, v, w);
    }

    let q = io.u32() as usize;
    for _ in 0..q {
        let kind = io.u32();
        if kind == 1 {
            let i = io.u32() as usize;
            let w = io.i64();
            let (u, v, _) = edges[i - 1];
            path_query.set_edge(u, v, w);
        } else {
            let u = io.usize1();
            let v = io.usize1();
            io.writeln(path_query.fold_edge_path(u, v));
        }
    }

    io.flush();
}

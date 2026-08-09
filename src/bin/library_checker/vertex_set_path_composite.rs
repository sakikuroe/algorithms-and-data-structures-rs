// Library Checker: Vertex Set Path Composite
// https://judge.yosupo.jp/problem/vertex_set_path_composite

use anmitsu::algebra::monoid::AffineMonoid;
use anmitsu::graph::graph::Graph;
use anmitsu::graph::hld_path_query::HldVertexPathQuery;
use anmitsu::io::fastio::Fastio;

const MOD: u64 = 998244353;
type Affine = AffineMonoid<MOD>;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    let functions = (0..n)
        .map(|_| {
            let a = io.u64();
            let b = io.u64();
            (a, b)
        })
        .collect::<Vec<(u64, u64)>>();

    let mut g = Graph::new(n);
    for _ in 0..n - 1 {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        g.add_undirected_edge(u, v, ());
    }

    let hld = g.try_hld(0).unwrap();
    let mut path_query = HldVertexPathQuery::<Affine>::new(&hld, &functions);

    for _ in 0..q {
        let kind = io.u32();
        if kind == 0 {
            let p = io.u32() as usize;
            let c = io.u64();
            let d = io.u64();
            path_query.set_vertex(p, (c, d));
        } else {
            let u = io.u32() as usize;
            let v = io.u32() as usize;
            let x = io.u64();

            let (a, b) = path_query.fold_vertex_path(u, v);
            let answer = (u128::from(a) * u128::from(x) + u128::from(b)) % u128::from(MOD);
            io.writeln(answer as u64);
        }
    }

    io.flush();
}

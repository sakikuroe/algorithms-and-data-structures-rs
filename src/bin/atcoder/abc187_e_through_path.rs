// AtCoder: ABC187 E - Through Path
// https://atcoder.jp/contests/abc187/tasks/abc187_e

use anmitsu::algebra::monoid;
use anmitsu::ds::segment_tree::segment_tree_dense::SegmentTreeDense;
use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

/// HLD の番号空間上で、区間 `[l, r)` に `x` を加算する (差分をとった配列に
/// 対する点更新2箇所で表現し、最終的な値は先頭からの累積和として求める)。
fn range_add(seg: &mut SegmentTreeDense<monoid::AddMonoid>, l: usize, r: usize, x: i64) {
    if l == r {
        return;
    }
    seg.update(l, seg.get(l) + x);
    if r < seg.len() {
        seg.update(r, seg.get(r) - x);
    }
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;

    let mut g = Graph::new(n);
    let mut edges = Vec::with_capacity(n - 1);
    for _ in 0..n - 1 {
        let a = io.u32() as usize - 1;
        let b = io.u32() as usize - 1;
        g.add_undirected_edge(a, b, ());
        edges.push((a, b));
    }

    let hld = g.try_hld(0).unwrap();

    // 全頂点の初期値は0であり、`new` 直後の状態がすでにその集約結果に
    // 一致するため、`set`/`build` を経由する必要はない。
    let mut seg = SegmentTreeDense::<monoid::AddMonoid>::new(n);

    let q = io.u32() as usize;
    for _ in 0..q {
        let t = io.u32();
        let e = io.u32() as usize - 1;
        let x = io.i64();

        let (a, b) = edges[e];
        // t=1 は a から b を経由せずに到達できる頂点、t=2 は b から a を
        // 経由せずに到達できる頂点が対象。深い方の端点が子であり、その
        // 部分木を切り離すことに対応するため、対象がどちら側かで
        // 「子の部分木そのもの」か「それ以外全体」かが決まる。
        let (target, avoid) = if t == 1 { (a, b) } else { (b, a) };
        if hld.depth(target) > hld.depth(avoid) {
            let (l, r) = hld.subtree_range(target);
            range_add(&mut seg, l, r, x);
        } else {
            let (l, r) = hld.subtree_range(avoid);
            range_add(&mut seg, 0, l, x);
            range_add(&mut seg, r, n, x);
        }
    }

    for v in 0..n {
        io.writeln(seg.fold(0, hld.vertex_id(v) + 1));
    }

    io.flush();
}

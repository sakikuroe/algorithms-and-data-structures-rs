// Library Checker: Range Affine Range Sum
// https://judge.yosupo.jp/problem/range_affine_range_sum

use anmitsu::ds::segment_tree::{lazy_segment_tree, range_affine_range_sum};
use anmitsu::io::fastio;
use anmitsu::modulo998244353::modint;

fn main() {
    let mut io = fastio::Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    // 各要素を (値, 要素数=1) のペアとして読み込む。
    let v = (0..n)
        .map(|_| (modint::ModInt998244353::new(io.u64()), 1_usize))
        .collect::<Vec<_>>();

    // 区間アフィン変換・区間和の遅延セグメント木を構築する。
    let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
        range_affine_range_sum::RangeAffineFoldSumMonoid,
        range_affine_range_sum::AffineAction,
    >::from_vec(v);

    for _ in 0..q {
        let t = io.u32();
        if t == 0 {
            // 区間 [l, r) にアフィン変換 f(x) = b*x + c を適用する。
            let l = io.u32() as usize;
            let r = io.u32() as usize;
            let b = modint::ModInt998244353::new(io.u64());
            let c = modint::ModInt998244353::new(io.u64());
            seg.effect(l, r, range_affine_range_sum::AffineAction::affine(b, c));
        } else {
            // 区間 [l, r) の和を出力する。
            let l = io.u32() as usize;
            let r = io.u32() as usize;
            let (sum, _) = seg.fold(l, r);
            io.writeln(sum.val());
        }
    }

    io.flush();
}

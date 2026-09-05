// AtCoder: ABC473 F - Valid String
// https://atcoder.jp/contests/abc473/tasks/abc473_f

use anmitsu::algebra::{monoid, semi_group};
use anmitsu::ds::segment_tree::segment_tree_dense;
use anmitsu::io::fastio;

/// 有効文字列判定用のモノイドである。
///
/// 各区間を `(sum, min_prefix)` の対で管理する。`sum` は 'A' を +1、'B' を -1 と
/// みなしたときの区間内の総和であり、`min_prefix` は区間の先頭からの累積和の最小値である。
/// 空文字列に 'A' または 'AB' を任意の位置に挿入して構成できる文字列の必要十分条件は、
/// 対象区間の `min_prefix` が 0 以上であることに等しい。
struct ValidStringMonoid;

impl semi_group::SemiGroup for ValidStringMonoid {
    type S = (i64, i64);

    /// 左右の区間を結合する。右区間の接頭辞累積和を左区間の総和で底上げし、
    /// 全体の最小接頭辞累積和を求める。
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        (a.0 + b.0, a.1.min(a.0.saturating_add(b.1)))
    }
}

impl monoid::Monoid for ValidStringMonoid {
    /// 空区間に対する単位元を返す。総和は 0、最小接頭辞累積和は
    /// `i64::MAX` とすることで、`min` 演算で他方の値を変化させない。
    fn id() -> Self::S {
        (0, i64::MAX)
    }
}

/// 文字 'A' または 'B' に対応するモノイド値を返す。
fn char_to_val(c: char) -> (i64, i64) {
    let v = if c == 'A' { 1 } else { -1 };
    (v, v)
}

fn main() {
    let mut io = fastio::Fastio::new();

    let n = io.u32() as usize;

    // 初期文字列を読み込み、各文字をモノイド値に変換してセグメント木を構築する。
    let s = io.chars();
    let q = io.u32() as usize;

    let mut seg = segment_tree_dense::SegmentTreeDense::<ValidStringMonoid>::new(n);
    for i in 0..n {
        seg.set(i, char_to_val(s[i]));
    }
    seg.build();

    for _ in 0..q {
        let t = io.u32();
        if t == 1 {
            // 1-indexed の位置 i の文字を c に変更する。
            let i = io.u32() as usize - 1;
            let c = io.char();
            seg.update(i, char_to_val(c));
        } else {
            // 1-indexed の区間 [l, r] が有効文字列かを判定する。
            let l = io.u32() as usize - 1;
            let r = io.u32() as usize;
            let result = seg.fold(l, r);
            if result.1 >= 0 {
                io.write('Y');
                io.write('e');
                io.write('s');
            } else {
                io.write('N');
                io.write('o');
            }
            io.write('\n');
        }
    }

    io.flush();
}

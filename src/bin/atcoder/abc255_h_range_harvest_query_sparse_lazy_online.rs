//! ABC255 H - Range Harvest Query を疎な遅延セグメント木で解きます。
//! https://atcoder.jp/contests/abc255/tasks/abc255_h

use anmitsu::{
    algebra::{monoid, semi_group},
    ds::segment_tree::lazy_segment_tree_sparse_online,
    io::fastio,
};

/// 収穫量を出力する際の法。
const MOD: u64 = 998_244_353;

/// 区間長、最終収穫日の和、区間内の番号で重み付けした収穫日の和。
///
/// 各成分は MOD で割ったあまりです。番号は区間の先頭を 1 とします。
struct HarvestMonoid;

impl semi_group::SemiGroup for HarvestMonoid {
    type S = (u64, u64, u64);

    /// 右区間の番号を左区間の長さだけずらして連結します。
    fn op(left: &Self::S, right: &Self::S) -> Self::S {
        (
            (left.0 + right.0) % MOD,
            (left.1 + right.1) % MOD,
            (left.2 + right.2 + left.0 * right.1) % MOD,
        )
    }
}

impl monoid::Monoid for HarvestMonoid {
    /// 長さ 0 の区間を表します。
    fn id() -> Self::S {
        (0, 0, 0)
    }
}

/// 最終収穫日を指定日へ上書きする作用。
#[derive(Clone)]
struct HarvestDay(u64);

impl lazy_segment_tree_sparse_online::RangeAction<(u64, u64, u64)> for HarvestDay {
    /// 全要素の収穫日が等しくなるため、番号の和から加重和を求めます。
    fn apply(&self, _value: &(u64, u64, u64), len: usize) -> (u64, u64, u64) {
        let len = len as u64 % MOD;
        (len, self.0 * len % MOD, self.0 * triangular(len) % MOD)
    }

    /// 後から指定された収穫日を優先します。
    fn composition(&self, other: &Self) -> Self {
        Self(other.0)
    }
}

/// 1 から n までの和を MOD で割ったあまりを求めます。
///
/// # Args
/// - `n` - 和の上端を MOD で割ったあまり。
///
/// # Returns
/// 等差数列の和を MOD で割ったあまりを返します。
fn triangular(n: u64) -> u64 {
    // 各乗算の前に剰余を取り、u64 の範囲内で計算します。
    n * (n + 1) % MOD * MOD.div_ceil(2) % MOD
}

/// 各収穫区間の成長総量から前回までの収穫量を引きます。
fn main() {
    let mut io = fastio::Fastio::new();
    let n = io.u64() as usize;
    let q = io.u32() as usize;
    let mut seg = lazy_segment_tree_sparse_online::SegmentTreeLazySparseOnline::<
        HarvestMonoid,
        HarvestDay,
    >::new(n);

    // 未収穫区間にも長さを持たせ、連結時の番号のずれを正しく求めます。
    seg.effect(.., HarvestDay(0));
    for _ in 0..q {
        let day = io.u64() % MOD;
        let left = io.u64() as usize - 1;
        let right = io.u64() as usize;
        let range = left..right;
        let (len, days, weighted_days) = seg.fold(range.clone());
        let offset = range.start as u64 % MOD;
        // 区間内の番号を木の番号に戻し、既収穫分を差し引きます。
        let growth = day * (triangular(len) + offset * len % MOD) % MOD;
        let harvested = (weighted_days + offset * days) % MOD;
        io.writeln((growth + MOD - harvested) % MOD);
        seg.effect(range, HarvestDay(day));
    }
    io.flush();
}

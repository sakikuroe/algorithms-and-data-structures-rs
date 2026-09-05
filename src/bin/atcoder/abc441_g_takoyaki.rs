// AtCoder: ABC441 G - Takoyaki
// https://atcoder.jp/contests/abc441/tasks/abc441_g

use anmitsu::algebra::{monoid, semi_group};
use anmitsu::ds::segment_tree::lazy_segment_tree;
use anmitsu::io::fastio;

/// 各区間が保持するデータ。表向きの皿のたこ焼き最大値と、
/// 表向き・裏向きの皿の枚数を管理する。
#[derive(Clone)]
struct PlateData {
    /// 区間内の表向きの皿におけるたこ焼きの最大値。
    /// 表向きの皿が存在しない場合は `i64::MIN` とする。
    max_up: i64,
    /// 表向きの皿の枚数。
    count_up: usize,
    /// 裏向きの皿の枚数。
    count_down: usize,
}

/// `PlateData` に対するモノイドの定義。
struct PlateMonoid;

impl semi_group::SemiGroup for PlateMonoid {
    type S = PlateData;
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        PlateData {
            max_up: a.max_up.max(b.max_up),
            count_up: a.count_up + b.count_up,
            count_down: a.count_down + b.count_down,
        }
    }
}

impl monoid::Monoid for PlateMonoid {
    fn id() -> Self::S {
        PlateData {
            max_up: i64::MIN,
            count_up: 0,
            count_down: 0,
        }
    }
}

/// 区間に対する遅延作用。3 種類の操作を統一的に表現する。
///
/// 意味: まず `zero` が真なら全値を 0 にリセットし、`flip` が
/// 真なら全皿をひっくり返し、最後に表向きの皿に `add` を加算する。
/// 操作の合成により `flip` が真のとき `zero` も必ず真になる
/// ことが帰納的に保証される。
#[derive(Clone)]
struct PlateAction {
    zero: bool,
    flip: bool,
    add: i64,
}

impl lazy_segment_tree::Hom<PlateData> for PlateAction {
    /// 作用をノードの値に適用する。
    fn f(&self, x: &PlateData) -> PlateData {
        if self.flip {
            // `flip` が真のとき `zero` も真であるため、元の値は
            // すべて 0 にリセットされる。もともと裏向きだった皿が
            // 表向きになり、加算値がそのまま最大値となる。
            PlateData {
                max_up: if x.count_down > 0 { self.add } else { i64::MIN },
                count_up: x.count_down,
                count_down: x.count_up,
            }
        } else {
            PlateData {
                max_up: if x.count_up > 0 {
                    (if self.zero { 0 } else { x.max_up }) + self.add
                } else {
                    i64::MIN
                },
                count_up: x.count_up,
                count_down: x.count_down,
            }
        }
    }

    /// `self` を先に適用し `other` を後から適用する合成を返す。
    fn composition(&self, other: &Self) -> Self {
        PlateAction {
            zero: self.zero || other.zero,
            flip: self.flip ^ other.flip,
            add: if other.zero {
                other.add
            } else {
                self.add + other.add
            },
        }
    }
}

fn main() {
    let mut io = fastio::Fastio::new();

    let n = io.u32() as usize;
    let q = io.u32() as usize;

    // 初期状態: すべての皿が表向きで、たこ焼きは 0 個である。
    let init = PlateData {
        max_up: 0,
        count_up: 1,
        count_down: 0,
    };
    let mut seg =
        lazy_segment_tree::SegmentTreeLazyDense::<PlateMonoid, PlateAction>::from_vec(vec![
            init;
            n
        ]);

    for _ in 0..q {
        let t = io.u32();
        match t {
            1 => {
                let l = io.u32() as usize;
                let r = io.u32() as usize;
                let x = io.i64();
                seg.effect(
                    l - 1,
                    r,
                    PlateAction {
                        zero: false,
                        flip: false,
                        add: x,
                    },
                );
            }
            2 => {
                let l = io.u32() as usize;
                let r = io.u32() as usize;
                seg.effect(
                    l - 1,
                    r,
                    PlateAction {
                        zero: true,
                        flip: true,
                        add: 0,
                    },
                );
            }
            3 => {
                let l = io.u32() as usize;
                let r = io.u32() as usize;
                let result = seg.fold(l - 1, r);
                // 裏向きの皿は常にたこ焼き 0 個であるため、
                // 表向きの最大値と 0 の大きい方が答えとなる。
                let ans = result.max_up.max(0);
                io.writeln(ans);
            }
            _ => unreachable!(),
        }
    }

    io.flush();
}

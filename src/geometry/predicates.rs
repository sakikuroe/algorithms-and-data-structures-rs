//! 整数座標の三点の向き、閉線分上の判定、線分同士の関係
//!
//! i64 の全座標域で整数のまま判定します。交点の座標は構成しません。

use super::{kernel, primitives};
use std::cmp;

/// 三点の回転方向です。同一点を含む場合も共線です。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    /// 時計回り。
    Clockwise,
    /// 同一直線上。
    Collinear,
    /// 反時計回り。
    CounterClockwise,
}

/// 図形に対する点の位置です。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// 外部。
    Outside,
    /// 辺・頂点、または退化した図形上。
    Boundary,
    /// 二次元の内部。
    Inside,
}

/// 閉線分同士の関係です。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentRelation {
    /// 共通部分がありません。
    Disjoint,
    /// 一点で接触します。長さゼロの線分の接触も含みます。
    Touching,
    /// 二つの非退化な線分の内部同士が一点で交わります。
    Crossing,
    /// 正の長さの共通部分を持ちます。
    Overlapping,
}

/// a→b と a→c の外積の符号を分類します。
///
/// # Args
/// - `a`, `b`, `c`: この順で向きを調べる三点。
/// # Returns
/// 時計回り・共線・反時計回りのいずれか。
/// # Complexity
/// 時間・追加領域とも O(1)。
pub fn orientation(
    a: primitives::Point2<i64>,
    b: primitives::Point2<i64>,
    c: primitives::Point2<i64>,
) -> Orientation {
    match kernel::cross_sign(
        kernel::difference((a.x, a.y), (b.x, b.y)),
        kernel::difference((a.x, a.y), (c.x, c.y)),
    ) {
        cmp::Ordering::Less => Orientation::Clockwise,
        cmp::Ordering::Equal => Orientation::Collinear,
        cmp::Ordering::Greater => Orientation::CounterClockwise,
    }
}

/// 点が端点を含む閉線分上にあるかを判定します。
///
/// # Args
/// - `point`: 判定する点。
/// - `a`, `b`: 線分の両端。同一点も受け付けます。
/// # Returns
/// 共線であり、両座標が端点の範囲に入る場合に真。
/// # Complexity
/// 時間・追加領域とも O(1)。
pub fn on_segment(
    point: primitives::Point2<i64>,
    a: primitives::Point2<i64>,
    b: primitives::Point2<i64>,
) -> bool {
    orientation(a, b, point) == Orientation::Collinear
        && (a.x.min(b.x)..=a.x.max(b.x)).contains(&point.x)
        && (a.y.min(b.y)..=a.y.max(b.y)).contains(&point.y)
}

/// 長さゼロを含む二つの閉線分の関係を判定します。
///
/// # Args
/// - `a`, `b`: 一つ目の線分の両端。
/// - `c`, `d`: 二つ目の線分の両端。
/// # Returns
/// 非交差・一点接触・内部の横断・正の長さの重なりのいずれか。
/// # Complexity
/// 時間・追加領域とも O(1)。
pub fn segment_relation(
    a: primitives::Point2<i64>,
    b: primitives::Point2<i64>,
    c: primitives::Point2<i64>,
    d: primitives::Point2<i64>,
) -> SegmentRelation {
    let first = (orientation(a, b, c), orientation(a, b, d));
    let second = (orientation(c, d, a), orientation(c, d, b));
    if first == (Orientation::Collinear, Orientation::Collinear)
        && second == (Orientation::Collinear, Orientation::Collinear)
    {
        // 共線なら辞書順の閉区間として、長さのある重なりと一点を区別できます。
        let a = (a.x, a.y);
        let b = (b.x, b.y);
        let c = (c.x, c.y);
        let d = (d.x, d.y);
        return match a.min(b).max(c.min(d)).cmp(&a.max(b).min(c.max(d))) {
            cmp::Ordering::Less => SegmentRelation::Overlapping,
            cmp::Ordering::Equal => SegmentRelation::Touching,
            cmp::Ordering::Greater => SegmentRelation::Disjoint,
        };
    }
    if on_segment(a, c, d) || on_segment(b, c, d) || on_segment(c, a, b) || on_segment(d, a, b) {
        return SegmentRelation::Touching;
    }
    if first.0 != first.1
        && second.0 != second.1
        && first.0 != Orientation::Collinear
        && first.1 != Orientation::Collinear
        && second.0 != Orientation::Collinear
        && second.1 != Orientation::Collinear
    {
        SegmentRelation::Crossing
    } else {
        SegmentRelation::Disjoint
    }
}

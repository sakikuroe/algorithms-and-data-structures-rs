//! 元添字を返す凸包と、正規化した凸多角形の問い合わせ・演算
//!
//! 頂点列は辞書順最小の頂点から反時計回りで、閉じる頂点の再掲や辺の途中の
//! 共線点を持ちません。空・一点・線分も同じ型で扱います。面積は取得時にだけ
//! 計算するため、u128 に収まらない図形も内外判定や最遠点の選択に使えます。
//!
//! ```
//! use anmitsu::geometry::{convex_polygon, predicates, primitives};
//! let points = [(0, 0), (3, 0), (3, 2), (0, 2)].map(|(x, y)| primitives::Point2 { x, y });
//! let polygon = convex_polygon::ConvexPolygon::from_points(&points);
//! assert_eq!(12, polygon.area2());
//! assert_eq!(predicates::Location::Inside, polygon.locate(primitives::Point2 { x: 1, y: 1 }));
//! let (a, b) = polygon.furthest_pair().unwrap();
//! assert_eq!(13, polygon.vertices()[a].distance_squared(polygon.vertices()[b]));
//! ```

use super::{direction, kernel, predicates, primitives};
use std::{cmp, error, fmt};

/// 凸包の辺の途中にある共線点を残すかどうかです。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollinearPoints {
    /// 各辺の両端だけを残します。
    EndpointsOnly,
    /// 異なる座標の境界点をすべて残します。
    AllBoundary,
}

/// 指定方向の内積を最大にする頂点、または辺です。添字は vertices 上の位置です。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportFace {
    /// 一頂点だけが最大です。
    Vertex(usize),
    /// 一辺全体が最大です。三頂点以上では反時計回りの隣接頂点です。
    Edge {
        /// 辺の始点。
        start: usize,
        /// 辺の終点。線分全体では start = 0、end = 1 です。
        end: usize,
    },
}

/// 頂点列が単純な凸境界として受理できないことを表します。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NonConvexError;

impl fmt::Display for NonConvexError {
    /// 構築できなかった境界の種類を示します。
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("vertices do not form a convex boundary")
    }
}
impl error::Error for NonConvexError {}

/// 正規化した空集合・一点・線分・狭義凸多角形です。
#[derive(Clone, Debug)]
pub struct ConvexPolygon {
    /// 辞書順最小の頂点から反時計回りに並ぶ座標列。
    vertices: Vec<primitives::Point2<i64>>,
    /// 外向き法線の絶対偏角が最小となる辺の位置。
    normal_start: usize,
}

/// 三角形の符号付き面積の二倍を、狭い差を作らず計算します。
///
/// # Args
/// - `a`, `b`, `c`: この順で向きを持つ三点。
/// # Returns
/// 差と積を狭めない符号付きの面積値。O(1) 時間で取得します。
fn signed_area(
    a: primitives::Point2<i64>,
    b: primitives::Point2<i64>,
    c: primitives::Point2<i64>,
) -> kernel::Wide {
    kernel::cross(
        kernel::difference((a.x, a.y), (b.x, b.y)),
        kernel::difference((a.x, a.y), (c.x, c.y)),
    )
}

/// 三点の向きだけを、面積の値を作らず判定します。
///
/// # Args
/// - `a`, `b`, `c`: この順で向きを調べる三点。
/// # Returns
/// 外積の符号を表す比較結果。O(1) 時間で取得します。
#[inline]
fn turn(
    a: primitives::Point2<i64>,
    b: primitives::Point2<i64>,
    c: primitives::Point2<i64>,
) -> cmp::Ordering {
    kernel::cross_sign(
        kernel::difference((a.x, a.y), (b.x, b.y)),
        kernel::difference((a.x, a.y), (c.x, c.y)),
    )
}

/// 点群の凸包を、元の点列の添字として返します。
///
/// # Args
/// - `points`: 重複や空を含めてよい点群。
/// - `collinear`: 辺上の共線点を残す方針。
/// # Returns
/// 辞書順最小の頂点から反時計回りの元添字。同一座標の代表は最小の元添字です。
/// 全点共線では EndpointsOnly は両端、AllBoundary は辞書順の全点を返します。
/// 閉じる頂点の再掲は行いません。
/// # Complexity
/// O(n log n) 時間、O(n) 追加領域。
pub fn convex_hull_indices(
    points: &[primitives::Point2<i64>],
    collinear: CollinearPoints,
) -> Vec<usize> {
    let mut sorted = (0..points.len()).collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|&i| (points[i].x, points[i].y, i));
    sorted.dedup_by_key(|i| (points[*i].x, points[*i].y));
    if sorted.len() <= 2 {
        return sorted;
    }
    let first = points[sorted[0]];
    let last = points[*sorted.last().unwrap()];
    if sorted.iter().all(|&i| turn(first, last, points[i]).is_eq()) {
        return if collinear == CollinearPoints::AllBoundary {
            sorted
        } else {
            vec![sorted[0], *sorted.last().unwrap()]
        };
    }
    // 二つの単調な鎖を作り、共線点の方針に従って中間頂点を除きます。
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for &index in &sorted {
        append_hull_vertex(points, &mut lower, index, collinear);
    }
    for &index in sorted.iter().rev() {
        append_hull_vertex(points, &mut upper, index, collinear);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Andrew の一方の鎖へ、向きの不変条件を保って頂点を追加します。
///
/// # Args
/// - `points`: 元の点群。
/// - `chain`: 走査済みの凸な鎖。必要な末尾を取り除いて追加します。
/// - `index`: 座標の走査順で次に追加する元添字。
/// - `collinear`: 共線点を残す方針。
/// # Complexity
/// 鎖全体の各頂点を高々一度取り除く償却 O(1) 時間。
fn append_hull_vertex(
    points: &[primitives::Point2<i64>],
    chain: &mut Vec<usize>,
    index: usize,
    collinear: CollinearPoints,
) {
    while chain.len() >= 2 {
        let sign = turn(
            points[chain[chain.len() - 2]],
            points[chain[chain.len() - 1]],
            points[index],
        );
        if sign.is_gt() || sign.is_eq() && collinear == CollinearPoints::AllBoundary {
            break;
        }
        chain.pop();
    }
    chain.push(index);
}

/// 点群の異なる元添字の最遠点対を返します。
///
/// # Args
/// - `points`: 元の点群。同一座標も別の要素として数えます。
/// # Returns
/// 小さい添字から並べた対。二要素未満なら None。同率の選択は任意です。
/// 同一点だけの入力も二要素以上なら異なる二添字を返します。
/// # Complexity
/// O(n log n) 時間、O(n) 追加領域。
pub fn furthest_pair(points: &[primitives::Point2<i64>]) -> Option<(usize, usize)> {
    if points.len() < 2 {
        return None;
    }
    let indices = convex_hull_indices(points, CollinearPoints::EndpointsOnly);
    if indices.len() == 1 {
        return Some((0, 1));
    }
    let polygon = ConvexPolygon::from_ccw_vertices(indices.iter().map(|&i| points[i]).collect());
    let (a, b) = polygon.furthest_pair().unwrap();
    Some((indices[a].min(indices[b]), indices[a].max(indices[b])))
}

impl ConvexPolygon {
    /// 点群から狭義凸包を構築します。
    ///
    /// # Args
    /// - `points`: 空・重複・共線点を含めてよい点群。
    /// # Returns
    /// 正規化した凸多角形。空や一点・線分への退化も受け付けます。
    /// # Complexity
    /// O(n log n) 時間、O(n) 領域。
    pub fn from_points(points: &[primitives::Point2<i64>]) -> Self {
        let ids = convex_hull_indices(points, CollinearPoints::EndpointsOnly);
        Self::from_ccw_vertices(ids.into_iter().map(|i| points[i]).collect())
    }

    /// 凸な境界を検査し、向きと重複・共線点を正規化します。
    ///
    /// # Args
    /// - `vertices`: 時計回りも可。隣接重複と末尾の先頭点の再掲を許します。
    /// # Returns
    /// 凸多角形。空・全点重複・全点共線は順序によらず受理します。
    /// 二次元では内点、逆走、非隣接の重複、複数周、自己交差を Err とします。
    /// # Complexity
    /// O(n log n) 時間、O(n) 追加領域。
    pub fn try_from_boundary(vertices: &[primitives::Point2<i64>]) -> Result<Self, NonConvexError> {
        let hull = convex_hull_indices(vertices, CollinearPoints::AllBoundary);
        if hull.len() <= 2 {
            return Ok(Self::from_ccw_vertices(
                hull.into_iter().map(|i| vertices[i]).collect(),
            ));
        }
        if turn(
            vertices[hull[0]],
            vertices[hull[1]],
            vertices[*hull.last().unwrap()],
        )
        .is_eq()
        {
            return Ok(Self::from_ccw_vertices(vec![
                vertices[hull[0]],
                vertices[*hull.last().unwrap()],
            ]));
        }
        let mut boundary = Vec::new();
        for &point in vertices {
            if boundary.last() != Some(&point) {
                boundary.push(point);
            }
        }
        if boundary.first() == boundary.last() {
            boundary.pop();
        }
        if boundary.len() != hull.len() {
            return Err(NonConvexError);
        }
        let size = boundary.len();
        let start = boundary
            .iter()
            .position(|&p| p == vertices[hull[0]])
            .ok_or(NonConvexError)?;
        // 局所的な向きだけでなく、凸包の全巡回順と正逆双方で照合します。
        let forward = (0..size).all(|i| boundary[(start + i) % size] == vertices[hull[i]]);
        let backward = (0..size).all(|i| boundary[(start + size - i) % size] == vertices[hull[i]]);
        if !forward && !backward {
            return Err(NonConvexError);
        }
        let strict = (0..size)
            .filter(|&i| {
                !turn(
                    vertices[hull[(i + size - 1) % size]],
                    vertices[hull[i]],
                    vertices[hull[(i + 1) % size]],
                )
                .is_eq()
            })
            .map(|i| vertices[hull[i]])
            .collect();
        Ok(Self::from_ccw_vertices(strict))
    }

    /// 正規化後の頂点列を読み取り専用で返します。
    ///
    /// # Returns
    /// 辞書順最小から反時計回りの頂点列。共線な中間点と閉じる点の再掲はありません。
    /// # Complexity
    /// O(1)。
    pub fn vertices(&self) -> &[primitives::Point2<i64>] {
        &self.vertices
    }

    /// 点の内部・境界・外部を判定します。
    ///
    /// # Args
    /// - `point`: 判定する点。
    /// # Returns
    /// 空集合は Outside、一点・線分上は Boundary。二次元の内部だけ Inside。
    /// # Complexity
    /// O(log n) 時間、O(1) 追加領域。二頂点以下は O(1)。
    pub fn locate(&self, point: primitives::Point2<i64>) -> predicates::Location {
        let v = &self.vertices;
        if v.is_empty() {
            return predicates::Location::Outside;
        }
        if v.len() < 3 {
            return if predicates::on_segment(point, v[0], v[v.len() - 1]) {
                predicates::Location::Boundary
            } else {
                predicates::Location::Outside
            };
        }
        let first = turn(v[0], v[1], point);
        let last = turn(v[0], v[v.len() - 1], point);
        if first.is_lt() || last.is_gt() {
            return predicates::Location::Outside;
        }
        for (sign, endpoint) in [(first, v[1]), (last, v[v.len() - 1])] {
            if sign.is_eq() {
                return if predicates::on_segment(point, v[0], endpoint) {
                    predicates::Location::Boundary
                } else {
                    predicates::Location::Outside
                };
            }
        }
        // 原点にした頂点から点を挟む扇形を二分探索します。
        let mut lower = 1;
        let mut upper = v.len() - 1;
        while upper - lower > 1 {
            let middle = lower + (upper - lower) / 2;
            if turn(v[0], v[middle], point).is_lt() {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        match turn(v[lower], v[upper], point) {
            cmp::Ordering::Less => predicates::Location::Outside,
            cmp::Ordering::Equal => predicates::Location::Boundary,
            cmp::Ordering::Greater => predicates::Location::Inside,
        }
    }

    /// 指定方向との内積を最大にする頂点または辺を返します。
    ///
    /// # Args
    /// - `direction`: 最大側を求める方向。最小側は逆方向を指定します。
    /// # Returns
    /// vertices 上の支持面。空集合だけ None。線分全体が同値なら Edge { 0, 1 }。
    /// 三頂点以上の Edge は反時計回りに隣接し、末尾から先頭へも折り返します。
    /// # Complexity
    /// O(log n) 時間、O(1) 追加領域。二頂点以下は O(1)。
    pub fn support(&self, direction: direction::Direction) -> Option<SupportFace> {
        let size = self.vertices.len();
        let normal = direction.components();
        match size {
            0 => return None,
            1 => return Some(SupportFace::Vertex(0)),
            2 => {
                let delta = kernel::difference(
                    (self.vertices[0].x, self.vertices[0].y),
                    (self.vertices[1].x, self.vertices[1].y),
                );
                return Some(match kernel::dot_sign(delta, normal) {
                    cmp::Ordering::Less => SupportFace::Vertex(0),
                    cmp::Ordering::Equal => SupportFace::Edge { start: 0, end: 1 },
                    cmp::Ordering::Greater => SupportFace::Vertex(1),
                });
            }
            _ => {}
        }
        // 外向き法線の列は一周だけ昇順です。法線は約分せず符号で比較します。
        let mut lower = 0;
        let mut upper = size;
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            let edge = (self.normal_start + middle) % size;
            if kernel::argument_cmp(self.edge_normal(edge), normal).is_lt() {
                lower = middle + 1;
            } else {
                upper = middle;
            }
        }
        let edge = (self.normal_start + lower) % size;
        Some(
            if kernel::argument_cmp(self.edge_normal(edge), normal).is_eq() {
                SupportFace::Edge {
                    start: edge,
                    end: (edge + 1) % size,
                }
            } else {
                SupportFace::Vertex(edge)
            },
        )
    }

    /// 正規化した頂点列の異なる二頂点の最遠点対を返します。
    ///
    /// # Returns
    /// vertices 上の二添字を小さい順に返します。二頂点未満は None、同率は任意。
    /// 距離の二乗が u128 を超えても、対の選択は正確です。
    /// # Complexity
    /// 回転キャリパーによる O(n) 時間、O(1) 追加領域。
    pub fn furthest_pair(&self) -> Option<(usize, usize)> {
        let size = self.vertices.len();
        if size < 2 {
            return None;
        }
        let mut best = (0, 1);
        let mut distance = self.squared_distance(0, 1);
        if size == 2 {
            return Some(best);
        }
        let mut opposite = 1;
        for edge in 0..size {
            let next = (edge + 1) % size;
            while signed_area(
                self.vertices[edge],
                self.vertices[next],
                self.vertices[(opposite + 1) % size],
            ) > signed_area(
                self.vertices[edge],
                self.vertices[next],
                self.vertices[opposite],
            ) {
                opposite = (opposite + 1) % size;
            }
            self.improve_pair(edge, opposite, &mut best, &mut distance);
            self.improve_pair(next, opposite, &mut best, &mut distance);
            let following = (opposite + 1) % size;
            // 平行な支持辺では両端とも候補になるため、同値側も評価します。
            if signed_area(
                self.vertices[edge],
                self.vertices[next],
                self.vertices[following],
            ) == signed_area(
                self.vertices[edge],
                self.vertices[next],
                self.vertices[opposite],
            ) {
                self.improve_pair(edge, following, &mut best, &mut distance);
                self.improve_pair(next, following, &mut best, &mut distance);
            }
        }
        Some(best)
    }

    /// 面積の二倍を取得します。構築時には面積を計算しません。
    ///
    /// # Returns
    /// 非負の面積の二倍。空・一点・線分では 0。
    /// # Panics
    /// 結果が u128 に収まらない場合。
    /// # Complexity
    /// O(n) 時間、O(1) 追加領域。
    pub fn area2(&self) -> u128 {
        let mut sum = 0_u128;
        for i in 2..self.vertices.len() {
            let area =
                signed_area(self.vertices[0], self.vertices[i - 1], self.vertices[i]).magnitude();
            sum = sum
                .checked_add(area)
                .expect("polygon area does not fit u128");
        }
        sum
    }

    /// 二つの凸図形の Minkowski 和を、辺の偏角順の併合で構築します。
    ///
    /// # Args
    /// - `other`: 和を取る凸図形。点・線分も受け付けます。
    /// # Returns
    /// 正規化した凸図形。空集合との和は空集合です。
    /// # Panics
    /// 生成する頂点の座標が i64 に収まらない場合。
    /// # Complexity
    /// 頂点数 n、m に対して O(n + m) 時間・領域。凸包の再構築はしません。
    pub fn minkowski_sum(&self, other: &Self) -> Self {
        if self.vertices.is_empty() || other.vertices.is_empty() {
            return Self::from_ccw_vertices(Vec::new());
        }
        let first = self.lowest_vertex();
        let second = other.lowest_vertex();
        let mut position = (
            i128::from(self.vertices[first].x) + i128::from(other.vertices[second].x),
            i128::from(self.vertices[first].y) + i128::from(other.vertices[second].y),
        );
        let mut result = vec![checked_point(position)];
        let size_a = if self.vertices.len() == 1 {
            0
        } else {
            self.vertices.len()
        };
        let size_b = if other.vertices.len() == 1 {
            0
        } else {
            other.vertices.len()
        };
        let mut a = 0;
        let mut b = 0;
        while a < size_a || b < size_b {
            let edge_a = if a < size_a {
                self.edge_vector((first + a) % size_a)
            } else {
                (0, 0)
            };
            let edge_b = if b < size_b {
                other.edge_vector((second + b) % size_b)
            } else {
                (0, 0)
            };
            let order = if a == size_a {
                cmp::Ordering::Greater
            } else if b == size_b {
                cmp::Ordering::Less
            } else {
                kernel::ccw_cmp((1, 0), edge_a, edge_b)
            };
            if !order.is_gt() {
                position.0 += edge_a.0;
                position.1 += edge_a.1;
                a += 1;
            }
            if !order.is_lt() {
                position.0 += edge_b.0;
                position.1 += edge_b.1;
                b += 1;
            }
            if a < size_a || b < size_b {
                result.push(checked_point(position));
            }
        }
        // 辺の併合で凸性と共線点の除去は保たれるため、始点だけを揃えます。
        let start = result
            .iter()
            .enumerate()
            .min_by_key(|&(_, p)| (p.x, p.y))
            .unwrap()
            .0;
        result.rotate_left(start);
        Self::from_ccw_vertices(result)
    }

    /// 正規化済みの列から、支持面検索の開始位置だけを構築します。
    ///
    /// # Args
    /// - `vertices`: 辞書順最小から反時計回りの狭義凸な列、または正規化済みの退化図形。
    /// # Returns
    /// 頂点列を所有し、面積を前計算しない多角形。
    /// # Complexity
    /// O(n) 時間、頂点列以外は O(1) 領域。
    fn from_ccw_vertices(vertices: Vec<primitives::Point2<i64>>) -> Self {
        let mut polygon = Self {
            vertices,
            normal_start: 0,
        };
        if polygon.vertices.len() >= 3 {
            for edge in 1..polygon.vertices.len() {
                if kernel::argument_cmp(
                    polygon.edge_normal(edge),
                    polygon.edge_normal(polygon.normal_start),
                )
                .is_lt()
                {
                    polygon.normal_start = edge;
                }
            }
        }
        polygon
    }

    /// 最下点を y、x の順で選び、辺の始角を正の x 軸以降へ揃えます。
    ///
    /// # Returns
    /// 非空の頂点列の最下点の位置。O(n) 時間で取得します。
    fn lowest_vertex(&self) -> usize {
        self.vertices
            .iter()
            .enumerate()
            .min_by_key(|&(_, p)| (p.y, p.x))
            .unwrap()
            .0
    }

    /// 成分の狭い差を避けて、反時計回りの辺を取得します。
    ///
    /// # Args
    /// - `edge`: 非空の頂点列における辺の始点位置。
    /// # Returns
    /// 次の頂点へ向かう二成分。O(1) 時間で取得します。
    fn edge_vector(&self, edge: usize) -> (i128, i128) {
        let from = self.vertices[edge];
        let to = self.vertices[(edge + 1) % self.vertices.len()];
        kernel::difference((from.x, from.y), (to.x, to.y))
    }

    /// 辺を時計回りに 90 度回し、外向き法線を取得します。
    ///
    /// # Args
    /// - `edge`: 辺の始点位置。
    /// # Returns
    /// 約分前の外向き法線。O(1) 時間で取得します。
    fn edge_normal(&self, edge: usize) -> (i128, i128) {
        let (x, y) = self.edge_vector(edge);
        (y, -x)
    }

    /// 二乗距離を、大小の比較に十分な 129 ビットで取得します。
    ///
    /// # Args
    /// - `a`, `b`: 頂点列内の二つの位置。
    /// # Returns
    /// u128 を超える場合も含む非負の二乗距離。O(1) 時間で取得します。
    fn squared_distance(&self, a: usize, b: usize) -> kernel::Wide {
        let a = self.vertices[a];
        let b = self.vertices[b];
        let delta = kernel::difference((a.x, a.y), (b.x, b.y));
        kernel::dot(delta, delta)
    }

    /// 異なる二頂点の候補によって、最遠距離と小さい順の対を更新します。
    ///
    /// # Args
    /// - `a`, `b`: 候補の位置。同じ位置なら更新しません。
    /// - `best`, `distance`: 現在の最遠点対と二乗距離。改善時に両方を更新します。
    /// # Complexity
    /// O(1) 時間。
    fn improve_pair(
        &self,
        a: usize,
        b: usize,
        best: &mut (usize, usize),
        distance: &mut kernel::Wide,
    ) {
        if a == b {
            return;
        }
        let candidate = self.squared_distance(a, b);
        let pair = (a.min(b), a.max(b));
        if candidate > *distance || candidate == *distance && pair < *best {
            *best = pair;
            *distance = candidate;
        }
    }
}

/// 生成する頂点だけを検査し、途中の広い辺ベクトルは狭めません。
///
/// # Args
/// - `x`, `y`: Minkowski 和の頂点の広い座標。
/// # Returns
/// i64 の座標を持つ点。
/// # Panics
/// 座標が i64 に収まらない場合。
/// # Complexity
/// O(1)。
fn checked_point((x, y): (i128, i128)) -> primitives::Point2<i64> {
    primitives::Point2 {
        x: i64::try_from(x).expect("Minkowski sum x does not fit i64"),
        y: i64::try_from(y).expect("Minkowski sum y does not fit i64"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 重複も現れる小さい整数座標の点群です。
    fn create_points(count: usize, seed: u64) -> Vec<primitives::Point2<i64>> {
        let mut state = seed + 1;
        (0..count)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let x = ((state >> 32) % 21) as i64 - 10;
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let y = ((state >> 32) % 21) as i64 - 10;
                primitives::Point2 { x, y }
            })
            .collect()
    }

    /// 小さい座標の参照計算を、内部の広い整数処理を使わずに行います。
    fn reference_area(
        a: primitives::Point2<i64>,
        b: primitives::Point2<i64>,
        c: primitives::Point2<i64>,
    ) -> i128 {
        i128::from(b.x - a.x) * i128::from(c.y - a.y)
            - i128::from(b.y - a.y) * i128::from(c.x - a.x)
    }

    /// Scenario: 支持面の二分探索が全頂点の最大内積と一致します。
    /// - Given: 空・一点・線分も含む複数の凸包と各方向があります。
    /// - When: 支持面を取得します。
    /// - Then: 同値な頂点も含めて、走査した最大値と一致します。
    #[test]
    fn matches_support_faces_against_all_vertices() {
        // Given
        for seed in 0..80 {
            let sut = ConvexPolygon::from_points(&create_points(seed as usize % 17, seed));
            for x in -4..=4 {
                for y in -4..=4 {
                    let Some(normal) =
                        direction::Direction::from_vector(primitives::Vector2 { x, y })
                    else {
                        continue;
                    };
                    let values = sut
                        .vertices()
                        .iter()
                        .map(|p| p.x * x + p.y * y)
                        .collect::<Vec<_>>();
                    let expected = values.iter().max().map(|maximum| {
                        values
                            .iter()
                            .enumerate()
                            .filter_map(|(i, value)| (value == maximum).then_some(i))
                            .collect::<Vec<_>>()
                    });
                    // When
                    let result = sut.support(normal);
                    // Then
                    match (expected, result) {
                        (None, None) => {}
                        (Some(indices), Some(SupportFace::Vertex(index))) => {
                            assert_eq!(vec![index], indices)
                        }
                        (Some(indices), Some(SupportFace::Edge { start, end })) => {
                            let mut result = vec![start, end];
                            result.sort_unstable();
                            assert_eq!(indices, result);
                        }
                        other => panic!("unexpected support face: {other:?}"),
                    }
                }
            }
        }
    }

    /// Scenario: 点の内外判定が全辺の半平面との照合に一致します。
    /// - Given: 二次元の小さい凸包と、周囲を含む格子点があります。
    /// - When: 扇形の二分探索で位置を分類します。
    /// - Then: 全辺を調べた内部・境界・外部と一致します。
    #[test]
    fn matches_location_against_all_edges() {
        // Given
        for seed in 0..35 {
            let sut = ConvexPolygon::from_points(&create_points(12, seed));
            let vertices = sut.vertices();
            assert!(vertices.len() >= 3);
            for x in -12..=12 {
                for y in -12..=12 {
                    let point = primitives::Point2 { x, y };
                    let signs = (0..vertices.len())
                        .map(|i| {
                            reference_area(vertices[i], vertices[(i + 1) % vertices.len()], point)
                        })
                        .collect::<Vec<_>>();
                    let expected = if signs.iter().any(|&value| value < 0) {
                        predicates::Location::Outside
                    } else if signs.contains(&0) {
                        predicates::Location::Boundary
                    } else {
                        predicates::Location::Inside
                    };
                    // When
                    let result = sut.locate(point);
                    // Then
                    assert_eq!(expected, result);
                }
            }
        }
    }

    /// Scenario: 線形時間の和が全点対の和から作る凸包と一致します。
    /// - Given: 空や一点・線分にも退化する二つの点群があります。
    /// - When: 構築済みの図形の辺を併合します。
    /// - Then: 全点対の和の凸包と頂点列が一致し、可換性も満たします。
    #[test]
    fn matches_minkowski_sums_including_degenerate_inputs() {
        // Given
        for seed in 0..120 {
            let first = create_points(seed as usize % 9, seed);
            let second = create_points(seed as usize % 7, seed + 100);
            let sums = first
                .iter()
                .flat_map(|a| {
                    second.iter().map(move |b| primitives::Point2 {
                        x: a.x + b.x,
                        y: a.y + b.y,
                    })
                })
                .collect::<Vec<_>>();
            let expected = ConvexPolygon::from_points(&sums);
            let a = ConvexPolygon::from_points(&first);
            let b = ConvexPolygon::from_points(&second);
            // When
            let result = a.minkowski_sum(&b);
            // Then
            assert_eq!(expected.vertices(), result.vertices());
            assert_eq!(result.vertices(), b.minkowski_sum(&a).vertices());
        }
    }

    /// Scenario: 境界の受理が全頂点の全辺に対する半平面条件と一致します。
    /// - Given: 辺上の共線点を含む五点の全順列があります。
    /// - When: 各順列から凸境界の構築を試みます。
    /// - Then: 一貫した半平面条件を満たす巡回順だけを受理します。
    #[test]
    fn checks_every_permutation_of_a_boundary_with_collinear_points() {
        // Given
        let points =
            [(0, 0), (1, 0), (3, 0), (3, 3), (0, 3)].map(|(x, y)| primitives::Point2 { x, y });
        let mut checked = 0;
        for mut code in 0..5_usize.pow(5) {
            let mut seen = [false; 5];
            let mut boundary = Vec::new();
            for _ in 0..5 {
                let index = code % 5;
                code /= 5;
                if seen[index] {
                    break;
                }
                seen[index] = true;
                boundary.push(points[index]);
            }
            if boundary.len() != 5 {
                continue;
            }
            let values = (0..5)
                .flat_map(|i| {
                    let boundary = &boundary;
                    points
                        .iter()
                        .map(move |&p| reference_area(boundary[i], boundary[(i + 1) % 5], p))
                })
                .collect::<Vec<_>>();
            let expected =
                values.iter().all(|&value| value >= 0) || values.iter().all(|&value| value <= 0);
            // When
            let result = ConvexPolygon::try_from_boundary(&boundary);
            // Then
            assert_eq!(expected, result.is_ok());
            checked += 1;
        }
        assert_eq!(120, checked);
    }
}

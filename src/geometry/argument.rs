//! 偏角比較、元添字を保持する方向列、円周上の範囲検索
//!
//! 正の x 軸を角度 0 として、反時計回りに測る角度を偏角と呼びます。
//! ここでは (-π, π] の範囲で比べるため、軸上では南・東・北・西の順になります。
//! [`argument_cmp`] はゼロベクトルを東向きと同じ角度 0 に置きます。
//! [`PolarOrder`] は方向のない要素を分離し、元の入力の添字を保持します。
//!
//! 次の例では、東を含み西を除く反時計回りの半周から、東と北を選びます。
//! 選んだ方向の元添字を取り出し、同じ範囲を累積和に渡して重みの合計も求めます。
//!
//! ```
//! use anmitsu::geometry::{argument, direction, primitives};
//! let vectors = [(1, 0), (0, 1), (-1, 0), (0, -1)];
//! let order = argument::PolarOrder::new(vectors.map(|(x, y)|
//!     direction::Direction::from_vector(primitives::Vector2 { x, y })));
//! let east = direction::Direction::from_vector(primitives::Vector2 { x: 1, y: 0 }).unwrap();
//! assert_eq!(2, order.ccw_range(east..east.opposite()).len());
//! let ids = order.ccw_range(east..east.opposite()).split().into_iter()
//!     .flat_map(|range| range.map(|position| order.indices()[position])).collect::<Vec<_>>();
//! assert_eq!(vec![0, 1], ids);
//! let weights = [2_i64, 3, 5, 7];
//! let mut prefix = vec![0_i64];
//! for &index in order.indices() {
//!     prefix.push(prefix.last().unwrap() + weights[index]);
//! }
//! let sum = order.ccw_range(east..east.opposite()).split().iter()
//!     .map(|range| prefix[range.end] - prefix[range.start]).sum::<i64>();
//! assert_eq!(5, sum);
//! ```

use super::{direction, kernel, primitives};
use std::{cmp, ops};

/// ベクトル `a` と `b` の偏角を比べ、角度の小さい方を先に並べます。
///
/// 正の x 軸を角度 0 とし、反時計回りに測る角度を (-π, π] の範囲で比べます。
/// 軸上のベクトルなら南・東・北・西の順です。`sort_by` に渡して偏角順に並べられます。
///
/// # Args
///
/// - `a`, `b`: 順番を比較する二つのベクトル。ゼロも受け付けます。
///
/// # Returns
///
/// `a` の角度が小さければ `Less`、大きければ `Greater`、同じなら `Equal`。
/// 長さは比べないので、正の整数倍のベクトルは `Equal` です。
/// ゼロベクトルは角度 0 とみなし、東向きのベクトルとも `Equal` になります。
///
/// # Complexity
///
/// 時間・追加領域とも O(1)。
///
/// # Examples
///
/// ```
/// use anmitsu::geometry::{argument, primitives};
/// let mut vectors = [(0, 1), (-1, 0), (0, -1), (1, 0)]
///     .map(|(x, y)| primitives::Vector2 { x, y });
/// vectors.sort_by(argument::argument_cmp);
/// assert_eq!([(0, -1), (1, 0), (0, 1), (-1, 0)], vectors.map(|v| (v.x, v.y)));
/// ```
pub fn argument_cmp(a: &primitives::Vector2<i64>, b: &primitives::Vector2<i64>) -> cmp::Ordering {
    kernel::argument_cmp(
        (i128::from(a.x), i128::from(a.y)),
        (i128::from(b.x), i128::from(b.y)),
    )
}

/// 指定した方向から反時計回りに並べるために、二つの方向を比較します。
///
/// `start` から反時計回りに回したとき、`a` と `b` のどちらへ先に到達するかを比べます。
/// `start` と同じ方向は回転せずに到達するものとして先頭に置きます。
/// 例えば `start` が北向きなら、北・西・南・東の順になります。
///
/// # Args
///
/// - `start`: 並び順の先頭とする基準の方向。
/// - `a`, `b`: 順番を比べる二つの方向。
///
/// # Returns
///
/// `a` へ先に到達すれば `Less`、`b` へ先に到達すれば `Greater`、同じ方向なら `Equal`。
///
/// # Complexity
///
/// 時間・追加領域とも O(1)。
///
/// # Examples
///
/// ```
/// use anmitsu::geometry::{argument, direction, primitives};
/// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y }).unwrap();
/// let north = ray(0, 1);
/// let west = ray(-1, 0);
/// let south = ray(0, -1);
/// let east = ray(1, 0);
/// let mut directions = [east, south, west, north];
/// directions.sort_by(|&a, &b| argument::cmp_ccw_from(north, a, b));
/// assert_eq!([north, west, south, east], directions);
/// ```
pub fn cmp_ccw_from(
    start: direction::Direction,
    a: direction::Direction,
    b: direction::Direction,
) -> cmp::Ordering {
    kernel::ccw_cmp(start.components(), a.components(), b.components())
}

/// 方向を偏角順に並べ、並べ替える前の入力との対応を保持します。
///
/// 正の x 軸を 0 とする (-π, π] の角度の昇順で並べます。
/// 渡した列の 0 始まりの位置を「元添字」、並べた後の位置を「整列位置」と呼びます。
/// 同じ方向の要素も統合せずに残し、その中では元添字の小さい順に並べます。
///
/// 整列位置 `p` の方向は `directions()[p]`、その方向を持つ入力要素の元添字は
/// `indices()[p]` です。`None` として渡した要素はこの二つの列から除き、
/// 元添字を [`zero_indices`](Self::zero_indices) に入力順で保持します。
/// 構築後に列を変更する操作はありません。
#[derive(Clone, Debug)]
pub struct PolarOrder {
    /// 絶対偏角順の非ゼロ方向。
    directions: Vec<direction::Direction>,
    /// 方向列と同じ位置順の元添字。
    indices: Vec<usize>,
    /// 方向がない元添字。入力順です。
    zero_indices: Vec<usize>,
}

/// 固定された方向列上の位置範囲です。元添字の範囲とは異なります。
///
/// 問い合わせ元の `PolarOrder` のスライスに適用してください。
/// 空でも始位置を保ち、補集合を二回取ると元の巡回順へ戻ります。
#[derive(Clone, Copy, Debug)]
pub struct CyclicRange {
    /// 対象となる列の長さ。
    size: usize,
    /// 結果の開始位置。空列では 0。
    start: usize,
    /// 選択する要素数。列の長さ以下。
    count: usize,
}

impl PolarOrder {
    /// 入力の方向を偏角順に並べ、元添字を保持する列を構築します。
    ///
    /// # Args
    ///
    /// - `directions`: 入力順の方向。列挙した位置 0, 1, ... を元添字として採番します。
    ///   `None` はゼロベクトルや基準点と同一点であるなど、方向がない要素を表します。
    ///
    /// # Returns
    ///
    /// `Some` の方向を (-π, π] の偏角の昇順に並べた列。
    /// 同方向の中では元添字の順を保ち、`None` の元添字は `zero_indices()` に分けます。
    ///
    /// # Complexity
    ///
    /// 入力の要素数 n に対して O(n log n) 時間、O(n) 領域。方向の構築時間は含みません。
    ///
    /// # Examples
    ///
    /// 入力を北・東・ゼロ・東の順に渡すと、元添字 1・3・0 の順に東・東・北が並びます。
    ///
    /// ```
    /// use anmitsu::geometry::{argument, direction, primitives};
    /// let vectors = [(0, 1), (1, 0), (0, 0), (2, 0)];
    /// let order = argument::PolarOrder::new(vectors.map(|(x, y)|
    ///     direction::Direction::from_vector(primitives::Vector2 { x, y })));
    /// assert_eq!([1, 3, 0], order.indices());
    /// assert_eq!([2], order.zero_indices());
    /// let north = direction::Direction::from_vector(primitives::Vector2 { x: 0, y: 1 }).unwrap();
    /// assert_eq!(north, order.directions()[2]);
    /// assert_eq!(0, order.indices()[2]);
    /// ```
    pub fn new(directions: impl IntoIterator<Item = Option<direction::Direction>>) -> Self {
        let mut entries = Vec::new();
        let mut zero_indices = Vec::new();
        for (index, ray) in directions.into_iter().enumerate() {
            if let Some(ray) = ray {
                entries.push((ray, index));
            } else {
                zero_indices.push(index);
            }
        }
        // 元添字を第二キーにし、同方向の順を固定します。
        entries.sort_unstable();
        let (directions, indices) = entries.into_iter().unzip();
        Self {
            directions,
            indices,
            zero_indices,
        }
    }

    /// 偏角順に並べた方向が、元の入力のどの要素に対応するかを返します。
    ///
    /// # Returns
    ///
    /// 整列位置 `p` にある方向 `directions()[p]` の元添字が `indices()[p]` です。
    /// 入力の非ゼロ方向の各要素が一度ずつ現れ、`None` の要素は含みません。
    /// 並べ替えの具体例は [`new`](Self::new) を参照してください。
    ///
    /// # Complexity
    ///
    /// O(1)。
    #[inline]
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// 方向がない要素の元添字を入力順に返します。
    ///
    /// # Returns
    /// None として渡された要素の元添字。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub fn zero_indices(&self) -> &[usize] {
        &self.zero_indices
    }

    /// 偏角順に並べた方向を、読み取り専用のスライスで返します。
    ///
    /// # Returns
    ///
    /// (-π, π] の角度の昇順の方向列。同じ方向の重複も残します。
    /// `directions()[p]` は、元の入力の `indices()[p]` 番目に渡した方向です。
    ///
    /// # Complexity
    ///
    /// O(1)。
    #[inline]
    pub fn directions(&self) -> &[direction::Direction] {
        &self.directions
    }

    /// 指定方向から時計回り側・反時計回り側で最も近い要素の位置を返します。
    ///
    /// 指定方向と一致する要素があれば、その同方向のまとまりの最後と最初を選びます。
    /// 一致する要素がなければ、指定方向を挿入するときの前後の要素を選びます。
    /// 並びの末尾と先頭は円周上でつながっているものとして折り返します。
    ///
    /// # Args
    ///
    /// - `direction`: 両側の要素を探す基準の方向。
    ///
    /// # Returns
    ///
    /// `Some([時計回り側の整列位置, 反時計回り側の整列位置])`。
    /// 返る値は元添字ではなく、`directions()` と `indices()` を参照する位置です。
    /// 元の入力の添字は `indices()[位置]` で取得できます。
    /// 二候補の方向が同じ場合もあり、一要素しかなければ同じ位置を二回返します。
    /// 非ゼロ方向がない列だけ `None` になります。
    ///
    /// # Complexity
    ///
    /// 非ゼロ方向の要素数 n に対して O(log n) 時間、O(1) 追加領域。
    ///
    /// # Examples
    ///
    /// ```
    /// use anmitsu::geometry::{argument, direction, primitives};
    /// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y }).unwrap();
    /// let east = ray(1, 0);
    /// let north = ray(0, 1);
    /// let order = argument::PolarOrder::new([Some(north), Some(east), None, Some(east)]);
    /// assert_eq!([1, 3, 0], order.indices());
    /// // 東に一致する二要素の、最後と最初の整列位置を返します。
    /// let [clockwise, counterclockwise] = order.bracketing_positions(east).unwrap();
    /// assert_eq!([1, 0], [clockwise, counterclockwise]);
    /// assert_eq!([3, 1], [order.indices()[clockwise], order.indices()[counterclockwise]]);
    /// // 北東は東と北の間、西は末尾の北と先頭の東の間にあります。
    /// assert_eq!(Some([1, 2]), order.bracketing_positions(ray(1, 1)));
    /// assert_eq!(Some([2, 0]), order.bracketing_positions(ray(-1, 0)));
    /// ```
    #[inline]
    pub fn bracketing_positions(&self, direction: direction::Direction) -> Option<[usize; 2]> {
        let size = self.directions.len();
        if size == 0 {
            return None;
        }
        let lower = self.directions.partition_point(|&ray| ray < direction);
        // 一致しない問い合わせでは、同じ挿入位置を二度探索しません。
        let upper = if lower < size && self.directions[lower] == direction {
            lower + self.directions[lower..].partition_point(|&ray| ray == direction)
        } else {
            lower
        };
        Some([
            if upper == 0 { size - 1 } else { upper - 1 },
            if lower == size { 0 } else { lower },
        ])
    }

    /// 始方向から終方向まで反時計回りに進んだときに含まれる要素を選びます。
    ///
    /// 異なる端点では、`a..b` は a と同じ方向を含み、b と同じ方向を除きます。
    /// `a..=b` は両方を含みます。端点と同じ方向の要素が複数あれば、まとめて含めるか除きます。
    /// 開始端を除くなどの指定には `Bound::Excluded`・`Bound::Included` の組も使えます。
    ///
    /// `a > b` が [`Direction`](direction::Direction) の (-π, π] の順序で成り立つときは、
    /// 整列列の末尾から先頭へ折り返して b まで進みます。
    ///
    /// # Args
    ///
    /// - `range`: 選ぶ範囲の方向と、各端点を含めるかどうか。
    ///   `..b`・`..=b` は絶対偏角順の先頭から、`a..` は末尾までを選び、折り返しません。
    ///   `..` は非ゼロ方向の全要素です。
    ///
    /// # Returns
    ///
    /// 選択した要素の整列位置を表す [`CyclicRange`]。
    /// 始方向からの走査順を保ち、全要素を選んでも絶対偏角の先頭へ戻しません。
    /// 両端が同じ場合は `d..=d` だけが d と同方向の全要素を含み、ほかの端点指定は空です。
    /// 一周全体を選ぶには `..` を使います。
    ///
    /// # Complexity
    ///
    /// 非ゼロ方向の要素数 n に対して O(log n) 時間、O(1) 追加領域。
    ///
    /// # Examples
    ///
    /// 北から東へ反時計回りに進み、東を除くと、北・西・南が選ばれます。
    /// 返る範囲は整列位置なので、元添字の取得には `indices()` を参照します。
    ///
    /// ```
    /// use anmitsu::geometry::{argument, direction, primitives};
    /// let ray = |x, y| direction::Direction::from_vector(primitives::Vector2 { x, y }).unwrap();
    /// let east = ray(1, 0);
    /// let north = ray(0, 1);
    /// let order = argument::PolarOrder::new([east, north, ray(-1, 0), ray(0, -1)].map(Some));
    /// assert_eq!([3, 0, 1, 2], order.indices());
    /// let selected = order.ccw_range(north..east);
    /// assert_eq!([2..4, 0..1], selected.split());
    /// let mut original_indices = Vec::new();
    /// for positions in selected.split() {
    ///     original_indices.extend_from_slice(&order.indices()[positions]);
    /// }
    /// assert_eq!(vec![1, 2, 3], original_indices);
    /// assert_eq!(4, order.ccw_range(north..=east).len());
    /// // 終端を省略すると折り返さず、北・西だけを選びます。
    /// assert_eq!(2, order.ccw_range(north..).len());
    /// assert!(order.ccw_range(north..north).is_empty());
    /// assert_eq!(1, order.ccw_range(north..=north).len());
    /// assert_eq!(4, order.ccw_range(..).len());
    /// ```
    pub fn ccw_range(&self, range: impl ops::RangeBounds<direction::Direction>) -> CyclicRange {
        let size = self.directions.len();
        let start = match range.start_bound() {
            ops::Bound::Unbounded => 0,
            ops::Bound::Included(&ray) => self.directions.partition_point(|&d| d < ray),
            ops::Bound::Excluded(&ray) => self.directions.partition_point(|&d| d <= ray),
        };
        let end = match range.end_bound() {
            ops::Bound::Unbounded => size,
            ops::Bound::Included(&ray) => self.directions.partition_point(|&d| d <= ray),
            ops::Bound::Excluded(&ray) => self.directions.partition_point(|&d| d < ray),
        };
        let count = match (range.start_bound(), range.end_bound()) {
            (
                ops::Bound::Included(a) | ops::Bound::Excluded(a),
                ops::Bound::Included(b) | ops::Bound::Excluded(b),
            ) if a == b => {
                if matches!(
                    (range.start_bound(), range.end_bound()),
                    (ops::Bound::Included(_), ops::Bound::Included(_))
                ) {
                    end - start
                } else {
                    0
                }
            }
            (
                ops::Bound::Included(a) | ops::Bound::Excluded(a),
                ops::Bound::Included(b) | ops::Bound::Excluded(b),
            ) if a > b => size - start + end,
            _ => end.saturating_sub(start),
        };
        CyclicRange {
            size,
            start: if start == size { 0 } else { start },
            count,
        }
    }
}

impl CyclicRange {
    /// 選択した要素数を返します。
    ///
    /// # Returns
    /// 元の列の長さ以下の個数。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// 選択した要素がないかを返します。
    ///
    /// # Returns
    /// 個数が 0 の場合に真。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// 選んだ要素の整列位置を、高々二つの半開区間に分けて返します。
    ///
    /// 返る区間は、問い合わせ元の `PolarOrder` の `directions()` または `indices()` を
    /// 参照する位置です。元の入力を直接スライスする区間ではありません。
    /// `indices()[区間]` を取り出せば、その部分に含まれる元添字を取得できます。
    ///
    /// # Returns
    ///
    /// 始方向から反時計回りに進む走査順の二つの範囲。
    /// 末尾から先頭へ折り返す場合は、末尾までの範囲が先、先頭からの範囲が後です。
    /// 不要な範囲は `0..0`、空の選択では二つとも `0..0` になります。
    /// 具体例は [`PolarOrder::ccw_range`] を参照してください。
    ///
    /// # Complexity
    ///
    /// O(1)。
    #[inline]
    pub fn split(&self) -> [ops::Range<usize>; 2] {
        if self.count == 0 {
            return [0..0, 0..0];
        }
        let first = self.count.min(self.size - self.start);
        [self.start..self.start + first, 0..self.count - first]
    }

    /// 選択範囲の直後から進む補集合を返します。
    ///
    /// # Returns
    /// 同じ列に対する補集合。空列では空のままです。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub fn complement(&self) -> Self {
        let tail = self.size - self.start;
        let start = if self.count >= tail {
            self.count - tail
        } else {
            self.start + self.count
        };
        Self {
            size: self.size,
            start,
            count: self.size - self.count,
        }
    }
}

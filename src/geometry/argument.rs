//! 偏角比較、元添字を保持する方向列、円周上の範囲検索
//!
//! 絶対偏角は (-π, π] の反時計回り順です。原点を受け付ける比較関数では、
//! 原点だけを角度 0 とみなします。方向列ではゼロの添字を別に保持します。
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

/// 原点を角度 0 とする、ベクトルの偏角だけの比較です。
///
/// # Args
/// - `a`, `b`: 比較するベクトル。ゼロも受け付けます。
/// # Returns
/// (-π, π] の順。同方向の長さは比較しません。
/// # Complexity
/// 時間・追加領域とも O(1)。
pub fn argument_cmp(a: &primitives::Vector2<i64>, b: &primitives::Vector2<i64>) -> cmp::Ordering {
    kernel::argument_cmp(
        (i128::from(a.x), i128::from(a.y)),
        (i128::from(b.x), i128::from(b.y)),
    )
}

/// 任意の始線と同方向を先頭にして、反時計回りの順を比較します。
///
/// # Args
/// - `start`: 先頭とする方向。
/// - `a`, `b`: 比較する非ゼロ方向。
/// # Returns
/// 一周分の方向順。同方向は等しいとします。
/// # Complexity
/// 時間・追加領域とも O(1)。
pub fn cmp_ccw_from(
    start: direction::Direction,
    a: direction::Direction,
    b: direction::Direction,
) -> cmp::Ordering {
    kernel::ccw_cmp(start.components(), a.components(), b.components())
}

/// 非ゼロ方向の整列列と、元の要素への対応です。構築後は変更しません。
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
    /// ゼロの元添字を分離し、非ゼロ方向を整列します。
    ///
    /// # Args
    /// - `directions`: 入力順の方向。None はゼロや基準点との一致を表します。
    /// # Returns
    /// 同方向では元添字順を保つ方向列。
    /// # Complexity
    /// n 要素に対して O(n log n) 時間、O(n) 領域。方向の構築時間は含みません。
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

    /// 非ゼロ方向と同じ整列位置の元添字を返します。
    ///
    /// # Returns
    /// 入力の各非ゼロ要素が一度ずつ現れるスライス。
    /// # Complexity
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

    /// `indices` と同じ位置順の、読み取り専用の方向列を返します。
    ///
    /// # Returns
    /// 同方向の重複を保ったスライス。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub fn directions(&self) -> &[direction::Direction] {
        &self.directions
    }

    /// 指定方向も含め、時計回り側と反時計回り側の前後候補を返します。
    ///
    /// # Args
    /// - `direction`: 挟む方向。
    /// # Returns
    /// 整列位置の `[時計回り側, 反時計回り側]`。各同方向群の末尾と先頭を
    /// 選びます。一要素だけなら同じ位置が二回現れ、空列だけ None です。
    /// # Complexity
    /// O(log n) 時間、O(1) 追加領域。
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

    /// 反時計回りの角度区間を、整列位置の範囲として取得します。
    ///
    /// # Args
    /// - `range`: 方向を端点とする範囲。始方向が終方向より後なら折り返します。
    ///   片端省略時は絶対偏角順の端まで、`..` は全体です。
    /// # Returns
    /// 同じ両端では `d..=d` だけが同方向の要素を含み、ほかは空です。
    /// 始方向からの走査順は、全要素を含む結果でも保ちます。
    /// # Complexity
    /// O(log n) 時間、O(1) 追加領域。
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

    /// 巡回順を保つ高々二つの通常の範囲へ分割します。
    ///
    /// # Returns
    /// 非空なら最初の範囲は非空で、不要な側は `0..0` です。
    /// # Complexity
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

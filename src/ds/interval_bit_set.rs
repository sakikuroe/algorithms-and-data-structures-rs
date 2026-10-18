//! `usize` の全範囲の 0/1 列を、値が連続する区間で管理します。
//!
//! `true` の最大の連続区間だけを保持し、それ以外を `false` とします。
//! 範囲の端の省略は `0` または `usize::MAX` を意味します。
//! 空範囲と逆転した範囲は、いずれも空として扱います。

use std::{collections, iter, ops};

/// `0..=usize::MAX` の各位置に真偽値を保持する、区間で圧縮したビット列です。
///
/// 初期値はすべて `false` です。隣接する同じ値の区間は結合します。
/// 保持する `true` の区間数を `K` とすると、記憶領域は `O(K)` です。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IntervalBitSet {
    /// 始点から終点への対応です。閉区間は重ならず、互いに隣接しません。
    ones: collections::BTreeMap<usize, usize>,
    /// 全範囲の集計で走査しないよう、`true` の総数を保持します。
    count_ones: u128,
}

impl IntervalBitSet {
    /// 初期値がすべて `false` のビット列を作ります。
    ///
    /// # Returns
    /// 区間をまだ登録していないビット列を返します。
    ///
    /// # Complexity
    /// 時間・空間計算量は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let bits = interval_bit_set::IntervalBitSet::new();
    /// assert!(!bits.get(0));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定範囲の値を設定し、値が変わった位置の個数を返します。
    ///
    /// # Args
    /// - `range`: 更新範囲です。包含・排他・省略した境界に対応します。
    /// - `value`: 設定する値です。
    ///
    /// # Returns
    /// 更新前の値が `value` と異なっていた位置の個数を返します。
    /// 空範囲と逆転した範囲では `0` を返し、状態を変更しません。
    ///
    /// # Complexity
    /// 触れる区間数を `A` とすると、時間は `O((A + 1) log(K + 2))`、
    /// 一時的な追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// assert_eq!(6, bits.set(2..8, true));
    /// assert_eq!(2, bits.set(4..6, false));
    /// assert_eq!(0, bits.set(2..4, true));
    /// ```
    pub fn set<R: ops::RangeBounds<usize>>(&mut self, range: R, value: bool) -> u128 {
        // 閉区間へ正規化するため、最大値を超える排他的終端は不要です。
        let Some((mut start, mut end)) = normalize(range) else {
            return 0;
        };

        if value {
            // 左で接する区間も結合し、最大の連続区間という不変条件を守ります。
            if let Some((&left, &right)) = self.ones.range(..=start).next_back()
                && right.saturating_add(1) >= start
            {
                start = left;
            }

            // 結合前の個数を差し引けば、既存の 1 を二重に数えません。
            let mut previous = 0;
            while let Some((&left, &right)) = self.ones.range(start..).next() {
                if left > end.saturating_add(1) {
                    break;
                }
                self.ones.remove(&left);
                previous += width(left, right);
                end = end.max(right);
            }
            self.ones.insert(start, end);
            let changed = width(start, end) - previous;
            self.count_ones += changed;
            changed
        } else {
            // 始点をまたぐ区間があれば、そこから削除対象を調べます。
            let mut lower = self
                .ones
                .range(..=start)
                .next_back()
                .filter(|&(_, &right)| right >= start)
                .map_or(start, |(&left, _)| left);
            let mut changed = 0;
            while let Some((&left, &right)) = self.ones.range(lower..=end).next() {
                self.ones.remove(&left);
                changed += width(left.max(start), right.min(end));

                // 更新範囲の外に残る部分だけを戻し、内側を 0 にします。
                if left < start {
                    self.ones.insert(left, start - 1);
                }
                if right > end {
                    self.ones.insert(end + 1, right);
                    break;
                }
                // 左の残存区間を再び選ばないよう、検索下限を更新します。
                lower = start;
            }
            self.count_ones -= changed;
            changed
        }
    }

    /// 指定した位置の値を返します。
    ///
    /// # Args
    /// - `x`: 値を取得する添字です。
    ///
    /// # Returns
    /// `x` に設定されている真偽値を返します。
    ///
    /// # Complexity
    /// 時間は `O(log(K + 1))`、追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..5, true);
    /// assert!(bits.get(4));
    /// assert!(!bits.get(5));
    /// ```
    pub fn get(&self, x: usize) -> bool {
        // 直前の始点を持つ区間だけが、この位置を覆う可能性があります。
        self.ones
            .range(..=x)
            .next_back()
            .is_some_and(|(_, &end)| x <= end)
    }

    /// 指定した位置を含む、同じ値の最大の連続区間を返します。
    ///
    /// # Args
    /// - `x`: 区間を取得する添字です。
    ///
    /// # Returns
    /// 値が `true` でも `false` でも、その位置を含む閉区間を返します。
    ///
    /// # Complexity
    /// 時間は `O(log(K + 1))`、追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// assert_eq!(2..=7, bits.run_at(4));
    /// assert_eq!(0..=1, bits.run_at(0));
    /// assert_eq!(8..=usize::MAX, bits.run_at(9));
    /// ```
    pub fn run_at(&self, x: usize) -> ops::RangeInclusive<usize> {
        // 1 の区間なら、その両端が求める境界です。
        let previous = self.ones.range(..=x).next_back();
        if let Some((&start, &end)) = previous
            && x <= end
        {
            return start..=end;
        }

        // 0 の区間では、左右の 1 の区間の直後と直前を境界にします。
        let start = previous.map_or(0, |(_, &end)| end + 1);
        let end = self
            .ones
            .range(x..)
            .next()
            .map_or(usize::MAX, |(&start, _)| start - 1);
        start..=end
    }

    /// 指定範囲内で、指定値を持つ最小の添字を探します。
    ///
    /// # Args
    /// - `range`: 検索する範囲です。
    /// - `value`: 検索する値です。
    ///
    /// # Returns
    /// 該当位置があれば `Some`、なければ `None` を返します。
    /// `find(x.., false)` は `x` 以上の mex に相当します。
    ///
    /// # Complexity
    /// 時間は `O(log(K + 1))`、追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// assert_eq!(Some(8), bits.find(4.., false));
    /// assert_eq!(None, bits.find(2..8, false));
    /// ```
    pub fn find<R: ops::RangeBounds<usize>>(&self, range: R, value: bool) -> Option<usize> {
        self.ranges(range, value).next().map(|run| *run.start())
    }

    /// 指定範囲内で、指定値を持つ位置の個数を返します。
    ///
    /// # Args
    /// - `range`: 集計する範囲です。
    /// - `value`: 数える値です。
    ///
    /// # Returns
    /// 該当個数を `u128` で返します。全範囲の個数も正確に表せます。
    ///
    /// # Complexity
    /// 全範囲では時間は `O(1)` です。それ以外は、触れる 1 の区間数を
    /// `A` として `O(log(K + 1) + A)` です。追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// assert_eq!(6, bits.count(.., true));
    /// assert_eq!(4, bits.count(0..10, false));
    /// ```
    pub fn count<R: ops::RangeBounds<usize>>(&self, range: R, value: bool) -> u128 {
        let Some((start, end)) = normalize(range) else {
            return 0;
        };
        // 全体の 1 の総数は更新時に維持しているため、走査を省略できます。
        let ones = if start == 0 && end == usize::MAX {
            self.count_ones
        } else {
            self.ranges(start..=end, true)
                .map(|run| width(*run.start(), *run.end()))
                .sum()
        };
        if value {
            ones
        } else {
            width(start, end) - ones
        }
    }

    /// 指定範囲内で指定値が連続する区間を、昇順に列挙します。
    ///
    /// # Args
    /// - `range`: 列挙する範囲です。返す各区間もこの範囲で切り取ります。
    /// - `value`: 列挙する値です。
    ///
    /// # Returns
    /// 閉区間を返す双方向 iterator です。空範囲では何も返しません。
    /// 全体は `ranges(.., value)`、補集合は `ranges(.., false)` で列挙できます。
    ///
    /// # Complexity
    /// 構築は `O(log(K + 1))`、`A` 区間の列挙は償却 `O(A + 1)`、
    /// 追加空間は `O(1)` です。列挙する区間を事前に確保しません。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// bits.set(10..15, true);
    /// assert_eq!(vec![4..=7, 10..=11],
    ///     bits.ranges(4..12, true).collect::<Vec<_>>());
    /// assert_eq!(vec![8..=9],
    ///     bits.ranges(4..12, false).rev().collect::<Vec<_>>());
    /// ```
    pub fn ranges<R: ops::RangeBounds<usize>>(
        &self,
        range: R,
        value: bool,
    ) -> impl DoubleEndedIterator<Item = ops::RangeInclusive<usize>> + '_ {
        let remaining = normalize(range);
        // 始点をまたぐ 1 の区間も列挙対象に含めます。
        let runs = if let Some((start, end)) = remaining {
            let lower = self
                .ones
                .range(..=start)
                .next_back()
                .filter(|&(_, &right)| right >= start)
                .map_or(start, |(&left, _)| left);
            self.ones.range(lower..=end)
        } else {
            self.ones.range(0..0)
        };
        Ranges {
            runs,
            remaining,
            value,
        }
    }
}

/// 包含・排他・無制限の境界を閉区間に直し、空範囲を除外します。
///
/// # Args
/// - `range`: 正規化する範囲です。
///
/// # Returns
/// 非空なら両端、空または逆転した範囲なら `None` を返します。
fn normalize<R: ops::RangeBounds<usize>>(range: R) -> Option<(usize, usize)> {
    // 排他的境界が値域の外を指す場合、加減算を飽和させず空と判定します。
    let start = match range.start_bound() {
        ops::Bound::Included(&x) => x,
        ops::Bound::Excluded(&x) => x.checked_add(1)?,
        ops::Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        ops::Bound::Included(&x) => x,
        ops::Bound::Excluded(&x) => x.checked_sub(1)?,
        ops::Bound::Unbounded => usize::MAX,
    };
    (start <= end).then_some((start, end))
}

/// 閉区間の個数を、全値域でも桁あふれしない型で計算します。
///
/// # Args
/// - `start`: 区間の始点です。
/// - `end`: `start` 以上の終点です。
///
/// # Returns
/// 両端を含めた位置の個数を返します。
fn width(start: usize, end: usize) -> u128 {
    end as u128 - start as u128 + 1
}

/// 1 の区間を一度ずつ走査し、必要に応じて区間の隙間を生成します。
struct Ranges<'a> {
    /// 両端からまだ取り出していない 1 の区間です。
    runs: collections::btree_map::Range<'a, usize, usize>,
    /// 両端からまだ返していない添字範囲です。
    remaining: Option<(usize, usize)>,
    /// 列挙する値です。
    value: bool,
}

impl Iterator for Ranges<'_> {
    type Item = ops::RangeInclusive<usize>;

    /// 左端から次の一致区間を取り出し、未処理範囲を進めます。
    fn next(&mut self) -> Option<Self::Item> {
        let (mut start, end) = self.remaining?;
        for (&left, &right) in self.runs.by_ref() {
            if self.value {
                let last = right.min(end);
                self.remaining = last.checked_add(1).filter(|&x| x <= end).map(|x| (x, end));
                return Some(left.max(start)..=last);
            }

            // 0 を列挙するときは、消費した 1 の区間の直後へ進みます。
            self.remaining = right.checked_add(1).filter(|&x| x <= end).map(|x| (x, end));
            if start < left {
                return Some(start..=left - 1);
            }
            start = self.remaining?.0;
        }

        // 末尾に残る 0 の区間は、境界を含めて一度だけ返します。
        self.remaining = None;
        (!self.value).then_some(start..=end)
    }
}

impl DoubleEndedIterator for Ranges<'_> {
    /// 右端から次の一致区間を取り出し、未処理範囲を狭めます。
    fn next_back(&mut self) -> Option<Self::Item> {
        let (start, mut end) = self.remaining?;
        while let Some((&left, &right)) = self.runs.next_back() {
            if self.value {
                let first = left.max(start);
                self.remaining = first
                    .checked_sub(1)
                    .filter(|&x| x >= start)
                    .map(|x| (start, x));
                return Some(first..=right.min(end));
            }

            // 左からの場合と対称に、消費した 1 の区間の直前へ進みます。
            self.remaining = left
                .checked_sub(1)
                .filter(|&x| x >= start)
                .map(|x| (start, x));
            if right < end {
                return Some(right + 1..=end);
            }
            end = self.remaining?.1;
        }

        // 両側からの取得が交差した後は、何も返しません。
        self.remaining = None;
        (!self.value).then_some(start..=end)
    }
}

impl iter::FusedIterator for Ranges<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 2 個の 1 の区間を持ち、その間と両端が 0 の列を作ります。
    fn create_bits() -> IntervalBitSet {
        let mut sut = IntervalBitSet::new();
        sut.set(2..8, true);
        sut.set(10..15, true);
        sut
    }

    // set のテスト: 結合、分割、全値域の更新と境界を検証します。
    mod set {
        use super::*;

        /// Scenario: 隣接区間を結合し、部分的な 0 の設定で分割します。
        /// - Given: 離れた 2 個の 1 の区間があります。
        /// - When: 間を 1 にしてから、内部の一部を 0 に戻します。
        /// - Then: 変更個数と更新後の最大の連続区間が一致します。
        #[test]
        fn merges_adjacent_runs_and_splits_on_clear() {
            // Given
            let mut sut = create_bits();
            // When
            let inserted = sut.set(8..10, true);
            let merged = sut.run_at(5);
            let removed = sut.set(6..=11, false);
            // Then
            assert_eq!(2, inserted);
            assert_eq!(2..=14, merged);
            assert_eq!(6, removed);
            assert_eq!(
                vec![2..=5, 12..=14],
                sut.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(7, sut.count(.., true));
        }

        /// Scenario: 全範囲とその両端の更新を桁あふれせず扱います。
        /// - Given: 初期値がすべて 0 の列があります。
        /// - When: 全範囲を 1 にして両端を 0 に戻し、再び全範囲を 0 にします。
        /// - Then: 全値域の個数、端の区間、mex が正しく求まります。
        #[test]
        fn updates_the_entire_domain_and_both_extremes() {
            // Given
            let mut sut = IntervalBitSet::new();
            let total = usize::MAX as u128 + 1;
            // When
            let filled = sut.set(.., true);
            let full_count = sut.count(.., true);
            let full_mex = sut.find(.., false);
            let cleared_first = sut.set(..=0, false);
            let cleared_last = sut.set(usize::MAX.., false);
            let gaps = sut.ranges(.., false).collect::<Vec<_>>();
            let cleared = sut.set(.., false);
            // Then
            assert_eq!(total, filled);
            assert_eq!(total, full_count);
            assert_eq!(None, full_mex);
            assert_eq!(1, cleared_first);
            assert_eq!(1, cleared_last);
            assert_eq!(vec![0..=0, usize::MAX..=usize::MAX], gaps);
            assert_eq!(total - 2, cleared);
            assert_eq!(total, sut.count(.., false));
            assert_eq!(0..=usize::MAX, sut.run_at(usize::MAX));
        }

        /// Scenario: 排他的な両端と逆転範囲を空範囲として扱います。
        /// - Given: 1 の区間を含む列があります。
        /// - When: 値域外の排他的な下限、空の上限、逆転範囲を設定します。
        /// - Then: 個数は 0 で、状態が変わりません。
        #[test]
        fn ignores_empty_excluded_and_reversed_ranges() {
            // Given
            let mut sut = create_bits();
            let before = sut.clone();
            let empty = (ops::Bound::Excluded(usize::MAX), ops::Bound::Unbounded);
            // When
            let changed = [
                sut.set(empty, true),
                sut.set(..0, true),
                sut.set(5..5, false),
                sut.set((ops::Bound::Included(9), ops::Bound::Included(2)), false),
            ];
            // Then
            assert_eq!([0; 4], changed);
            assert_eq!(before, sut);
        }

        /// Scenario: 複数の範囲構文が同じ位置を更新します。
        /// - Given: 初期値がすべて 0 の列があります。
        /// - When: 半開・閉・端の省略・排他的下限を順に使います。
        /// - Then: 各構文の境界どおりに値が更新されます。
        #[test]
        fn accepts_all_bound_forms() {
            // Given
            let mut sut = IntervalBitSet::new();
            // When
            let changed = [
                sut.set(..2, true),
                sut.set(..=2, true),
                sut.set(3..5, true),
                sut.set(5..=6, true),
                sut.set((ops::Bound::Excluded(6), ops::Bound::Included(8)), true),
                sut.set(9.., true),
            ];
            // Then
            assert_eq!([2, 1, 2, 2, 2, usize::MAX as u128 - 8], changed);
            assert_eq!(0..=usize::MAX, sut.run_at(4));
        }
    }

    // get と run_at のテスト: 0 と 1 の点および最大の連続区間を検証します。
    mod point_queries {
        use super::*;

        /// Scenario: 点の値と、その点を含む最大の連続区間を返します。
        /// - Given: 離れた 2 個の 1 の区間があります。
        /// - When: 両端と内部の点を問い合わせます。
        /// - Then: 0 の隙間も 1 の区間も正しい両端が求まります。
        #[test]
        fn returns_values_and_maximal_runs_of_either_value() {
            // Given
            let sut = create_bits();
            let cases = [
                (0, false, 0..=1),
                (2, true, 2..=7),
                (7, true, 2..=7),
                (8, false, 8..=9),
                (10, true, 10..=14),
                (15, false, 15..=usize::MAX),
                (usize::MAX, false, 15..=usize::MAX),
            ];
            for (x, value, run) in cases {
                // When
                let actual_value = sut.get(x);
                let actual_run = sut.run_at(x);
                // Then
                assert_eq!(value, actual_value);
                assert_eq!(run, actual_run);
            }
        }
    }

    // find のテスト: 対象範囲と検索値、最大値における mex を検証します。
    mod find {
        use super::*;

        /// Scenario: 指定範囲の最初の一致位置を返します。
        /// - Given: 内部に 0 と 1 の区間がある列があります。
        /// - When: 両方の値と、空範囲、値がない範囲を検索します。
        /// - Then: 最小の一致位置または None を返します。
        #[test]
        fn finds_the_first_matching_position_within_bounds() {
            // Given
            let sut = create_bits();
            // When
            let results = [
                sut.find(4..12, false),
                sut.find(8..12, true),
                sut.find(2..8, false),
                sut.find(..2, true),
                sut.find(5..5, false),
                sut.find(usize::MAX.., false),
            ];
            // Then
            assert_eq!(
                [Some(8), Some(10), None, None, None, Some(usize::MAX)],
                results
            );
        }

        /// Scenario: 末尾が 1 のとき、最大値以上の mex は存在しません。
        /// - Given: 最大値の位置が 1 の列があります。
        /// - When: 最大値から 0 と 1 を検索します。
        /// - Then: 0 は None、1 は最大値を返します。
        #[test]
        fn returns_none_when_mex_would_exceed_the_domain() {
            // Given
            let mut sut = IntervalBitSet::new();
            sut.set(usize::MAX.., true);
            // When
            let zero = sut.find(usize::MAX.., false);
            let one = sut.find(usize::MAX.., true);
            // Then
            assert_eq!(None, zero);
            assert_eq!(Some(usize::MAX), one);
        }
    }

    // count のテスト: 部分範囲と全範囲の個数の一致を検証します。
    mod count {
        use super::*;

        /// Scenario: 指定範囲で切り取った位置の個数を返します。
        /// - Given: 離れた 2 個の 1 の区間があります。
        /// - When: 部分範囲と全範囲を両方の値で集計します。
        /// - Then: 両方の値の個数の和は問い合わせ範囲の長さになります。
        #[test]
        fn counts_clipped_ranges_and_the_whole_domain() {
            // Given
            let sut = create_bits();
            // When
            let counts = [
                sut.count(4..12, true),
                sut.count(4..12, false),
                sut.count(.., true),
                sut.count(.., false),
                sut.count(..0, false),
            ];
            // Then
            assert_eq!([6, 2, 11, usize::MAX as u128 + 1 - 11, 0], counts);
        }
    }

    // ranges のテスト: 切り取り、逆順、両端からの交互取得を検証します。
    mod ranges {
        use super::*;

        /// Scenario: 0 と 1 の区間を指定範囲で切り取って列挙します。
        /// - Given: 離れた 2 個の 1 の区間があります。
        /// - When: 部分範囲を昇順と降順で列挙します。
        /// - Then: 出力は指定範囲内の最大の連続区間になります。
        #[test]
        fn clips_runs_and_supports_reverse_iteration() {
            // Given
            let sut = create_bits();
            // When
            let ones = sut.ranges(4..12, true).collect::<Vec<_>>();
            let zeros = sut.ranges(..=16, false).rev().collect::<Vec<_>>();
            // Then
            assert_eq!(vec![4..=7, 10..=11], ones);
            assert_eq!(vec![15..=16, 8..=9, 0..=1], zeros);
        }

        /// Scenario: 両端から交互に取り出しても区間を重複して返しません。
        /// - Given: 0 と 1 の区間が交互に並ぶ列があります。
        /// - When: 先頭、末尾、先頭の順で取り出し、終了後も取得します。
        /// - Then: 各区間を一度だけ返し、終了後は両方向とも None になります。
        #[test]
        fn interleaves_front_and_back_without_duplicates() {
            // Given
            let sut = create_bits();
            let mut zeros = sut.ranges(.., false);
            let mut ones = sut.ranges(4..12, true);
            // When
            let zero_results = [
                zeros.next(),
                zeros.next_back(),
                zeros.next(),
                zeros.next_back(),
            ];
            let one_results = [ones.next_back(), ones.next(), ones.next_back(), ones.next()];
            // Then
            assert_eq!(
                [Some(0..=1), Some(15..=usize::MAX), Some(8..=9), None],
                zero_results,
            );
            assert_eq!([Some(10..=11), Some(4..=7), None, None], one_results);
            assert_eq!(None, zeros.next());
        }

        /// Scenario: 一様な列でも全範囲と空範囲を正しく列挙します。
        /// - Given: 全位置が 0 の列と、全位置が 1 の列があります。
        /// - When: 両方の値の全範囲と空範囲を列挙します。
        /// - Then: 一致値だけが全区間を返し、空範囲は何も返しません。
        #[test]
        fn handles_uniform_domains_and_empty_ranges() {
            // Given
            let mut sut = IntervalBitSet::new();
            for filled in [false, true] {
                sut.set(.., filled);
                // When
                let matching = sut.ranges(.., filled).rev().collect::<Vec<_>>();
                let opposite = sut.ranges(.., !filled).collect::<Vec<_>>();
                let empty = sut.ranges(..0, filled).collect::<Vec<_>>();
                // Then
                assert_eq!(vec![0..=usize::MAX], matching);
                assert!(opposite.is_empty());
                assert!(empty.is_empty());
            }
        }
    }

    // API の組み合わせのテスト: 小さなビット列の全状態で公開動作を照合します。
    mod model {
        use super::*;

        /// Scenario: 両方向からのすべての取得順で列挙結果が一致します。
        /// - Given: 6 ビットの全状態と、それぞれの部分範囲があります。
        /// - When: 0 と 1 の区間を左右の任意の順に取り出します。
        /// - Then: 素朴な区間配列と一致し、終了後は両方向とも None です。
        #[test]
        fn matches_all_double_ended_iteration_orders() {
            // Given
            let size = 6;
            for mask in 0..1 << size {
                let mut sut = IntervalBitSet::new();
                for x in 0..size {
                    sut.set(x..=x, mask & (1 << x) != 0);
                }
                for start in 0..=size {
                    for end in start..=size {
                        for value in [false, true] {
                            let expected = sut.ranges(start..end, value).collect::<Vec<_>>();
                            for order in 0..1 << expected.len() {
                                let mut expected = collections::VecDeque::from(expected.clone());
                                let mut actual = sut.ranges(start..end, value);
                                for turn in 0..expected.len() {
                                    // When
                                    let (next_expected, next_actual) = if order & (1 << turn) == 0 {
                                        (expected.pop_front(), actual.next())
                                    } else {
                                        (expected.pop_back(), actual.next_back())
                                    };
                                    // Then
                                    assert_eq!(next_expected, next_actual);
                                }
                                assert_eq!(None, actual.next());
                                assert_eq!(None, actual.next_back());
                            }
                        }
                    }
                }
            }
        }

        /// Scenario: 最大値付近で更新を繰り返しても個数と区間が一致します。
        /// - Given: 末尾 16 ビットを含む全範囲が、一様な値を持ちます。
        /// - When: 末尾の部分範囲に決定的な系列で 200 回の更新を行います。
        /// - Then: 各更新後の点、変更個数、区間列が素朴な配列と一致します。
        #[test]
        fn matches_repeated_updates_near_usize_max() {
            // Given
            let size = 16;
            let base = usize::MAX - (size - 1);
            for initial in [false, true] {
                let mut sut = IntervalBitSet::new();
                sut.set(.., initial);
                let mut expected = vec![initial; size];
                let mut state = 1_u64;
                for _ in 0..200 {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let start = (state >> 32) as usize % size;
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let end = start + (state >> 32) as usize % (size - start);
                    let value = state >> 63 != 0;
                    let changed = expected[start..=end]
                        .iter()
                        .filter(|&&x| x != value)
                        .count();
                    expected[start..=end].fill(value);
                    // When
                    let actual = sut.set(base + start..=base + end, value);
                    // Then
                    assert_eq!(changed as u128, actual);
                    for (x, &value) in expected.iter().enumerate() {
                        assert_eq!(value, sut.get(base + x));
                    }
                    for target in [false, true] {
                        let expanded = sut.ranges(base.., target).flatten().collect::<Vec<_>>();
                        let matching = expected
                            .iter()
                            .enumerate()
                            .filter(|&(_, &x)| x == target)
                            .map(|(i, _)| base + i)
                            .collect::<Vec<_>>();
                        assert_eq!(matching, expanded);
                        assert_eq!(matching.len() as u128, sut.count(base.., target));
                    }
                    let ones = expected.iter().filter(|&&x| x).count() as u128;
                    let prefix_ones = if initial { base as u128 } else { 0 };
                    assert_eq!(prefix_ones + ones, sut.count(.., true));
                }
            }
        }

        /// Scenario: 小さな列への全範囲更新が素朴な配列と一致します。
        /// - Given: 6 ビットの全初期状態があります。
        /// - When: すべての部分範囲に両方の値を設定します。
        /// - Then: 変更個数、点、個数、区間、検索、最大連続区間が配列と一致します。
        #[test]
        fn matches_a_dense_model_for_all_small_updates() {
            // Given
            let size = 6;
            for mask in 0..1 << size {
                let initial = (0..size).map(|x| mask & (1 << x) != 0).collect::<Vec<_>>();
                let mut before = IntervalBitSet::new();
                for (x, &value) in initial.iter().enumerate() {
                    before.set(x..=x, value);
                }
                for start in 0..=size {
                    for end in start..=size {
                        for value in [false, true] {
                            let mut sut = before.clone();
                            let mut expected = initial.clone();
                            let changed =
                                expected[start..end].iter().filter(|&&x| x != value).count();
                            expected[start..end].fill(value);
                            // When
                            let actual = sut.set(start..end, value);
                            // Then
                            assert_eq!(changed as u128, actual);
                            assert_eq!(
                                expected.iter().filter(|&&x| x).count() as u128,
                                sut.count(.., true)
                            );
                            for x in 0..size {
                                assert_eq!(expected[x], sut.get(x));
                                let left = (0..x)
                                    .rev()
                                    .find(|&i| expected[i] != expected[x])
                                    .map_or(0, |i| i + 1);
                                let right = (x + 1..size)
                                    .find(|&i| expected[i] != expected[x])
                                    .map_or(if expected[x] { size } else { usize::MAX }, |i| i);
                                let right = if right == usize::MAX {
                                    right
                                } else {
                                    right - 1
                                };
                                assert_eq!(left..=right, sut.run_at(x));
                            }
                            for target in [false, true] {
                                let mut expected_runs = Vec::new();
                                let mut x = start;
                                while x < end {
                                    if expected[x] != target {
                                        x += 1;
                                        continue;
                                    }
                                    let first = x;
                                    while x + 1 < end && expected[x + 1] == target {
                                        x += 1;
                                    }
                                    expected_runs.push(first..=x);
                                    x += 1;
                                }
                                assert_eq!(
                                    expected_runs,
                                    sut.ranges(start..end, target).collect::<Vec<_>>()
                                );
                                assert_eq!(
                                    expected_runs.iter().rev().cloned().collect::<Vec<_>>(),
                                    sut.ranges(start..end, target).rev().collect::<Vec<_>>()
                                );
                                assert_eq!(
                                    (start..end).find(|&i| expected[i] == target),
                                    sut.find(start..end, target)
                                );
                                assert_eq!(
                                    expected[start..end]
                                        .iter()
                                        .filter(|&&x| x == target)
                                        .count() as u128,
                                    sut.count(start..end, target)
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

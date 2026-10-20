//! `usize` の全範囲の 0/1 列を、値が連続する区間で管理します。
//!
//! `true` の最大の連続区間だけを保持し、それ以外を `false` とします。
//! 範囲の端の省略は `0` または `usize::MAX` を意味します。
//! 空範囲と逆転した範囲は、いずれも空として扱います。

use std::{cmp, collections, iter, ops};

/// `0..=usize::MAX` の各位置に真偽値を保持する、区間で圧縮したビット列です。
///
/// 初期値はすべて `false` です。隣接する同じ値の区間は結合します。
/// 保持する `true` の区間数を `K` とすると、記憶領域は `O(K)` です。
///
/// 列どうしの AND (`&`)、OR (`|`)、XOR (`^`) と、対応する代入演算に
/// 対応します。値と参照のどちらも演算の左右に指定できます。
/// 補集合 `!` と、値域外を切り捨てる `usize` 幅のシフト `<<`・`>>` にも
/// 対応します。区間列は `collect` で和集合として構築できます。
///
/// # 範囲の境界
/// 範囲引数は [`ops::RangeBounds<usize>`] を受け付けます。`l..r`、`l..=r`、
/// `..r`、`..=r`、`l..`、`..`、[`ops::Bound`] の組を指定できます。
/// 下端の省略は `0`、上端の省略は `usize::MAX` を含む指定です。
/// [`Self::run_at`] と [`Self::ranges`] は最大添字も表せる閉区間を返します。
/// 全値域の位置数は `usize::MAX + 1` なので、位置数と順序には `u128` を使います。
/// 空範囲や逆転範囲は空として扱い、更新は何もしません。
///
/// # 論理演算とシフト
/// `&` は共通部分、`|` は和集合、`^` は対称差を返します。
/// 左右に所有値・参照のどちらも指定でき、`&=`・`|=`・`^=` にも対応します。
/// 各演算の結果も最大連続区間へ結合され、借用した入力を変更しません。
/// `!bits` と `!&bits` は全値域での補集合であり、`&a & !&b` で差集合を作れます。
///
/// `bits << distance` は添字を増加させ、`bits >> distance` は減少させます。
/// `distance` は `usize` で、剰余を取らずにその距離だけ移動します。
/// 値域外へ移る部分を切り捨て、空いた位置には `false` を入れます。
/// 所有値・参照でのシフトと `<<=`・`>>=` に対応します。
///
/// # 区間列からの構築
/// `collect::<IntervalBitSet>()` と `IntervalBitSet::from_iter(ranges)` は、
/// 各区間の和集合を構築します。入力順・重複・隣接を問いません。
/// `extend(ranges)` は既存の列に各区間を 1 として追加します。
/// これらは区間を受け付けるため、点の列は `points.map(|x| x..=x)` として渡します。
/// `Default` は [`Self::new`] と同じ初期状態です。
/// 等値比較は更新履歴によらず、全位置の値が同じかを判定します。
///
/// # Complexity
/// 各メソッドの計算量は、それぞれの説明を参照してください。
/// 論理演算では、入力の区間数を `K, L`、結果の区間数を `R` とすると、
/// 境界走査は `O(K + L)`、木の構築を含む上限は
/// `O(K + L + R log(R + 1))` 時間、追加空間は `O(R)` です。
/// 補集合とシフトは、木の構築を含めて `O((K + 1) log(K + 2))` 時間、
/// 追加空間は `O(K + 1)` です。代入演算も同じです。
/// `N` 個の区間からの構築は `O(N log(N + 1))` 時間、追加空間は `O(N)`、
/// `extend` は各入力区間への [`Self::set`] と同じ計算量です。
///
/// 多数の位置を少数の区間へ圧縮できる入力に向きます。
/// 長さの条件による空き区間の検索は [`Self::ranges`] を走査するため、
/// 多数の連続ブロック検索を対数時間で処理する用途には対応していません。
///
/// # Examples
/// ```rust
/// use anmitsu::ds::interval_bit_set;
/// let mut a = interval_bit_set::IntervalBitSet::new();
/// let mut b = interval_bit_set::IntervalBitSet::new();
/// a.set(2..8, true);
/// b.set(5..10, true);
/// assert_eq!(3, (&a & &b).count(.., true));
/// assert_eq!(8, (&a | &b).count(.., true));
/// assert_eq!(5, (&a ^ &b).count(.., true));
/// a ^= &b;
/// assert_eq!(vec![2..=4, 8..=9], a.ranges(.., true).collect::<Vec<_>>());
/// ```
///
/// 区間列から構築し、移動後の差集合を指定範囲で切り出せます。
///
/// ```rust
/// use anmitsu::ds::interval_bit_set;
/// let a = [2..8, 10..15].into_iter()
///     .collect::<interval_bit_set::IntervalBitSet>();
/// let b = [4..6, 12..20].into_iter()
///     .collect::<interval_bit_set::IntervalBitSet>();
/// let shifted = (&a & !&b) << 3_usize;
/// let window = shifted.ranges(5..16, true)
///     .collect::<interval_bit_set::IntervalBitSet>();
/// assert_eq!(vec![5..=6, 9..=10, 13..=14],
///     window.ranges(.., true).collect::<Vec<_>>());
/// ```
#[derive(Clone, Debug, Default)]
pub struct IntervalBitSet {
    /// 始点から終点への対応です。閉区間は重ならず、互いに隣接しません。
    ones: collections::BTreeMap<usize, usize>,
    /// 順序検索と部分範囲の集計に使う、部分木の個数を持つ AVL 木です。
    index: Option<Box<RunIndex>>,
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
    /// 一時的な追加空間は `O(log(K + 2))` です。
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
                self.remove_run(left);
                previous += width(left, right);
                end = end.max(right);
            }
            self.insert_run(start, end);
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
                self.remove_run(left);
                changed += width(left.max(start), right.min(end));

                // 更新範囲の外に残る部分だけを戻し、内側を 0 にします。
                if left < start {
                    self.insert_run(left, start - 1);
                }
                if right > end {
                    self.insert_run(end + 1, right);
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
    /// 全範囲では時間は `O(1)`、それ以外は `O(log(K + 1))` です。
    /// 追加空間は `O(1)` です。
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
            RunIndex::prefix(&self.index, end as u128 + 1).0
                - RunIndex::prefix(&self.index, start as u128).0
        };
        if value {
            ones
        } else {
            width(start, end) - ones
        }
    }

    /// 指定範囲の 0 と 1 を反転します。
    ///
    /// # Args
    /// - `range`: 反転する範囲です。空範囲では何もしません。
    ///
    /// # Complexity
    /// 触れる区間数を `A` として `O((A + 1) log(K + A + 2))` 時間、
    /// 一時的な追加空間は `O(A + log(K + A + 2))` です。
    /// 範囲外の区間は走査しません。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// bits.flip(4..10);
    /// assert_eq!(vec![2..=3, 8..=9], bits.ranges(.., true).collect::<Vec<_>>());
    /// ```
    pub fn flip<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        let Some((start, end)) = normalize(range) else {
            return;
        };
        // 更新前の 0 の区間だけを保存し、借用中の列を変更しないようにします。
        let runs = self.ranges(start..=end, false).collect::<Vec<_>>();
        self.set(start..=end, false);
        for run in runs {
            self.set(run, true);
        }
    }

    /// 指定範囲内で、指定値を持つ k 番目の位置を返します。
    ///
    /// # Args
    /// - `range`: 検索する範囲です。
    /// - `value`: 検索する値です。
    /// - `k`: 範囲内での 0 始まりの順序です。
    ///
    /// # Returns
    /// 個数が足りない場合や空範囲では `None`、それ以外は該当添字です。
    ///
    /// # Complexity
    /// 時間は `O(log(K + 1))`、追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// assert_eq!(Some(9), bits.select(4..12, false, 1));
    /// assert_eq!(None, bits.select(4..12, true, 4));
    /// ```
    pub fn select<R: ops::RangeBounds<usize>>(
        &self,
        range: R,
        value: bool,
        k: u128,
    ) -> Option<usize> {
        let (start, end) = normalize(range)?;
        // 先に範囲内の個数を確認し、巨大な k の加算も安全にします。
        if k >= self.count(start..=end, value) {
            return None;
        }
        let before = self.count(..start, value);
        Some(RunIndex::select(&self.index, value, before + k))
    }

    /// 指定範囲で切り取った、指定値の最大連続区間の個数を返します。
    ///
    /// # Args
    /// - `range`: 集計する範囲です。
    /// - `value`: 数える区間の値です。
    ///
    /// # Returns
    /// 一致区間の個数です。空範囲では `0` です。
    ///
    /// # Complexity
    /// 時間は `O(log(K + 1))`、追加空間は `O(1)` です。
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::interval_bit_set;
    /// let mut bits = interval_bit_set::IntervalBitSet::new();
    /// bits.set(2..8, true);
    /// bits.set(10..15, true);
    /// assert_eq!(2, bits.count_ranges(4..12, true));
    /// assert_eq!(1, bits.count_ranges(4..12, false));
    /// ```
    pub fn count_ranges<R: ops::RangeBounds<usize>>(&self, range: R, value: bool) -> usize {
        let Some((start, end)) = normalize(range) else {
            return 0;
        };
        let first = self.get(start);
        // 始点が範囲より前でも、左端を覆う 1 の区間は 1 個として数えます。
        let ones = RunIndex::prefix(&self.index, end as u128 + 1).1
            - RunIndex::prefix(&self.index, start as u128 + 1).1
            + usize::from(first);
        if value {
            ones
        } else {
            // 0 と 1 は交互に現れるため、両端の値だけで補集合の数が決まります。
            ones + 1 - usize::from(first) - usize::from(self.get(end))
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

    /// 2 列の値が変わる境界を同時に走査し、論理演算の結果を構築します。
    ///
    /// # Args
    /// - `other`: 右辺の列です。
    /// - `operation`: 2 個の真偽値を結合します。両方が `false` なら
    ///   `false` を返す AND・OR・XOR に使用します。
    ///
    /// # Returns
    /// 同じ値が連続する区間を結合した、新しいビット列を返します。
    ///
    /// # Complexity
    /// 入力の区間数を `K, L`、出力の区間数を `R` とすると、境界走査は
    /// `O(K + L)`、木の構築を含む上限は `O(K + L + R log(R + 1))` です。
    /// 追加空間は `O(R)` です。
    fn combine<F>(&self, other: &Self, operation: F) -> Self
    where
        F: Fn(bool, bool) -> bool,
    {
        // 終端を u128 に広げ、usize::MAX の直後も通常の境界として扱います。
        let mut left = self
            .ones
            .iter()
            .flat_map(|(&start, &end)| [start as u128, end as u128 + 1])
            .peekable();
        let mut right = other
            .ones
            .iter()
            .flat_map(|(&start, &end)| [start as u128, end as u128 + 1])
            .peekable();
        let mut left_value = false;
        let mut right_value = false;
        let mut start = 0;
        let mut runs = Vec::new();
        let mut count_ones = 0;

        // 両方の境界が同じ位置にあれば、同時に反映して不要な区切りを防ぎます。
        loop {
            let position = match (left.peek(), right.peek()) {
                (Some(&a), Some(&b)) => a.min(b),
                (Some(&a), None) => a,
                (None, Some(&b)) => b,
                (None, None) => break,
            };
            let before = operation(left_value, right_value);
            if left.peek() == Some(&position) {
                left.next();
                left_value = !left_value;
            }
            if right.peek() == Some(&position) {
                right.next();
                right_value = !right_value;
            }
            let after = operation(left_value, right_value);

            // 結果の値が変わったときだけ、新たな 1 の区間を開始・終了します。
            if !before && after {
                start = position;
            } else if before && !after {
                runs.push((start as usize, (position - 1) as usize));
                count_ones += position - start;
            }
        }

        // 昇順で重複のない端点列から木をまとめて構築します。
        let result = Self::from_sorted_runs(runs);
        debug_assert_eq!(count_ones, result.count_ones);
        result
    }

    /// 走査用の木と集計用の木に、同じ区間を登録します。
    ///
    /// # Args
    /// - `start`, `end`: 登録する閉区間の両端です。
    fn insert_run(&mut self, start: usize, end: usize) {
        self.ones.insert(start, end);
        self.index = Some(RunIndex::insert(self.index.take(), start, end));
    }

    /// 走査用の木と集計用の木から、同じ区間を削除します。
    ///
    /// # Args
    /// - `start`: 削除する区間の始点です。
    fn remove_run(&mut self, start: usize) {
        self.ones.remove(&start);
        self.index = RunIndex::remove(self.index.take(), start);
    }

    /// 結合済みで昇順の区間列から、両方の木と総数を構築します。
    ///
    /// # Args
    /// - `runs`: 重複も隣接もない閉区間列です。
    ///
    /// # Returns
    /// 指定区間だけが 1 の列を返します。
    fn from_sorted_runs(runs: Vec<(usize, usize)>) -> Self {
        Self {
            count_ones: runs.iter().map(|&(start, end)| width(start, end)).sum(),
            index: RunIndex::build(&runs),
            ones: runs.into_iter().collect(),
        }
    }
}

/// 区間の内容だけを比較し、更新履歴による AVL 木の形の違いを無視します。
impl PartialEq for IntervalBitSet {
    /// 最大の連続区間が一致するか調べます。
    fn eq(&self, other: &Self) -> bool {
        self.ones == other.ones
    }
}

impl Eq for IntervalBitSet {}

/// 1 の区間を始点順に保持し、部分木の区間数と位置数を集計する AVL 木です。
#[derive(Clone, Debug)]
struct RunIndex {
    /// この節点の閉区間の始点です。
    start: usize,
    /// この節点の閉区間の終点です。
    end: usize,
    /// 高さの差を 1 以下に保つための、葉を 1 とする高さです。
    height: usize,
    /// 部分木内の 1 の区間数です。
    runs: usize,
    /// 部分木内の 1 の位置数です。
    ones: u128,
    /// 始点が小さい区間の部分木です。
    left: Option<Box<Self>>,
    /// 始点が大きい区間の部分木です。
    right: Option<Box<Self>>,
}

impl RunIndex {
    /// 1 区間だけを持つ節点を作ります。
    ///
    /// # Args
    /// - `start`, `end`: 閉区間の両端です。
    ///
    /// # Returns
    /// 集計値を初期化した葉を返します。
    fn new(start: usize, end: usize) -> Box<Self> {
        Box::new(Self {
            start,
            end,
            height: 1,
            runs: 1,
            ones: width(start, end),
            left: None,
            right: None,
        })
    }

    /// 左右の部分木の集計値から、この節点の集計値を復元します。
    fn pull(&mut self) {
        self.height = 1 + self
            .left
            .as_ref()
            .map_or(0, |n| n.height)
            .max(self.right.as_ref().map_or(0, |n| n.height));
        self.runs = 1
            + self.left.as_ref().map_or(0, |n| n.runs)
            + self.right.as_ref().map_or(0, |n| n.runs);
        self.ones = width(self.start, self.end)
            + self.left.as_ref().map_or(0, |n| n.ones)
            + self.right.as_ref().map_or(0, |n| n.ones);
    }

    /// 左部分木が右部分木より何段高いか返します。
    fn balance_factor(&self) -> isize {
        self.left.as_ref().map_or(0, |n| n.height) as isize
            - self.right.as_ref().map_or(0, |n| n.height) as isize
    }

    /// 右の子を親にする回転を行い、集計値を修復します。
    fn rotate_left(mut self: Box<Self>) -> Box<Self> {
        let mut root = self.right.take().unwrap();
        self.right = root.left.take();
        self.pull();
        root.left = Some(self);
        root.pull();
        root
    }

    /// 左の子を親にする回転を行い、集計値を修復します。
    fn rotate_right(mut self: Box<Self>) -> Box<Self> {
        let mut root = self.left.take().unwrap();
        self.left = root.right.take();
        self.pull();
        root.right = Some(self);
        root.pull();
        root
    }

    /// 挿入・削除後の高さの差を回転で解消します。
    fn rebalance(mut self: Box<Self>) -> Box<Self> {
        self.pull();
        if self.balance_factor() > 1 {
            // 内側に偏った子は先に逆向きへ回転し、二重回転にします。
            if self.left.as_ref().unwrap().balance_factor() < 0 {
                self.left = self.left.take().map(Self::rotate_left);
            }
            self.rotate_right()
        } else if self.balance_factor() < -1 {
            if self.right.as_ref().unwrap().balance_factor() > 0 {
                self.right = self.right.take().map(Self::rotate_right);
            }
            self.rotate_left()
        } else {
            self
        }
    }

    /// 指定始点を挿入または更新し、経路上の集計と平衡を修復します。
    ///
    /// # Args
    /// - `root`: 更新前の部分木です。
    /// - `start`, `end`: 登録する閉区間です。
    ///
    /// # Returns
    /// 更新後の部分木を返します。
    fn insert(root: Option<Box<Self>>, start: usize, end: usize) -> Box<Self> {
        let Some(mut root) = root else {
            return Self::new(start, end);
        };
        match start.cmp(&root.start) {
            cmp::Ordering::Less => {
                root.left = Some(Self::insert(root.left.take(), start, end));
            }
            cmp::Ordering::Greater => {
                root.right = Some(Self::insert(root.right.take(), start, end));
            }
            cmp::Ordering::Equal => root.end = end,
        }
        root.rebalance()
    }

    /// 指定始点を削除し、経路上の集計と平衡を修復します。
    ///
    /// # Args
    /// - `root`: 更新前の部分木です。
    /// - `start`: 削除する始点です。
    ///
    /// # Returns
    /// 更新後の部分木を返します。
    fn remove(root: Option<Box<Self>>, start: usize) -> Option<Box<Self>> {
        let mut root = root?;
        if start < root.start {
            root.left = Self::remove(root.left.take(), start);
        } else if start > root.start {
            root.right = Self::remove(root.right.take(), start);
        } else {
            // 子が 1 個以下なら、その子を親へ繰り上げられます。
            if root.left.is_none() {
                return root.right;
            }
            if root.right.is_none() {
                return root.left;
            }
            // 子が両方あれば、右部分木の最小キーで置き換えます。
            let mut next = root.right.as_deref().unwrap();
            while let Some(left) = next.left.as_deref() {
                next = left;
            }
            root.start = next.start;
            root.end = next.end;
            root.right = Self::remove(root.right.take(), root.start);
        }
        Some(root.rebalance())
    }

    /// 昇順の区間列の中央を親に選び、対数の高さの木を線形時間で作ります。
    ///
    /// # Args
    /// - `runs`: 昇順で重複しない閉区間列です。
    ///
    /// # Returns
    /// 構築した部分木を返します。
    fn build(runs: &[(usize, usize)]) -> Option<Box<Self>> {
        if runs.is_empty() {
            return None;
        }
        let middle = runs.len() / 2;
        let (start, end) = runs[middle];
        let mut root = Self::new(start, end);
        root.left = Self::build(&runs[..middle]);
        root.right = Self::build(&runs[middle + 1..]);
        root.pull();
        Some(root)
    }

    /// 指定位置より前にある 1 の個数と、区間の始点数を集計します。
    ///
    /// # Args
    /// - `root`: 検索対象の木です。
    /// - `end`: 排他的終端です。`usize::MAX + 1` まで指定できます。
    ///
    /// # Returns
    /// `(位置数, 始点数)` を返します。
    fn prefix(root: &Option<Box<Self>>, end: u128) -> (u128, usize) {
        let mut node = root.as_deref();
        let mut ones = 0;
        let mut runs = 0;
        while let Some(current) = node {
            if end <= current.start as u128 {
                node = current.left.as_deref();
            } else {
                // 始点を越えれば、左部分木を集計値だけで飛び越えられます。
                ones += current.left.as_ref().map_or(0, |n| n.ones)
                    + end.min(current.end as u128 + 1)
                    - current.start as u128;
                runs += current.left.as_ref().map_or(0, |n| n.runs) + 1;
                if end <= current.end as u128 + 1 {
                    break;
                }
                node = current.right.as_deref();
            }
        }
        (ones, runs)
    }

    /// 全値域で k 番目の指定値を、部分木の位置数を使って探します。
    ///
    /// # Args
    /// - `root`: 検索対象の木です。
    /// - `value`: 検索する値です。
    /// - `k`: 存在することを呼び出し側で確認済みの、0 始まりの順序です。
    ///
    /// # Returns
    /// 該当する添字を返します。
    fn select(root: &Option<Box<Self>>, value: bool, mut k: u128) -> usize {
        let mut node = root.as_deref();
        // lower は、祖先ですでに飛ばした領域の直後です。
        let mut lower = 0;
        while let Some(current) = node {
            let left_ones = current.left.as_ref().map_or(0, |n| n.ones);
            let left_count = if value {
                left_ones
            } else {
                current.start as u128 - lower - left_ones
            };
            if k < left_count {
                node = current.left.as_deref();
                continue;
            }
            k -= left_count;
            if value {
                let length = width(current.start, current.end);
                if k < length {
                    return (current.start as u128 + k) as usize;
                }
                k -= length;
            }
            lower = current.end as u128 + 1;
            node = current.right.as_deref();
        }
        // 0 の検索だけが、節点のない隙間または最後の区間の先へ到達します。
        debug_assert!(!value);
        (lower + k) as usize
    }
}

/// 同じ論理演算について、所有値・借用値と代入演算の入口を揃えます。
///
/// 区間を走査する処理は `combine` に集約し、所有権の違いで複製しません。
macro_rules! impl_bitwise_operation {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $operator:tt) => {
        impl ops::$trait<&IntervalBitSet> for &IntervalBitSet {
            /// 演算結果を保持するビット列です。
            type Output = IntervalBitSet;

            /// 両方の列を借用し、論理演算した新しい列を返します。
            fn $method(self, rhs: &IntervalBitSet) -> Self::Output {
                self.combine(rhs, |left, right| left $operator right)
            }
        }

        impl ops::$trait<&IntervalBitSet> for IntervalBitSet {
            /// 演算結果を保持するビット列です。
            type Output = Self;

            /// 左辺を消費し、借用した右辺との論理演算結果を返します。
            fn $method(self, rhs: &IntervalBitSet) -> Self::Output {
                ops::$trait::$method(&self, rhs)
            }
        }

        impl ops::$trait<IntervalBitSet> for &IntervalBitSet {
            /// 演算結果を保持するビット列です。
            type Output = IntervalBitSet;

            /// 左辺を借用し、所有する右辺との論理演算結果を返します。
            fn $method(self, rhs: IntervalBitSet) -> Self::Output {
                ops::$trait::$method(self, &rhs)
            }
        }

        impl ops::$trait for IntervalBitSet {
            /// 演算結果を保持するビット列です。
            type Output = Self;

            /// 両方の列を消費して論理演算した新しい列を返します。
            fn $method(self, rhs: Self) -> Self::Output {
                ops::$trait::$method(&self, &rhs)
            }
        }

        impl ops::$assign_trait<&IntervalBitSet> for IntervalBitSet {
            /// 借用した右辺との論理演算結果で左辺を置き換えます。
            fn $assign_method(&mut self, rhs: &IntervalBitSet) {
                *self = ops::$trait::$method(&*self, rhs);
            }
        }

        impl ops::$assign_trait for IntervalBitSet {
            /// 所有する右辺との論理演算結果で左辺を置き換えます。
            fn $assign_method(&mut self, rhs: Self) {
                ops::$assign_trait::$assign_method(self, &rhs);
            }
        }
    };
}

impl_bitwise_operation!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
impl_bitwise_operation!(BitOr, bitor, BitOrAssign, bitor_assign, |);
impl_bitwise_operation!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

impl ops::Not for &IntervalBitSet {
    /// 全値域で反転した列です。
    type Output = IntervalBitSet;

    /// 0 の区間を新しい 1 の区間として、補集合を構築します。
    fn not(self) -> Self::Output {
        IntervalBitSet::from_sorted_runs(
            self.ranges(.., false)
                .map(|run| (*run.start(), *run.end()))
                .collect(),
        )
    }
}

impl ops::Not for IntervalBitSet {
    /// 全値域で反転した列です。
    type Output = Self;

    /// 所有する列を消費し、補集合を返します。
    fn not(self) -> Self::Output {
        !&self
    }
}

/// 所有値・借用値と代入に対応するシフト演算を定義します。
macro_rules! impl_shift_operation {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $left:expr) => {
        impl ops::$trait<usize> for &IntervalBitSet {
            /// 値域外を切り捨てた移動後の列です。
            type Output = IntervalBitSet;

            /// 全区間を同じ距離だけ移動し、値域内に残る部分を構築します。
            fn $method(self, distance: usize) -> Self::Output {
                let runs = self
                    .ones
                    .iter()
                    .filter_map(|(&start, &end)| {
                        if $left {
                            // 左端まで桁あふれする区間は、全体が値域外です。
                            Some((start.checked_add(distance)?, end.saturating_add(distance)))
                        } else {
                            // 右端まで負になる区間は、全体が値域外です。
                            Some((start.saturating_sub(distance), end.checked_sub(distance)?))
                        }
                    })
                    .collect();
                IntervalBitSet::from_sorted_runs(runs)
            }
        }

        impl ops::$trait<usize> for IntervalBitSet {
            /// 値域外を切り捨てた移動後の列です。
            type Output = Self;

            /// 所有する列を消費してシフトします。
            fn $method(self, distance: usize) -> Self::Output {
                ops::$trait::$method(&self, distance)
            }
        }

        impl ops::$assign_trait<usize> for IntervalBitSet {
            /// シフト後の列で自分自身を置き換えます。
            fn $assign_method(&mut self, distance: usize) {
                *self = ops::$trait::$method(&*self, distance);
            }
        }
    };
}

impl_shift_operation!(Shl, shl, ShlAssign, shl_assign, true);
impl_shift_operation!(Shr, shr, ShrAssign, shr_assign, false);

impl<R: ops::RangeBounds<usize>> FromIterator<R> for IntervalBitSet {
    /// 区間列の和集合を作ります。重複・隣接・入力順を問いません。
    fn from_iter<T: IntoIterator<Item = R>>(ranges: T) -> Self {
        let mut ranges = ranges.into_iter().filter_map(normalize).collect::<Vec<_>>();
        ranges.sort_unstable();
        let mut runs = Vec::new();
        let mut remaining = ranges.into_iter();
        if let Some((mut start, mut end)) = remaining.next() {
            for (left, right) in remaining {
                if left <= end.saturating_add(1) {
                    end = end.max(right);
                } else {
                    runs.push((start, end));
                    (start, end) = (left, right);
                }
            }
            runs.push((start, end));
        }
        Self::from_sorted_runs(runs)
    }
}

impl<R: ops::RangeBounds<usize>> Extend<R> for IntervalBitSet {
    /// 渡された各区間を 1 に設定し、既存の区間へ追加します。
    fn extend<T: IntoIterator<Item = R>>(&mut self, ranges: T) {
        for range in ranges {
            self.set(range, true);
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

    // 追加 API のテスト: 全値域、空範囲、所有権、更新履歴を検証します。
    mod extended_operations {
        use super::*;

        /// Scenario: 全値域の反転と順序検索が最大添字を正確に扱います。
        /// - Given: 全体が 0 の列があります。
        /// - When: 全体、両端、内部を反転して順序検索します。
        /// - Then: 巨大な k と値域の両端を正確に処理します。
        #[test]
        fn flips_and_selects_across_the_entire_domain() {
            // Given
            let mut sut = IntervalBitSet::new();
            let last = usize::MAX;
            let total = last as u128 + 1;
            // When
            sut.flip(..);
            let full = [sut.select(.., true, last as u128), sut.select(.., false, 0)];
            sut.flip(1..last);
            // Then
            assert_eq!([Some(last), None], full);
            assert_eq!(2, sut.count(.., true));
            assert_eq!(Some(last), sut.select(.., true, 1));
            assert_eq!(Some(last - 1), sut.select(.., false, total - 3));
            assert_eq!(None, sut.select(.., false, total - 2));
            assert_eq!(None, sut.select(.., false, u128::MAX));
            assert_eq!(2, sut.count_ranges(.., true));
            assert_eq!(1, sut.count_ranges(.., false));
            sut.flip(..);
            assert_eq!(total - 2, sut.count(.., true));
            assert_eq!(Some(last), sut.select(.., false, 1));
        }

        /// Scenario: 空や逆転した範囲では追加操作も何も選びません。
        /// - Given: 離れた 1 の区間がある列と空範囲があります。
        /// - When: 各空範囲を反転、集計、順序検索します。
        /// - Then: 状態が変わらず、個数は 0、検索結果は None です。
        #[test]
        fn treats_empty_bounds_consistently() {
            // Given
            let mut sut = create_bits();
            let before = sut.clone();
            let cases = [
                (ops::Bound::Excluded(usize::MAX), ops::Bound::Unbounded),
                (ops::Bound::Unbounded, ops::Bound::Excluded(0)),
                (ops::Bound::Included(9), ops::Bound::Included(2)),
            ];
            for range in cases {
                // When
                sut.flip(range);
                // Then
                assert_eq!(before, sut);
                for value in [false, true] {
                    assert_eq!(0, sut.count_ranges(range, value));
                    assert_eq!(None, sut.select(range, value, 0));
                }
            }
        }

        /// Scenario: 補集合を取ると各値の位置と連続区間が入れ替わります。
        /// - Given: 両端が 0 で内部に 1 がある列があります。
        /// - When: 参照と所有値で補集合を求めます。
        /// - Then: 元の列を変更せず、二重反転で元に戻ります。
        #[test]
        fn complements_owned_and_borrowed_sets() {
            // Given
            let sut = create_bits();
            let before = sut.clone();
            // When
            let complement = !&sut;
            let owned = !sut.clone();
            // Then
            assert_eq!(before, sut);
            assert_eq!(complement, owned);
            assert_eq!(sut, !complement.clone());
            assert_eq!(
                sut.ranges(.., false).collect::<Vec<_>>(),
                complement.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(usize::MAX as u128 - 10, complement.count(.., true));
            assert_eq!(IntervalBitSet::new(), &sut & &complement);
            assert_eq!(0, (!IntervalBitSet::from_iter([..])).count(.., true));
        }

        /// Scenario: シフトは最大幅でも剰余を取らず、値域外だけを落とします。
        /// - Given: 両端に 1 を含む列があります。
        /// - When: 0、通常の距離、最大距離で左右にシフトします。
        /// - Then: 所有値、参照、代入とも同じ結果になり、空いた位置は 0 です。
        #[test]
        fn shifts_and_clips_at_both_extremes() {
            // Given
            let last = usize::MAX;
            let sut = IntervalBitSet::from_iter([0..=2, last - 2..=last]);
            // When
            let left = &sut << 2_usize;
            let right = &sut >> 2_usize;
            let mut assigned = sut.clone();
            assigned <<= last;
            let shifted_left = assigned.clone();
            assigned >>= last;
            // Then
            assert_eq!(sut, &sut << 0_usize);
            assert_eq!(sut, sut.clone() >> 0_usize);
            assert_eq!(left, sut.clone() << 2_usize);
            assert_eq!(right, sut.clone() >> 2_usize);
            assert_eq!(
                vec![2..=4, last..=last],
                left.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(
                vec![0..=0, last - 4..=last - 2],
                right.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(
                vec![last..=last],
                shifted_left.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(vec![0..=0], assigned.ranges(.., true).collect::<Vec<_>>());
            let mut missing = IntervalBitSet::from_iter([1..=1]);
            missing <<= last;
            assert_eq!(IntervalBitSet::new(), missing);
        }

        /// Scenario: 区間列の構築と追加は重複と隣接を結合します。
        /// - Given: 順不同で重複する区間と空範囲があります。
        /// - When: collect で構築し、extend で区間を追加します。
        /// - Then: 和集合の最大連続区間が得られます。
        #[test]
        fn collects_and_extends_ranges_as_a_union() {
            // Given
            let input = [10..15, 4..8, 5..5, 2..5];
            // When
            let mut sut = input.into_iter().collect::<IntervalBitSet>();
            let collected = sut.clone();
            sut.extend([8..10, 20..21]);
            // Then
            assert_eq!(create_bits(), collected);
            assert_eq!(
                vec![2..=14, 20..=20],
                sut.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(
                IntervalBitSet::new(),
                iter::empty::<ops::Range<usize>>().collect()
            );
            assert_eq!(
                usize::MAX as u128 + 1,
                IntervalBitSet::from_iter([..]).count(.., true)
            );
        }

        /// 木の高さ、部分木の集計、キーの順序を再帰的に検証します。
        ///
        /// # Args
        /// - `root`: 公開 API の操作で作られた木です。
        ///
        /// # Returns
        /// `(高さ, 区間数, 位置数, 最小始点, 最大始点)` を返します。
        fn assert_index(root: &Option<Box<RunIndex>>) -> (usize, usize, u128, usize, usize) {
            let Some(root) = root else {
                return (0, 0, 0, usize::MAX, 0);
            };
            let left = assert_index(&root.left);
            let right = assert_index(&root.right);
            assert!(left.0.abs_diff(right.0) <= 1);
            assert_eq!(1 + left.0.max(right.0), root.height);
            assert_eq!(1 + left.1 + right.1, root.runs);
            assert_eq!(width(root.start, root.end) + left.2 + right.2, root.ones);
            assert!(root.left.is_none() || left.4 < root.start);
            assert!(root.right.is_none() || root.start < right.3);
            (
                root.height,
                root.runs,
                root.ones,
                left.3.min(root.start),
                right.4.max(root.start),
            )
        }

        /// Scenario: 挿入、削除、結合、分割の後も集計用の木が平衡を保ちます。
        /// - Given: 昇順に追加する多数の離れた区間があります。
        /// - When: 順序を入れ替えて削除し、広い範囲の更新を繰り返します。
        /// - Then: 集計値と平衡条件が正しく、内容が同じ列は等しくなります。
        #[test]
        fn keeps_the_index_balanced_through_update_histories() {
            // Given
            let mut sut = IntervalBitSet::new();
            // When
            for x in 0..2048 {
                sut.set(3 * x..=3 * x, true);
            }
            for x in 0..2048 {
                let x = (x * 1237) % 2048;
                sut.set(3 * x..=3 * x, false);
                // Then
                let (_, runs, ones, _, _) = assert_index(&sut.index);
                assert_eq!(sut.ones.len(), runs);
                assert_eq!(sut.count(.., true), ones);
            }
            // When
            sut.extend([1..1000, 2000..3000, 4000..5000]);
            sut.flip(100..4500);
            let copied = sut.ranges(.., true).collect::<IntervalBitSet>();
            // Then
            assert_index(&sut.index);
            assert_eq!(copied, sut);
            assert_eq!(copied.count(500..4600, true), sut.count(500..4600, true));
        }
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

    // 論理演算のテスト: 全値域、所有権、正規化と素朴なビット演算を検証します。
    mod bitwise_operations {
        use super::*;

        /// Background: 既存の 2 区間と部分的に重なる、右辺の列を作ります。
        fn create_other() -> IntervalBitSet {
            let mut other = IntervalBitSet::new();
            other.set(5..12, true);
            other
        }

        /// Scenario: AND・OR・XOR は、入力を変更せず正規化した区間を返します。
        /// - Given: 部分的に重なる複数の 1 の区間があります。
        /// - When: 借用した 2 列を AND・OR・XOR します。
        /// - Then: 共通部分・和集合・対称差の区間と個数が一致し、元の列は変わりません。
        #[test]
        fn combines_borrowed_sets_into_maximal_runs() {
            // Given
            let sut = create_bits();
            let other = create_other();
            let before = sut.clone();
            let other_before = other.clone();
            // When
            let intersection = &sut & &other;
            let union = &sut | &other;
            let exclusive = &sut ^ &other;
            // Then
            assert_eq!(
                vec![5..=7, 10..=11],
                intersection.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(vec![2..=14], union.ranges(.., true).collect::<Vec<_>>());
            assert_eq!(
                vec![2..=4, 8..=9, 12..=14],
                exclusive.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(5, intersection.count(.., true));
            assert_eq!(13, union.count(.., true));
            assert_eq!(8, exclusive.count(.., true));
            assert_eq!(before, sut);
            assert_eq!(other_before, other);
        }

        /// Scenario: 所有値と参照のすべての組み合わせで演算できます。
        /// - Given: 部分的に重なる 2 列があります。
        /// - When: 借用・所有の組み合わせを変えて演算し、結果を次の演算へ渡します。
        /// - Then: 所有権の渡し方によらず同じ区間列が得られます。
        #[test]
        fn accepts_owned_and_borrowed_operands_and_chains_results() {
            // Given
            let sut = create_bits();
            let other = create_other();
            // When
            let intersections = [
                &sut & &other,
                sut.clone() & &other,
                &sut & other.clone(),
                sut.clone() & other.clone(),
            ];
            let unions = [
                &sut | &other,
                sut.clone() | &other,
                &sut | other.clone(),
                sut.clone() | other.clone(),
            ];
            let exclusives = [
                &sut ^ &other,
                sut.clone() ^ &other,
                &sut ^ other.clone(),
                sut.clone() ^ other.clone(),
            ];
            let chained = (&sut & &other) ^ (&sut | &other);
            // Then
            for result in intersections {
                assert_eq!(
                    vec![5..=7, 10..=11],
                    result.ranges(.., true).collect::<Vec<_>>()
                );
            }
            for result in unions {
                assert_eq!(vec![2..=14], result.ranges(.., true).collect::<Vec<_>>());
            }
            for result in exclusives.into_iter().chain([chained]) {
                assert_eq!(
                    vec![2..=4, 8..=9, 12..=14],
                    result.ranges(.., true).collect::<Vec<_>>()
                );
            }
        }

        /// Scenario: 代入演算は借用・所有どちらの右辺でも左辺を更新します。
        /// - Given: 部分的に重なる 2 列があります。
        /// - When: AND・OR・XOR の代入演算を適用します。
        /// - Then: 左辺の区間と個数が更新され、借用した右辺は変わりません。
        #[test]
        fn assigns_results_with_owned_or_borrowed_rhs() {
            // Given
            let other = create_other();
            let other_before = other.clone();
            for owned in [false, true] {
                let mut intersection = create_bits();
                let mut union = create_bits();
                let mut exclusive = create_bits();
                // When
                if owned {
                    intersection &= other.clone();
                    union |= other.clone();
                    exclusive ^= other.clone();
                } else {
                    intersection &= &other;
                    union |= &other;
                    exclusive ^= &other;
                }
                // Then
                assert_eq!(
                    vec![5..=7, 10..=11],
                    intersection.ranges(.., true).collect::<Vec<_>>()
                );
                assert_eq!(vec![2..=14], union.ranges(.., true).collect::<Vec<_>>());
                assert_eq!(
                    vec![2..=4, 8..=9, 12..=14],
                    exclusive.ranges(.., true).collect::<Vec<_>>()
                );
                assert_eq!(5, intersection.count(.., true));
                assert_eq!(13, union.count(.., true));
                assert_eq!(8, exclusive.count(.., true));
            }
            assert_eq!(other_before, other);
        }

        /// Scenario: 全値域の演算でも両端と要素数を正しく扱います。
        /// - Given: 全体、空集合、両端だけ、両端以外が 1 の列があります。
        /// - When: これらの列を AND・OR・XOR します。
        /// - Then: 0 と usize::MAX を含む区間も正しく結合・相殺されます。
        #[test]
        fn handles_empty_full_and_extreme_positions() {
            // Given
            let empty = IntervalBitSet::new();
            let mut full = IntervalBitSet::new();
            full.set(.., true);
            let mut edges = IntervalBitSet::new();
            edges.set(..=0, true);
            edges.set(usize::MAX.., true);
            let mut middle = IntervalBitSet::new();
            middle.set(1..usize::MAX, true);
            // When
            let results = [
                (&edges | &middle),
                (&edges ^ &middle),
                (&full | &empty),
                (&full & &full),
            ];
            let complement = &full ^ &edges;
            let cancelled = &full ^ &full;
            let disjoint = &edges & &middle;
            // Then
            for result in results {
                assert_eq!(
                    vec![0..=usize::MAX],
                    result.ranges(.., true).collect::<Vec<_>>()
                );
                assert_eq!(usize::MAX as u128 + 1, result.count(.., true));
            }
            assert_eq!(middle, complement);
            assert_eq!(empty, cancelled);
            assert_eq!(empty, disjoint);
            assert_eq!(edges, &full & &edges);
            assert_eq!(edges, &edges ^ &empty);
            assert_eq!(empty, &edges & &empty);
        }

        /// Scenario: 同じ境界での切り替えと 1 点だけの交差を扱います。
        /// - Given: 端で接する半開区間と、端を共有する閉区間があります。
        /// - When: AND・OR・XOR を適用します。
        /// - Then: 空の区間を生成せず、同じ値の隣接区間を結合します。
        #[test]
        fn handles_coincident_boundaries_and_singleton_intersections() {
            // Given
            let mut sut = IntervalBitSet::new();
            let mut other = IntervalBitSet::new();
            sut.set(1..5, true);
            other.set(5..9, true);
            // When
            let adjacent_intersection = &sut & &other;
            let adjacent_xor = &sut ^ &other;
            sut.set(5..=5, true);
            let singleton_intersection = &sut & &other;
            let exclusive = &sut ^ &other;
            // Then
            assert_eq!(0, adjacent_intersection.count(.., true));
            assert_eq!(
                vec![1..=8],
                adjacent_xor.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(
                vec![5..=5],
                singleton_intersection.ranges(.., true).collect::<Vec<_>>()
            );
            assert_eq!(
                vec![1..=4, 6..=8],
                exclusive.ranges(.., true).collect::<Vec<_>>()
            );
        }

        /// Scenario: 演算で作った列にも既存の更新・検索 API を適用できます。
        /// - Given: 2 列の XOR によって 3 個の区間ができています。
        /// - When: その間を埋める範囲更新を行い、連続区間と mex を取得します。
        /// - Then: 変更個数、総数、連続区間と mex が一致します。
        #[test]
        fn supports_updates_and_queries_on_operator_results() {
            // Given
            let mut sut = create_bits() ^ create_other();
            // When
            let changed = sut.set(3..13, true);
            let run = sut.run_at(4);
            let mex = sut.find(2.., false);
            // Then
            assert_eq!(5, changed);
            assert_eq!(13, sut.count(.., true));
            assert_eq!(2..=14, run);
            assert_eq!(Some(15), mex);
        }

        /// Scenario: 小さな全組み合わせで通常の整数ビット演算と一致します。
        /// - Given: 5 ビットのすべての状態を持つ 2 列があります。
        /// - When: 各組み合わせを AND・OR・XOR します。
        /// - Then: 点、区間、要素数、mex が整数のビット演算と一致します。
        #[test]
        fn matches_dense_bitwise_operations_for_all_small_pairs() {
            // Given
            let size = 5;
            for left_mask in 0..1_u32 << size {
                for right_mask in 0..1_u32 << size {
                    let mut left = IntervalBitSet::new();
                    let mut right = IntervalBitSet::new();
                    for x in 0..size {
                        left.set(x..=x, left_mask & (1 << x) != 0);
                        right.set(x..=x, right_mask & (1 << x) != 0);
                    }
                    // When
                    let results = [
                        (&left & &right, left_mask & right_mask),
                        (&left | &right, left_mask | right_mask),
                        (&left ^ &right, left_mask ^ right_mask),
                    ];
                    // Then
                    for (sut, mask) in results {
                        let expected = (0..size)
                            .filter(|&x| mask & (1 << x) != 0)
                            .collect::<Vec<_>>();
                        assert_eq!(expected.len() as u128, sut.count(.., true));
                        assert_eq!(expected, sut.ranges(.., true).flatten().collect::<Vec<_>>());
                        for x in 0..=size {
                            assert_eq!(mask & (1 << x) != 0, sut.get(x));
                        }
                        let expected_mex = (0..=size).find(|&x| mask & (1 << x) == 0);
                        assert_eq!(expected_mex, sut.find(.., false));
                        let runs = sut.ranges(.., true).collect::<Vec<_>>();
                        for adjacent in runs.windows(2) {
                            assert!(adjacent[0].end() + 1 < *adjacent[1].start());
                        }
                    }
                }
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

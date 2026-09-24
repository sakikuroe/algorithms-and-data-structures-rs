use std::ops::{Bound, RangeBounds};

/// 範囲境界を配列長以内の半開インデックス区間へ正規化する。
///
/// 含む境界と含まない境界を半開区間へ変換し、配列長を超える端点は `len` に丸める。
/// 正規化後に開始位置が終了位置を超える場合は、終了位置での空区間にする。
///
/// # Args
/// - `range`: `RangeBounds` で表したインデックス範囲。
/// - `len`: 配列の長さ。戻り値の各端点はこの値以下になる。
///
/// # Returns
/// 正規化後の半開区間 `(start, end)` を返す。`start <= end <= len` を満たす。
///
/// # Complexity
/// - 時間計算量: O(1) である。
/// - 空間計算量: O(1) である。
///
/// # Examples
/// ```rust
/// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
///
/// let matrix = WaveletMatrix::new(&[4, 7, 2]);
/// assert_eq!(2, matrix.count(1.., ..)); // 末尾の範囲は配列長までを対象とする。
/// assert_eq!(0, matrix.count(3..1, ..)); // 逆転した範囲は空として扱う。
/// ```
pub(super) fn normalize_index_range<R>(range: R, len: usize) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
    // 開始境界を包含インデックスへ直し、排他的境界は 1 を加えて半開区間にそろえる。
    let start = match range.start_bound() {
        Bound::Included(&value) => value,
        Bound::Excluded(&value) => value.saturating_add(1),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&value) => value.saturating_add(1),
        Bound::Excluded(&value) => value,
        Bound::Unbounded => len,
    };

    // 配列長を超える指定を末尾へ丸め、逆転した範囲は空区間として扱う。
    let start = start.min(len);
    let end = end.min(len);
    if start > end {
        (end, end)
    } else {
        (start, end)
    }
}

/// 値の範囲をソート済み一意値列上の半開インデックス範囲へ変換する。
///
/// 含む境界と含まない境界を `partition_point` で順位へ変換する。値が存在しない境界も
/// 正しい挿入位置へ写されるため、返る区間は値範囲に含まれる一意値の範囲を表す。
///
/// # Args
/// - `range`: `RangeBounds` で表した値範囲。
/// - `sorted_values`: 昇順に並び、重複を含まない値の列。
///
/// # Returns
/// `sorted_values` 上の半開区間 `(start, end)` を返す。逆転した境界の場合は `start > end`
/// となるため、呼び出し側で空範囲として扱う。
///
/// # Complexity
/// - 時間計算量: O(log(U + 1)) である。$U$ は `sorted_values` の長さである。
/// - 空間計算量: O(1) である。
///
/// # Examples
/// ```rust
/// use anmitsu::ds::wavelet_matrix::WaveletMatrix;
///
/// let matrix = WaveletMatrix::new(&[4, 7, 2, 7]);
/// assert_eq!(3, matrix.count(.., 4..=7));
/// assert_eq!(0, matrix.count(.., 7..4));
/// ```
pub(super) fn normalize_value_range<R>(range: R, sorted_values: &[usize]) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
    // 値の下限を満たす最初の順位と、上限までを含む最後の順位を二分探索で求める。
    let start = match range.start_bound() {
        Bound::Included(&value) => sorted_values.partition_point(|&x| x < value),
        Bound::Excluded(&value) => sorted_values.partition_point(|&x| x <= value),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&value) => sorted_values.partition_point(|&x| x <= value),
        Bound::Excluded(&value) => sorted_values.partition_point(|&x| x < value),
        Bound::Unbounded => sorted_values.len(),
    };

    (start, end)
}

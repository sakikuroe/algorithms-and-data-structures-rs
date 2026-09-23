use std::ops::{Bound, RangeBounds};

/// 範囲境界を配列の有効な半開区間へ変換する。
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

/// 値の範囲をソート済み値の添字範囲へ変換する。
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

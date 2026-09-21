use std::ops::{Bound, RangeBounds};

/// 範囲境界を配列の有効な半開区間へ変換する。
pub(super) fn normalize_index_range<R>(range: R, len: usize) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
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

    (start.min(len), end.min(len))
}

/// 値の範囲をソート済み値の添字範囲へ変換する。
pub(super) fn normalize_value_range<R>(range: R, sorted_values: &[usize]) -> (usize, usize)
where
    R: RangeBounds<usize>,
{
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

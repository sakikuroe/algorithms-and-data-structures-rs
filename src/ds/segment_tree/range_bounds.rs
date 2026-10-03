//! セグメント木の公開 API で指定された範囲を半開区間へ変換する。

use std::ops::{Bound, RangeBounds};

/// 包含・排他・無制限の境界を検証し、論理長内の半開区間を返す。
///
/// 範囲外や逆順の指定は拒否する。
/// `usize::MAX` を含む境界の加算も範囲外として扱う。
///
/// # Panics
/// 範囲が逆順、または論理長の外に出る場合にパニックする。
#[inline]
pub(super) fn normalize<R: RangeBounds<usize>>(range: R, len: usize, axis: &str) -> (usize, usize) {
    let start = match range.start_bound() {
        Bound::Included(&value) => value,
        Bound::Excluded(&value) => value.checked_add(1).expect("range out of bounds"),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&value) => value.checked_add(1).expect("range out of bounds"),
        Bound::Excluded(&value) => value,
        Bound::Unbounded => len,
    };
    assert!(start <= end && end <= len, "{axis} range out of bounds");
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全種類の境界を半開区間に変換できることを確認する。
    #[test]
    fn normalizes_inclusive_exclusive_and_unbounded_bounds() {
        assert_eq!((0, 5), normalize(.., 5, "index"));
        assert_eq!((1, 4), normalize(1..4, 5, "index"));
        assert_eq!((1, 4), normalize(1..=3, 5, "index"));
        assert_eq!((0, 3), normalize(..3, 5, "index"));
        assert_eq!((0, 3), normalize(..=2, 5, "index"));
        assert_eq!((2, 5), normalize(2.., 5, "index"));
        assert_eq!(
            (2, 4),
            normalize((Bound::Excluded(1), Bound::Included(3)), 5, "index")
        );
        assert_eq!((5, 5), normalize(5.., 5, "index"));
    }

    /// 逆順、論理長超過、端点の加算オーバーフローを拒否する。
    #[test]
    fn rejects_invalid_bounds() {
        assert!(
            std::panic::catch_unwind(|| {
                normalize((Bound::Included(4), Bound::Excluded(2)), 5, "index")
            })
            .is_err()
        );
        assert!(std::panic::catch_unwind(|| normalize(0..6, 5, "index")).is_err());
        assert!(std::panic::catch_unwind(|| normalize(..=5, 5, "index")).is_err());
        assert!(std::panic::catch_unwind(|| normalize(..=usize::MAX, 5, "index")).is_err());
        assert!(
            std::panic::catch_unwind(|| {
                normalize((Bound::Excluded(usize::MAX), Bound::Unbounded), 5, "index")
            })
            .is_err()
        );
    }
}

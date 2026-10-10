//! 密な 2 次元セグメント木を提供する。
//!
//! 各行方向の節点に列方向の木を持ち、可換モノイドで半開矩形を集約する。
//! 高さを `H`、幅を `W` とすると、領域は `O(HW)`、点更新と矩形集約は
//! `O(log(H + 1) log(W + 1))` 時間である。

use super::super::super::algebra::monoid;
use std::ops::RangeBounds;

/// 可換モノイドによる点更新と矩形集約を扱う密な 2 次元セグメント木。
///
/// 論理上の座標範囲は `[0, height) × [0, width)` であり、
/// 未設定の点は単位元を持つ。`set` の後は `build` を呼んでから
/// 矩形集約を行う。`update` は直ちに集約値へ反映される。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_dense};
///
/// let mut seg = segment_tree_2d_dense::SegmentTree2dDense::<monoid::AddMonoid>::new(3, 4);
/// seg.set((1, 2), 5);
/// seg.build();
/// seg.update((2, 3), 7);
/// assert_eq!(12, seg.fold(1..3, 2..4));
/// assert_eq!(0, seg.get((0, 0)));
/// ```
pub struct SegmentTree2dDense<M>
where
    M: monoid::CommutativeMonoid,
{
    height: usize,
    width: usize,
    row_size: usize,
    col_size: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTree2dDense<M>
where
    M: monoid::CommutativeMonoid,
    M::S: Clone,
{
    /// すべての点が単位元である `height × width` の木を作成する。
    ///
    /// # Args
    /// - `height` - 行座標の論理上の長さ。
    /// - `width` - 列座標の論理上の長さ。
    ///
    /// # Returns
    /// 空間 `O(height × width)` の 2 次元セグメント木を返す。
    ///
    /// # Panics
    /// 内部配列の長さが `usize` に収まらない場合にパニックする。
    ///
    /// # Complexity
    /// 時間・空間ともに `O(height × width)`。
    pub fn new(height: usize, width: usize) -> Self {
        let row_size = height
            .max(1)
            .checked_next_power_of_two()
            .expect("height is too large");
        let col_size = width
            .max(1)
            .checked_next_power_of_two()
            .expect("width is too large");
        let stride = col_size.checked_mul(2).expect("width is too large");
        let capacity = row_size
            .checked_mul(2)
            .and_then(|rows| rows.checked_mul(stride))
            .expect("segment tree is too large");
        Self {
            height,
            width,
            row_size,
            col_size,
            data: vec![M::id(); capacity],
        }
    }

    /// 論理上の行数を返す。
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// 論理上の列数を返す。
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// いずれかの次元が空であるかを返す。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.height == 0 || self.width == 0
    }

    /// 点の葉を設定し、祖先の集約は `build` に委ねる。
    ///
    /// # Args
    /// - `point` - 設定する行・列座標。
    /// - `value` - 点へ置く値。
    ///
    /// # Panics
    /// 座標が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに `O(1)`。
    pub fn set(&mut self, point: (usize, usize), value: M::S) {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        let stride = self.col_size * 2;
        self.data[(self.row_size + row) * stride + self.col_size + col] = value;
    }

    /// 設定済みの葉から行方向・列方向の集約値を構築する。
    ///
    /// # Complexity
    /// 時間 `O(height × width)`、追加領域 `O(1)`。
    pub fn build(&mut self) {
        let stride = self.col_size * 2;
        // まず各行の列方向の木を作る。
        for row in self.row_size..self.row_size + self.height {
            let base = row * stride;
            for col in (1..self.col_size).rev() {
                self.data[base + col] =
                    M::op(&self.data[base + col * 2], &self.data[base + col * 2 + 1]);
            }
        }
        // 親の各列節点は、左右の行区間にある同じ列区間の和である。
        for row in (1..self.row_size).rev() {
            let base = row * stride;
            let left = row * 2 * stride;
            let right = (row * 2 + 1) * stride;
            for col in 1..stride {
                self.data[base + col] = M::op(&self.data[left + col], &self.data[right + col]);
            }
        }
    }

    /// 点を更新し、行方向・列方向の祖先へ直ちに反映する。
    ///
    /// # Args
    /// - `point` - 更新する行・列座標。
    /// - `value` - 点の新しい値。
    ///
    /// # Panics
    /// 座標が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 `O(log(height + 1) log(width + 1))`、追加領域 `O(1)`。
    pub fn update(&mut self, point: (usize, usize), value: M::S) {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        let stride = self.col_size * 2;
        let mut row_node = self.row_size + row;
        let col_leaf = self.col_size + col;
        self.data[row_node * stride + col_leaf] = value;
        self.rebuild_columns(row_node, col_leaf);

        while row_node > 1 {
            row_node >>= 1;
            let base = row_node * stride;
            self.data[base + col_leaf] = M::op(
                &self.data[row_node * 2 * stride + col_leaf],
                &self.data[(row_node * 2 + 1) * stride + col_leaf],
            );
            self.rebuild_columns(row_node, col_leaf);
        }
    }

    /// 指定点の値を返す。
    ///
    /// # Args
    /// - `point` - 取得する行・列座標。
    ///
    /// # Returns
    /// 未設定点なら単位元、設定済みならその値を返す。
    ///
    /// # Panics
    /// 座標が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに `O(1)`。
    pub fn get(&self, point: (usize, usize)) -> M::S {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        self.data[(self.row_size + row) * (self.col_size * 2) + self.col_size + col].clone()
    }

    /// 指定した行範囲と列範囲の矩形を集約する。
    ///
    /// # Args
    /// - `rows` - 行範囲。`top..bottom`、`top..=bottom`、`..` などを指定できる。
    /// - `columns` - 列範囲。同様に指定できる。
    ///
    /// # Returns
    /// 矩形内の値の集約結果を返す。空矩形では単位元を返す。
    ///
    /// # Panics
    /// 端点が範囲外か、両端の順序が逆の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 `O(log(height + 1) log(width + 1))`、追加領域 `O(1)`。
    pub fn fold(&self, rows: impl RangeBounds<usize>, columns: impl RangeBounds<usize>) -> M::S {
        let (top, bottom) = super::range_bounds::normalize(rows, self.height, "row");
        let (left, right) = super::range_bounds::normalize(columns, self.width, "column");
        if top == bottom || left == right {
            return M::id();
        }

        let mut top_node = self.row_size + top;
        let mut bottom_node = self.row_size + bottom;
        let mut result = M::id();
        while top_node < bottom_node {
            if top_node & 1 == 1 {
                result = M::op(&result, &self.fold_columns(top_node, left, right));
                top_node += 1;
            }
            if bottom_node & 1 == 1 {
                bottom_node -= 1;
                result = M::op(&result, &self.fold_columns(bottom_node, left, right));
            }
            top_node >>= 1;
            bottom_node >>= 1;
        }
        result
    }

    /// 行節点内で、変更した列の祖先だけを再集約する。
    /// `row_node` は確保済みの行節点、`col_leaf` は列の葉節点を指す。
    fn rebuild_columns(&mut self, row_node: usize, col_leaf: usize) {
        debug_assert!(row_node > 0 && row_node < self.row_size * 2);
        debug_assert!(col_leaf >= self.col_size && col_leaf < self.col_size * 2);
        let base = row_node * (self.col_size * 2);
        let data = self.data.as_mut_ptr();
        let mut col = col_leaf >> 1;
        while col > 0 {
            // SAFETY: 1 <= col < col_size なので、親と左右の子は異なる節点で、
            // いずれも確保済みの row_node の範囲にある。この間 data は再確保しない。
            unsafe {
                let merged = M::op(&*data.add(base + col * 2), &*data.add(base + col * 2 + 1));
                *data.add(base + col) = merged;
            }
            col >>= 1;
        }
    }

    /// 行節点に対応する列方向の半開区間を集約する。
    fn fold_columns(&self, row_node: usize, left: usize, right: usize) -> M::S {
        let base = row_node * (self.col_size * 2);
        let mut l = self.col_size + left;
        let mut r = self.col_size + right;
        let mut result = M::id();
        while l < r {
            if l & 1 == 1 {
                result = M::op(&result, &self.data[base + l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                result = M::op(&result, &self.data[base + r]);
            }
            l >>= 1;
            r >>= 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::{segment_tree_2d_sparse_offline, segment_tree_2d_sparse_online};
    use super::*;

    /// 3 種の実装が同じ操作列に対して一致することを確認する。
    mod variants {
        use super::*;

        /// 3 種の実装が包含端点と無制限端点を同じ矩形へ変換する。
        #[test]
        fn accepts_range_bounds_on_both_axes() {
            let mut dense = SegmentTree2dDense::<monoid::AddMonoid>::new(3, 4);
            let mut offline = segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<
                monoid::AddMonoid,
            >::new([(0, 1), (2, 3)]);
            let mut online = segment_tree_2d_sparse_online::SegmentTree2dSparseOnline::<
                monoid::AddMonoid,
            >::new(3, 4);
            for (point, value) in [((0, 1), 4), ((2, 3), 7)] {
                dense.update(point, value);
                offline.update(point, value);
                online.update(point, value);
            }

            assert_eq!(11, dense.fold(.., ..));
            assert_eq!(11, offline.fold(.., ..));
            assert_eq!(11, online.fold(.., ..));
            assert_eq!(7, dense.fold(1..=2, 2..=3));
            assert_eq!(7, offline.fold(1..=2, 2..=3));
            assert_eq!(7, online.fold(1..=2, 2..=3));
            assert_eq!(4, dense.fold(..=1, ..2));
            assert_eq!(4, offline.fold(..=1, ..2));
            assert_eq!(4, online.fold(..=1, ..2));
            assert_eq!(0, dense.fold(3..3, ..));
            assert_eq!(0, offline.fold(3..3, ..));
            assert_eq!(0, online.fold(3..3, ..));
        }

        /// Scenario: 同じ値への再設定を含む更新列で矩形和が一致する。
        /// - Given: 全点を登録した 3 × 4 の木を 3 種用意する。
        /// - When: 各木に同じ更新を適用する。
        /// - Then: 全半開矩形の集約値が一致する。
        #[test]
        fn agree_on_same_operations() {
            // Given
            let points = (0..3)
                .flat_map(|row| (0..4).map(move |col| (row, col)))
                .collect::<Vec<_>>();
            let mut dense = SegmentTree2dDense::<monoid::AddMonoid>::new(3, 4);
            let mut offline = segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<
                monoid::AddMonoid,
            >::new(points);
            let mut online = segment_tree_2d_sparse_online::SegmentTree2dSparseOnline::<
                monoid::AddMonoid,
            >::new(3, 4);

            // When
            for (point, value) in [
                ((0, 0), 4),
                ((2, 3), 7),
                ((1, 2), -3),
                ((0, 0), 5),
                ((2, 3), 0),
            ] {
                dense.update(point, value);
                offline.update(point, value);
                online.update(point, value);

                // Then
                for top in 0..=3 {
                    for bottom in top..=3 {
                        for left in 0..=4 {
                            for right in left..=4 {
                                let expected = dense.fold(top..bottom, left..right);
                                assert_eq!(expected, offline.fold(top..bottom, left..right));
                                assert_eq!(expected, online.fold(top..bottom, left..right));
                            }
                        }
                    }
                }
            }
        }
    }

    /// 葉を設定した後の構築結果を確認する。
    mod build {
        use super::*;

        /// Scenario: 設定した点と未設定点が矩形集約に反映される。
        /// - Given: 2 × 3 の木の 2 点を設定する。
        /// - When: 木を構築して矩形を集約する。
        /// - Then: 設定した値だけが合計に含まれる。
        #[test]
        fn aggregates_set_points() {
            // Given
            let mut sut = SegmentTree2dDense::<monoid::AddMonoid>::new(2, 3);
            sut.set((0, 1), 4);
            sut.set((1, 2), 7);
            // When
            sut.build();
            // Then
            assert_eq!(11, sut.fold(0..2, 0..3));
            assert_eq!(4, sut.fold(0..1, 1..2));
            assert_eq!(0, sut.get((1, 1)));
        }
    }

    /// 点更新の伝播と再構築後の結果を確認する。
    mod update {
        use super::*;
        use rand::{Rng, SeedableRng, rngs};

        /// Scenario: 非 2 冪の格子で点更新がすべての包含矩形に反映される。
        /// - Given: 3 × 5 の木に初期値を設定して構築する。
        /// - When: 右下の点を更新する。
        /// - Then: その点を含む矩形だけが変化する。
        #[test]
        fn updates_non_power_of_two_grid() {
            // Given
            let mut sut = SegmentTree2dDense::<monoid::AddMonoid>::new(3, 5);
            sut.set((0, 0), 2);
            sut.set((2, 4), 3);
            sut.build();
            // When
            sut.update((2, 4), 9);
            // Then
            assert_eq!(11, sut.fold(0..3, 0..5));
            assert_eq!(2, sut.fold(0..2, 0..4));
            assert_eq!(9, sut.get((2, 4)));
        }

        /// Scenario: 更新を繰り返してもすべての矩形が素朴な和に一致する。
        /// - Given: 5 × 7 の木と同じ形の配列がある。
        /// - When: 複数の点を更新する。
        /// - Then: 各半開矩形の和が配列から計算した値に一致する。
        #[test]
        fn matches_naive_rectangle_sums() {
            // Given
            let mut sut = SegmentTree2dDense::<monoid::AddMonoid>::new(5, 7);
            let mut values = [[0_i64; 7]; 5];
            let mut rng = rngs::StdRng::seed_from_u64(1);
            // When
            for _ in 0..40 {
                let row = rng.random_range(0..5);
                let col = rng.random_range(0..7);
                let value = rng.random_range(-20..=20);
                sut.update((row, col), value);
                values[row][col] = value;

                // Then
                for top in 0..=5 {
                    for bottom in top..=5 {
                        for left in 0..=7 {
                            for right in left..=7 {
                                let expected = values[top..bottom]
                                    .iter()
                                    .flat_map(|line| &line[left..right])
                                    .sum::<i64>();
                                assert_eq!(expected, sut.fold(top..bottom, left..right));
                            }
                        }
                    }
                }
            }
        }
    }

    /// 空の次元と空矩形の扱いを確認する。
    mod empty {
        use super::*;

        /// Scenario: 行数または列数が 0 の木の集約は単位元になる。
        /// - Given: 一方の次元が空の木がある。
        /// - When: 論理上の全範囲を集約する。
        /// - Then: 木は空で、集約値は単位元である。
        #[test]
        fn folds_empty_dimensions() {
            // Given
            let rows = SegmentTree2dDense::<monoid::AddMonoid>::new(0, 5);
            let cols = SegmentTree2dDense::<monoid::AddMonoid>::new(5, 0);
            // When
            let row_result = rows.fold(0..0, 0..5);
            let col_result = cols.fold(0..5, 0..0);
            // Then
            assert!(rows.is_empty());
            assert!(cols.is_empty());
            assert_eq!(0, row_result);
            assert_eq!(0, col_result);
            assert_eq!(0, rows.height());
            assert_eq!(5, cols.height());
        }
    }

    /// 範囲外の座標を拒否することを確認する。
    mod bounds {
        use super::*;

        /// Scenario: 論理上の幅に等しい列の更新を拒否する。
        /// - Given: 幅 3 の木がある。
        /// - When: 列 3 を更新する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "point out of bounds")]
        fn rejects_out_of_bounds_point() {
            // Given
            let mut sut = SegmentTree2dDense::<monoid::AddMonoid>::new(2, 3);
            // When
            sut.update((0, 3), 1);
        }
    }
}

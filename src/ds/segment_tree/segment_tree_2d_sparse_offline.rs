//! 更新候補点を事前登録する疎な 2 次元セグメント木を提供する。
//!
//! 行座標を圧縮し、各行方向の節点に現れる列座標だけを保持する。
//! 登録点数を `K` とすると、領域は `O(K log(K + 1))`、点更新と
//! 矩形集約は `O(log²(K + 1))` 時間である。矩形集約には可換モノイドを使う。

use super::super::super::algebra::monoid;

/// 行節点に付属する列方向の木が、連続配列内で占める範囲。
#[derive(Clone, Copy, Default)]
struct InnerLayout {
    /// 登録済み列座標の開始位置。
    coord_start: usize,
    /// この行節点内の異なる列座標数。
    coord_len: usize,
    /// 列方向の木の開始位置。
    data_start: usize,
    /// 列方向の木の葉数。列座標がなければ 0。
    size: usize,
}

/// 更新候補点を事前に指定する疎な 2 次元セグメント木。
///
/// 論理上の座標範囲は `[0, height) × [0, width)` である。
/// 未登録点の `get` は単位元を返し、`set` と `update` は登録点だけを
/// 受け付ける。`set` の後は `build` で祖先を再集約する。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_offline};
///
/// let mut seg = segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<
///     monoid::AddMonoid,
/// >::new(100, 100, [(2, 5), (80, 90)]);
/// seg.set((2, 5), 3);
/// seg.build();
/// seg.update((80, 90), 7);
/// assert_eq!(10, seg.fold((0, 0), (100, 100)));
/// assert_eq!(0, seg.get((50, 50)));
/// ```
pub struct SegmentTree2dSparseOffline<M>
where
    M: monoid::CommutativeMonoid,
{
    height: usize,
    width: usize,
    x_coordinates: Vec<usize>,
    row_size: usize,
    layouts: Vec<InnerLayout>,
    y_coordinates: Vec<usize>,
    data: Vec<M::S>,
}

impl<M> SegmentTree2dSparseOffline<M>
where
    M: monoid::CommutativeMonoid,
    M::S: Clone,
{
    /// 更新候補点を登録し、すべて単位元の木を作成する。
    ///
    /// # Args
    /// - `height` - 行座標の論理上の長さ。
    /// - `width` - 列座標の論理上の長さ。
    /// - `points` - 更新する可能性のある点。順序と重複を問わない。
    ///
    /// # Returns
    /// 更新候補点に応じた領域だけを確保した木を返す。
    ///
    /// # Panics
    /// 登録点が範囲外か、配列の長さが `usize` に収まらない場合に
    /// パニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(K log²(K + 1))`、
    /// 空間 `O(K log(K + 1))`。
    pub fn new(
        height: usize,
        width: usize,
        points: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        let mut points = points.into_iter().collect::<Vec<_>>();
        assert!(
            points.iter().all(|&(row, col)| row < height && col < width),
            "registered point out of bounds"
        );
        points.sort_unstable();
        points.dedup();
        let mut x_coordinates = points.iter().map(|&(row, _)| row).collect::<Vec<_>>();
        x_coordinates.dedup();
        let row_size = x_coordinates
            .len()
            .max(1)
            .checked_next_power_of_two()
            .expect("too many registered rows");
        let row_capacity = row_size.checked_mul(2).expect("too many registered rows");
        let mut local_y = vec![Vec::new(); row_capacity];

        // 各登録点の列座標を、その行葉から根までの全節点へ配る。
        let mut x_rank = 0;
        for &(row, col) in &points {
            while x_coordinates[x_rank] < row {
                x_rank += 1;
            }
            let mut node = row_size + x_rank;
            while node > 0 {
                local_y[node].push(col);
                node >>= 1;
            }
        }

        // 列方向の座標と木を、それぞれ一つの配列に詰めて保持する。
        let mut layouts = vec![InnerLayout::default(); row_capacity];
        let mut y_coordinates = Vec::new();
        let mut data = Vec::new();
        for node in 1..row_capacity {
            let coordinates = &mut local_y[node];
            coordinates.sort_unstable();
            coordinates.dedup();
            let size = if coordinates.is_empty() {
                0
            } else {
                coordinates
                    .len()
                    .checked_next_power_of_two()
                    .expect("too many registered columns")
            };
            layouts[node] = InnerLayout {
                coord_start: y_coordinates.len(),
                coord_len: coordinates.len(),
                data_start: data.len(),
                size,
            };
            y_coordinates.extend_from_slice(coordinates);
            let capacity = size.checked_mul(2).expect("too many registered columns");
            let new_len = data
                .len()
                .checked_add(capacity)
                .expect("segment tree is too large");
            data.resize_with(new_len, M::id);
        }

        Self {
            height,
            width,
            x_coordinates,
            row_size,
            layouts,
            y_coordinates,
            data,
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

    /// 登録点の葉を設定し、祖先の集約は `build` に委ねる。
    ///
    /// # Args
    /// - `point` - 登録済みの行・列座標。
    /// - `value` - 点へ置く値。
    ///
    /// # Panics
    /// 点が範囲外、または未登録の場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(log(K + 1))`、追加領域 `O(1)`。
    pub fn set(&mut self, point: (usize, usize), value: M::S) {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        let row_rank = self
            .x_coordinates
            .binary_search(&row)
            .expect("point is not registered");
        let node = self.row_size + row_rank;
        let layout = self.layouts[node];
        let coord = &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        let col_rank = coord.binary_search(&col).expect("point is not registered");
        self.data[layout.data_start + layout.size + col_rank] = value;
    }

    /// 設定済みの点から、列方向と行方向の集約値を構築する。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(K log(K + 1))`、
    /// 追加領域 `O(1)`。
    pub fn build(&mut self) {
        for node in self.row_size..self.layouts.len() {
            self.rebuild_inner(node);
        }

        for node in (1..self.row_size).rev() {
            let parent = self.layouts[node];
            if parent.coord_len == 0 {
                continue;
            }
            let left = self.layouts[node * 2];
            let right = self.layouts[node * 2 + 1];
            let mut left_rank = 0;
            let mut right_rank = 0;
            for rank in 0..parent.coord_len {
                let col = self.y_coordinates[parent.coord_start + rank];
                while left_rank < left.coord_len
                    && self.y_coordinates[left.coord_start + left_rank] < col
                {
                    left_rank += 1;
                }
                while right_rank < right.coord_len
                    && self.y_coordinates[right.coord_start + right_rank] < col
                {
                    right_rank += 1;
                }
                let left_value = if left_rank < left.coord_len
                    && self.y_coordinates[left.coord_start + left_rank] == col
                {
                    self.data[left.data_start + left.size + left_rank].clone()
                } else {
                    M::id()
                };
                let right_value = if right_rank < right.coord_len
                    && self.y_coordinates[right.coord_start + right_rank] == col
                {
                    self.data[right.data_start + right.size + right_rank].clone()
                } else {
                    M::id()
                };
                self.data[parent.data_start + parent.size + rank] =
                    M::op(&left_value, &right_value);
            }
            self.rebuild_inner(node);
        }
    }

    /// 登録点を更新し、行方向・列方向の祖先へ直ちに反映する。
    ///
    /// # Args
    /// - `point` - 登録済みの行・列座標。
    /// - `value` - 点の新しい値。
    ///
    /// # Panics
    /// 点が範囲外、または未登録の場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(log²(K + 1))`、追加領域 `O(1)`。
    pub fn update(&mut self, point: (usize, usize), mut value: M::S) {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        let row_rank = self
            .x_coordinates
            .binary_search(&row)
            .expect("point is not registered");
        let mut node = self.row_size + row_rank;
        let layout = self.layouts[node];
        let coordinates =
            &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        let col_rank = coordinates
            .binary_search(&col)
            .expect("point is not registered");
        self.data[layout.data_start + layout.size + col_rank] = value.clone();
        self.rebuild_inner_path(node, col_rank);

        while node > 1 {
            let sibling = node ^ 1;
            let sibling_value = self.inner_get(sibling, col);
            value = if node & 1 == 0 {
                M::op(&value, &sibling_value)
            } else {
                M::op(&sibling_value, &value)
            };
            node >>= 1;
            let layout = self.layouts[node];
            let coordinates =
                &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
            let col_rank = coordinates
                .binary_search(&col)
                .expect("point is not registered");
            self.data[layout.data_start + layout.size + col_rank] = value.clone();
            self.rebuild_inner_path(node, col_rank);
        }
    }

    /// 点の値を返し、未登録点には単位元を返す。
    ///
    /// # Args
    /// - `point` - 取得する行・列座標。
    ///
    /// # Returns
    /// 登録点の現在値、または単位元を返す。
    ///
    /// # Panics
    /// 点が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(log(K + 1))`、追加領域 `O(1)`。
    pub fn get(&self, point: (usize, usize)) -> M::S {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        match self.x_coordinates.binary_search(&row) {
            Ok(rank) => self.inner_get(self.row_size + rank, col),
            Err(_) => M::id(),
        }
    }

    /// 半開矩形 `[top, bottom) × [left, right)` を集約する。
    ///
    /// # Args
    /// - `top_left` - 矩形の上端と左端。
    /// - `bottom_right` - 矩形の下端と右端。
    ///
    /// # Returns
    /// 矩形内の値の集約結果を返す。空矩形では単位元を返す。
    ///
    /// # Panics
    /// 端点が範囲外か、両端の順序が逆の場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(log²(K + 1))`、追加領域 `O(1)`。
    pub fn fold(&self, top_left: (usize, usize), bottom_right: (usize, usize)) -> M::S {
        let (top, left) = top_left;
        let (bottom, right) = bottom_right;
        assert!(
            top <= bottom && bottom <= self.height,
            "row range out of bounds"
        );
        assert!(
            left <= right && right <= self.width,
            "column range out of bounds"
        );
        if top == bottom || left == right {
            return M::id();
        }

        let mut top_node = self.row_size + self.x_coordinates.partition_point(|&x| x < top);
        let mut bottom_node = self.row_size + self.x_coordinates.partition_point(|&x| x < bottom);
        let mut result = M::id();
        while top_node < bottom_node {
            if top_node & 1 == 1 {
                result = M::op(&result, &self.inner_fold(top_node, left, right));
                top_node += 1;
            }
            if bottom_node & 1 == 1 {
                bottom_node -= 1;
                result = M::op(&result, &self.inner_fold(bottom_node, left, right));
            }
            top_node >>= 1;
            bottom_node >>= 1;
        }
        result
    }

    /// 行節点に登録された列座標の葉値を取得する。
    fn inner_get(&self, node: usize, col: usize) -> M::S {
        let layout = self.layouts[node];
        let coordinates =
            &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        match coordinates.binary_search(&col) {
            Ok(rank) => self.data[layout.data_start + layout.size + rank].clone(),
            Err(_) => M::id(),
        }
    }

    /// 行節点の列方向の木を、葉から根まで再構築する。
    fn rebuild_inner(&mut self, node: usize) {
        let layout = self.layouts[node];
        for col in (1..layout.size).rev() {
            self.data[layout.data_start + col] = M::op(
                &self.data[layout.data_start + col * 2],
                &self.data[layout.data_start + col * 2 + 1],
            );
        }
    }

    /// 行節点で変更した列の祖先だけを再集約する。
    fn rebuild_inner_path(&mut self, node: usize, col_rank: usize) {
        let layout = self.layouts[node];
        let mut col = (layout.size + col_rank) >> 1;
        while col > 0 {
            self.data[layout.data_start + col] = M::op(
                &self.data[layout.data_start + col * 2],
                &self.data[layout.data_start + col * 2 + 1],
            );
            col >>= 1;
        }
    }

    /// 行節点内で、登録列座標に対する半開区間を集約する。
    fn inner_fold(&self, node: usize, left: usize, right: usize) -> M::S {
        let layout = self.layouts[node];
        let coordinates =
            &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        // 接頭辞では右端の順位だけ探し、左側の兄弟を集約する。
        if left == 0 {
            let rank = if right == self.width {
                layout.coord_len
            } else {
                coordinates.partition_point(|&col| col < right)
            };
            if rank == 0 {
                return M::id();
            }
            if rank == layout.coord_len {
                return self.data[layout.data_start + 1].clone();
            }
            let mut index = layout.size + rank;
            let mut result = M::id();
            while index > 1 {
                if index & 1 == 1 {
                    result = M::op(&self.data[layout.data_start + index - 1], &result);
                }
                index >>= 1;
            }
            return result;
        }

        let mut l = layout.size + coordinates.partition_point(|&col| col < left);
        let mut r = layout.size
            + if right == self.width {
                layout.coord_len
            } else {
                coordinates.partition_point(|&col| col < right)
            };
        let mut result = M::id();
        while l < r {
            if l & 1 == 1 {
                result = M::op(&result, &self.data[layout.data_start + l]);
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                result = M::op(&result, &self.data[layout.data_start + r]);
            }
            l >>= 1;
            r >>= 1;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 未整列の登録点からの構築結果を確認する。
    mod build {
        use super::*;

        /// Scenario: 重複登録した点が 1 個の葉として集約される。
        /// - Given: 3 × 5 の木に順序と重複のある点を登録する。
        /// - When: 登録点を設定して構築する。
        /// - Then: 設定値と未登録点の単位元を取得できる。
        #[test]
        fn sorts_deduplicates_and_aggregates() {
            // Given
            let mut sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
                3,
                5,
                [(2, 4), (0, 1), (2, 4), (2, 0)],
            );
            sut.set((2, 4), 7);
            sut.set((0, 1), 3);
            // When
            sut.build();
            // Then
            assert_eq!(10, sut.fold((0, 0), (3, 5)));
            assert_eq!(7, sut.fold((1, 2), (3, 5)));
            assert_eq!(0, sut.get((1, 4)));
            assert_eq!(0, sut.get((2, 0)));
        }
    }

    /// 登録済みの点を更新した後の矩形集約を確認する。
    mod update {
        use super::*;

        /// Scenario: 列座標が行ごとに異なっても更新値が伝播する。
        /// - Given: 一部の行と列だけに登録点がある。
        /// - When: 登録点を複数回更新する。
        /// - Then: 各矩形の集約値が現在の点値に一致する。
        #[test]
        fn propagates_across_different_column_sets() {
            // Given
            let mut sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
                10,
                10,
                [(1, 1), (1, 8), (4, 3), (9, 8)],
            );
            // When
            sut.update((1, 8), 5);
            sut.update((9, 8), 7);
            sut.update((4, 3), 2);
            sut.update((1, 8), 11);
            // Then
            assert_eq!(20, sut.fold((0, 0), (10, 10)));
            assert_eq!(18, sut.fold((0, 8), (10, 9)));
            assert_eq!(2, sut.fold((2, 0), (9, 8)));
            assert_eq!(11, sut.get((1, 8)));
        }

        /// Scenario: 登録点の更新を繰り返しても素朴な矩形和に一致する。
        /// - Given: 5 × 6 の格子の一部を登録した木と配列がある。
        /// - When: 登録点を繰り返し更新する。
        /// - Then: すべての半開矩形の和が配列の和に一致する。
        #[test]
        fn matches_naive_rectangle_sums() {
            // Given
            let points = (0..5)
                .flat_map(|row| (0..6).map(move |col| (row, col)))
                .filter(|&(row, col)| (row + col) % 3 == 0)
                .collect::<Vec<_>>();
            let mut sut =
                SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(5, 6, points.iter().copied());
            let mut values = [[0_i64; 6]; 5];
            // When
            for step in 0..24 {
                let (row, col) = points[step * 7 % points.len()];
                let value = step as i64 - 8;
                sut.update((row, col), value);
                values[row][col] = value;

                // Then
                for top in 0..=5 {
                    for bottom in top..=5 {
                        for left in 0..=6 {
                            for right in left..=6 {
                                let expected = values[top..bottom]
                                    .iter()
                                    .flat_map(|line| &line[left..right])
                                    .sum::<i64>();
                                assert_eq!(expected, sut.fold((top, left), (bottom, right)));
                            }
                        }
                    }
                }
            }
        }
    }

    /// 登録点がない木の操作を確認する。
    mod empty {
        use super::*;

        /// Scenario: 登録点がなくても任意の矩形を集約できる。
        /// - Given: 登録点のない広い論理範囲がある。
        /// - When: 全範囲を集約する。
        /// - Then: 単位元を返す。
        #[test]
        fn folds_without_registered_points() {
            // Given
            let sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(100, 200, []);
            // When
            let result = sut.fold((0, 0), (100, 200));
            // Then
            assert_eq!(0, result);
            assert_eq!(0, sut.get((50, 150)));
            assert!(!sut.is_empty());
        }
    }

    /// 未登録点への更新を拒否することを確認する。
    mod bounds {
        use super::*;

        /// Scenario: 論理範囲内でも未登録の点は更新できない。
        /// - Given: 点 (1, 1) だけを登録した木がある。
        /// - When: 未登録の点 (1, 2) を更新する。
        /// - Then: 未登録点としてパニックする。
        #[test]
        #[should_panic(expected = "point is not registered")]
        fn rejects_unregistered_point() {
            // Given
            let mut sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(4, 4, [(1, 1)]);
            // When
            sut.update((1, 2), 9);
        }
    }
}

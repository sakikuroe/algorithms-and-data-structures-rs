//! 更新点を事前登録しない疎な 2 次元セグメント木を提供する。
//!
//! 行方向と列方向の節点を、更新時に必要になった経路だけ生成する。
//! 更新した異なる点数を `P` とすると、領域は
//! `O(P log(H + 1) log(W + 1))`、点更新と矩形集約は
//! `O(log(H + 1) log(W + 1))` 時間である。

use super::super::super::algebra::monoid;
use std::ops::RangeBounds;

/// 子の節点がまだ生成されていないことを表す。
const NONE: usize = usize::MAX;

/// 行方向の区間と、その区間に属する列方向の木を保持する。
struct RowNode {
    /// 左右の行子。未生成の子には `NONE` を入れる。
    children: [usize; 2],
    /// この行区間に対する列方向の木の根。
    column_root: usize,
}

/// 列方向の区間と、そこに属する値の集約を保持する。
struct ColumnNode<S> {
    /// 担当する行・列区間の集約値。
    value: S,
    /// 左右の列子。未生成の子には `NONE` を入れる。
    children: [usize; 2],
}

/// 再帰中に共有する半開矩形の四つの端点。
struct Rectangle {
    /// 行の下限。
    top: usize,
    /// 行の上限。
    bottom: usize,
    /// 列の下限。
    left: usize,
    /// 列の上限。
    right: usize,
}

/// 更新点を事前に指定せずに使える疎な 2 次元セグメント木。
///
/// 論理上の範囲は `[0, height) × [0, width)` であり、
/// 未設定の点は単位元を持つ。`set` と `update` はともに
/// 点の変更を直ちに矩形集約へ反映する。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_online};
///
/// let mut seg = segment_tree_2d_sparse_online::SegmentTree2dSparseOnline::<
///     monoid::AddMonoid,
/// >::new(1_000_000, 1_000_000);
/// seg.update((2, 5), 3);
/// seg.set((800_000, 900_000), 7);
/// assert_eq!(10, seg.fold(0..1_000_000, 0..1_000_000));
/// assert_eq!(0, seg.get((50, 50)));
/// ```
pub struct SegmentTree2dSparseOnline<M>
where
    M: monoid::CommutativeMonoid,
{
    height: usize,
    width: usize,
    rows: Vec<RowNode>,
    columns: Vec<ColumnNode<M::S>>,
}

impl<M> SegmentTree2dSparseOnline<M>
where
    M: monoid::CommutativeMonoid,
    M::S: Clone,
{
    /// すべての点が単位元である論理上の木を作成する。
    ///
    /// # Args
    /// - `height` - 行座標の論理上の長さ。
    /// - `width` - 列座標の論理上の長さ。
    ///
    /// # Returns
    /// 根以外の節点を生成していない木を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに `O(1)`。
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            rows: vec![RowNode {
                children: [NONE; 2],
                column_root: NONE,
            }],
            columns: Vec::new(),
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

    /// 点を設定し、変更を直ちに矩形集約へ反映する。
    ///
    /// # Args
    /// - `point` - 設定する行・列座標。
    /// - `value` - 点の新しい値。
    ///
    /// # Panics
    /// 点が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに `O(log(height + 1) log(width + 1))`。
    pub fn set(&mut self, point: (usize, usize), value: M::S) {
        self.update(point, value);
    }

    /// 点を更新し、通過する行と列の節点だけを生成する。
    ///
    /// # Args
    /// - `point` - 更新する行・列座標。
    /// - `value` - 点の新しい値。
    ///
    /// # Panics
    /// 点が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・追加領域ともに `O(log(height + 1) log(width + 1))`。
    pub fn update(&mut self, point: (usize, usize), mut value: M::S) {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");

        // 行方向の経路を記録し、葉から戻りながら列の木を更新する。
        let mut path = [0; usize::BITS as usize];
        let mut directions = [0; usize::BITS as usize];
        let mut depth = 0;
        let mut node = 0;
        let mut lower = 0;
        let mut upper = self.height;
        while upper - lower > 1 {
            let middle = lower + (upper - lower) / 2;
            let direction = usize::from(row >= middle);
            path[depth] = node;
            directions[depth] = direction;
            depth += 1;
            let child = self.rows[node].children[direction];
            if child == NONE {
                let created = self.rows.len();
                self.rows.push(RowNode {
                    children: [NONE; 2],
                    column_root: NONE,
                });
                self.rows[node].children[direction] = created;
                node = created;
            } else {
                node = child;
            }
            if direction == 0 {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        self.update_column(node, col, value.clone());

        // 兄弟の同じ列にある値を集約し、祖先の列方向の木へ反映する。
        while depth > 0 {
            depth -= 1;
            let parent = path[depth];
            let direction = directions[depth];
            let sibling = self.rows[parent].children[1 - direction];
            let sibling_value = if sibling == NONE {
                M::id()
            } else {
                self.column_get(self.rows[sibling].column_root, col)
            };
            value = if direction == 0 {
                M::op(&value, &sibling_value)
            } else {
                M::op(&sibling_value, &value)
            };
            self.update_column(parent, col, value.clone());
        }
    }

    /// 点の値を取得し、未設定点には単位元を返す。
    ///
    /// # Args
    /// - `point` - 取得する行・列座標。
    ///
    /// # Returns
    /// 点の現在値、または単位元を返す。
    ///
    /// # Panics
    /// 点が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 時間 `O(log(height + 1) + log(width + 1))`、追加領域 `O(1)`。
    pub fn get(&self, point: (usize, usize)) -> M::S {
        let (row, col) = point;
        assert!(row < self.height && col < self.width, "point out of bounds");
        let mut node = 0;
        let mut lower = 0;
        let mut upper = self.height;
        while upper - lower > 1 {
            let middle = lower + (upper - lower) / 2;
            let direction = usize::from(row >= middle);
            node = self.rows[node].children[direction];
            if node == NONE {
                return M::id();
            }
            if direction == 0 {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        self.column_get(self.rows[node].column_root, col)
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
    /// 時間 `O(log(height + 1) log(width + 1))`、
    /// 追加領域 `O(log(height + 1) + log(width + 1))`。
    pub fn fold(&self, rows: impl RangeBounds<usize>, columns: impl RangeBounds<usize>) -> M::S {
        let (top, bottom) = super::range_bounds::normalize(rows, self.height, "row");
        let (left, right) = super::range_bounds::normalize(columns, self.width, "column");
        if top == bottom || left == right {
            return M::id();
        }
        let rectangle = Rectangle {
            top,
            bottom,
            left,
            right,
        };
        self.fold_rows(0, 0, self.height, &rectangle)
    }

    /// 行節点の列方向の木に点値を設定し、列の祖先を再集約する。
    fn update_column(&mut self, row_node: usize, col: usize, value: M::S) {
        let mut node = self.rows[row_node].column_root;
        if node == NONE {
            node = self.columns.len();
            self.columns.push(ColumnNode {
                value: M::id(),
                children: [NONE; 2],
            });
            self.rows[row_node].column_root = node;
        }
        let mut path = [0; usize::BITS as usize];
        let mut depth = 0;
        let mut lower = 0;
        let mut upper = self.width;
        while upper - lower > 1 {
            let middle = lower + (upper - lower) / 2;
            let direction = usize::from(col >= middle);
            path[depth] = node;
            depth += 1;
            let child = self.columns[node].children[direction];
            if child == NONE {
                let created = self.columns.len();
                self.columns.push(ColumnNode {
                    value: M::id(),
                    children: [NONE; 2],
                });
                self.columns[node].children[direction] = created;
                node = created;
            } else {
                node = child;
            }
            if direction == 0 {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        self.columns[node].value = value;

        let identity = M::id();
        while depth > 0 {
            depth -= 1;
            node = path[depth];
            let [left, right] = self.columns[node].children;
            let left_value = if left == NONE {
                &identity
            } else {
                &self.columns[left].value
            };
            let right_value = if right == NONE {
                &identity
            } else {
                &self.columns[right].value
            };
            self.columns[node].value = M::op(left_value, right_value);
        }
    }

    /// 行節点に付属する列方向の木から、指定列の値を読む。
    fn column_get(&self, root: usize, col: usize) -> M::S {
        if root == NONE {
            return M::id();
        }
        let mut node = root;
        let mut lower = 0;
        let mut upper = self.width;
        while upper - lower > 1 {
            let middle = lower + (upper - lower) / 2;
            let direction = usize::from(col >= middle);
            node = self.columns[node].children[direction];
            if node == NONE {
                return M::id();
            }
            if direction == 0 {
                upper = middle;
            } else {
                lower = middle;
            }
        }
        self.columns[node].value.clone()
    }

    /// 行方向の部分木をたどり、完全に含まれる節点で列方向を集約する。
    fn fold_rows(&self, node: usize, lower: usize, upper: usize, rectangle: &Rectangle) -> M::S {
        if node == NONE || rectangle.bottom <= lower || upper <= rectangle.top {
            return M::id();
        }
        if rectangle.top <= lower && upper <= rectangle.bottom {
            return self.fold_columns(
                self.rows[node].column_root,
                0,
                self.width,
                rectangle.left,
                rectangle.right,
            );
        }
        let middle = lower + (upper - lower) / 2;
        let left_value = self.fold_rows(self.rows[node].children[0], lower, middle, rectangle);
        let right_value = self.fold_rows(self.rows[node].children[1], middle, upper, rectangle);
        M::op(&left_value, &right_value)
    }

    /// 列方向の部分木で半開区間を集約する。
    fn fold_columns(
        &self,
        node: usize,
        lower: usize,
        upper: usize,
        left: usize,
        right: usize,
    ) -> M::S {
        if node == NONE || right <= lower || upper <= left {
            return M::id();
        }
        if left <= lower && upper <= right {
            return self.columns[node].value.clone();
        }
        let middle = lower + (upper - lower) / 2;
        let left_value =
            self.fold_columns(self.columns[node].children[0], lower, middle, left, right);
        let right_value =
            self.fold_columns(self.columns[node].children[1], middle, upper, left, right);
        M::op(&left_value, &right_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 点更新が矩形集約へ反映されることを確認する。
    mod update {
        use super::*;

        /// Scenario: 行ごとに異なる列の値も正しく集約する。
        /// - Given: まばらな点を持つ論理範囲がある。
        /// - When: 同じ点を含め複数回更新する。
        /// - Then: 各矩形の値は現在の点値の和と一致する。
        #[test]
        fn propagates_through_rows_and_columns() {
            // Given
            let mut sut = SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(10, 10);
            // When
            sut.set((1, 8), 5);
            sut.update((9, 8), 7);
            sut.update((4, 3), 2);
            sut.update((1, 8), 11);
            // Then
            assert_eq!(20, sut.fold(0..10, 0..10));
            assert_eq!(18, sut.fold(0..10, 8..9));
            assert_eq!(2, sut.fold(2..9, 0..8));
            assert_eq!(11, sut.get((1, 8)));
            assert_eq!(0, sut.get((1, 7)));
        }

        /// Scenario: 更新を繰り返してもすべての矩形和が素朴な計算に一致する。
        /// - Given: 5 × 6 の格子と同じ大きさの配列がある。
        /// - When: 点値を繰り返し変更する。
        /// - Then: 各半開矩形の集約値は配列の和と一致する。
        #[test]
        fn matches_naive_rectangle_sums() {
            // Given
            let mut sut = SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(5, 6);
            let mut values = [[0_i64; 6]; 5];
            // When
            for step in 0..24 {
                let row = step * 7 % 5;
                let col = step * 11 % 6;
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
                                assert_eq!(expected, sut.fold(top..bottom, left..right));
                            }
                        }
                    }
                }
            }
        }
    }

    /// 大きな論理範囲でも必要な経路だけを生成することを確認する。
    mod large_coordinates {
        use super::*;

        /// Scenario: 遠く離れた二点の値を個別に取得・集約できる。
        /// - Given: 各軸が 10 億の論理範囲がある。
        /// - When: 対角付近の二点を更新する。
        /// - Then: 狭い矩形と全体の集約値が正しい。
        #[test]
        fn handles_far_apart_points() {
            // Given
            let mut sut =
                SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(1_000_000_000, 1_000_000_000);
            // When
            sut.update((0, 0), 3);
            sut.update((999_999_999, 999_999_999), 5);
            // Then
            assert_eq!(8, sut.fold(0..1_000_000_000, 0..1_000_000_000));
            assert_eq!(
                5,
                sut.fold(999_999_999..1_000_000_000, 999_999_999..1_000_000_000)
            );
            assert_eq!(0, sut.get((500_000_000, 500_000_000)));
        }
    }

    /// 空の次元と未生成の点を確認する。
    mod empty {
        use super::*;

        /// Scenario: 空次元では空矩形の集約値を返す。
        /// - Given: 行数が 0 の木がある。
        /// - When: 全範囲を集約する。
        /// - Then: 単位元を返す。
        #[test]
        fn folds_empty_dimension() {
            // Given
            let sut = SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(0, 7);
            // When
            let result = sut.fold(0..0, 0..7);
            // Then
            assert_eq!(0, result);
            assert!(sut.is_empty());
            assert_eq!(0, sut.height());
            assert_eq!(7, sut.width());
        }
    }

    /// 論理範囲外の点を拒否することを確認する。
    mod bounds {
        use super::*;

        /// Scenario: 行の上限にある点は更新できない。
        /// - Given: 4 × 4 の木がある。
        /// - When: 行 4 を更新する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "point out of bounds")]
        fn rejects_out_of_bounds_point() {
            // Given
            let mut sut = SegmentTree2dSparseOnline::<monoid::AddMonoid>::new(4, 4);
            // When
            sut.update((4, 1), 9);
        }
    }
}

//! 更新候補点を事前登録する疎な 2 次元セグメント木を提供する。
//!
//! 行座標を圧縮し、各行方向の節点に現れる列座標だけを保持する。
//! 登録点数を `K` とすると、領域は `O(K log(K + 1))`、点更新と
//! 矩形集約は `O(log²(K + 1))` 時間である。矩形集約には可換モノイドを使う。
//! `build` 後に `prepare_prefix_folds` を呼ぶと、列の先頭からの矩形集約を
//! `O(log(K + 1))` 時間で処理できる。

use super::super::super::algebra::monoid;
use std::ops::RangeBounds;

/// 行節点に付属する列方向の木が、連続配列内で占める範囲。
#[derive(Clone, Copy, Default)]
struct InnerLayout {
    /// 登録済み列座標の開始位置。
    coord_start: usize,
    /// この行節点内の異なる列座標数。
    coord_len: usize,
    /// 列方向の木の開始位置。
    data_start: usize,
    /// 列方向の木の葉数。登録列座標数と同じ。
    size: usize,
    /// 親の列順位から左右の子の列順位への対応表の開始位置。
    rank_start: usize,
}

/// 更新候補点を事前に指定する疎な 2 次元セグメント木。
///
/// 論理上の座標範囲は `[0, height) × [0, width)` である。
/// 未登録点の `get` は単位元を返し、`set` と `update` は登録点だけを
/// 受け付ける。`set` の後は `build` で祖先を再集約する。
/// 構築後の接頭辞集約が多い場合は `prepare_prefix_folds` を呼ぶ。
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
/// assert_eq!(10, seg.fold(0..100, 0..100));
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
    /// 親順位に対応する左右の子の順位を下位・上位 32 bit に保持する。
    child_ranks: Vec<u64>,
    /// 論理列幅が小さいときの、列端点から根の圧縮順位への対応表。
    root_ranks: Option<Vec<u32>>,
    /// 構築後に準備した列方向の接頭辞集約。更新時に無効化する。
    prefix_data: Option<Vec<M::S>>,
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
    /// 登録点が範囲外か、配列の長さが `usize` に収まらない場合、
    /// または行節点の登録列数が `u32` に収まらない場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(K log(K + 1))`、
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

        // 点は行・列の順に整列済みなので、各行葉の列座標も昇順になる。
        let mut x_rank = 0;
        for &(row, col) in &points {
            while x_coordinates[x_rank] < row {
                x_rank += 1;
            }
            local_y[row_size + x_rank].push(col);
        }

        // 親の列座標は子の整列済み座標を併合する。同じ列は一度だけ残す。
        for node in (1..row_size).rev() {
            let left = &local_y[node * 2];
            let right = &local_y[node * 2 + 1];
            let mut merged = Vec::with_capacity(left.len() + right.len());
            let mut left_rank = 0;
            let mut right_rank = 0;
            while left_rank < left.len() || right_rank < right.len() {
                let col = if right_rank == right.len()
                    || (left_rank < left.len() && left[left_rank] <= right[right_rank])
                {
                    let col = left[left_rank];
                    left_rank += 1;
                    col
                } else {
                    let col = right[right_rank];
                    right_rank += 1;
                    col
                };
                if merged.last().copied() != Some(col) {
                    merged.push(col);
                }
            }
            local_y[node] = merged;
        }

        // 列方向の座標と木を、それぞれ一つの配列に詰めて保持する。
        let mut layouts = vec![InnerLayout::default(); row_capacity];
        let mut y_coordinates = Vec::new();
        let mut data = Vec::new();
        let mut child_ranks = Vec::new();
        for node in 1..row_capacity {
            let coordinates = &local_y[node];
            // 集約演算が可換なので、葉数を 2 の累乗に揃える必要はない。
            let size = coordinates.len();
            layouts[node] = InnerLayout {
                coord_start: y_coordinates.len(),
                coord_len: coordinates.len(),
                data_start: data.len(),
                size,
                rank_start: child_ranks.len(),
            };
            y_coordinates.extend_from_slice(coordinates);
            if node < row_size {
                let left = &local_y[node * 2];
                let right = &local_y[node * 2 + 1];
                let mut left_rank = 0;
                let mut right_rank = 0;
                child_ranks.push(0);
                for &col in coordinates {
                    if left.get(left_rank) == Some(&col) {
                        left_rank += 1;
                    }
                    if right.get(right_rank) == Some(&col) {
                        right_rank += 1;
                    }
                    let left_rank_u32 =
                        u32::try_from(left_rank).expect("too many registered columns");
                    let right_rank_u32 =
                        u32::try_from(right_rank).expect("too many registered columns");
                    child_ranks.push((u64::from(right_rank_u32) << 32) | u64::from(left_rank_u32));
                }
            }
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
            child_ranks,
            root_ranks: None,
            prefix_data: None,
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
        // 全行が登録済みなら、論理座標と圧縮後の順位が一致する。
        let row_rank = if self.x_coordinates.len() == self.height {
            row
        } else {
            self.x_coordinates
                .binary_search(&row)
                .expect("point is not registered")
        };
        let node = self.row_size + row_rank;
        let layout = self.layouts[node];
        let coord = &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        let col_rank = coord.binary_search(&col).expect("point is not registered");
        self.prefix_data = None;
        self.data[layout.data_start + layout.size + col_rank] = value;
    }

    /// 設定済みの点から、列方向と行方向の集約値を構築する。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間 `O(K log(K + 1))`、
    /// 追加領域 `O(1)`。
    pub fn build(&mut self) {
        self.prefix_data = None;
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

    /// 構築済みの木に対し、列の先頭からの集約を各行節点で事前計算する。
    ///
    /// 以後の接頭辞矩形集約は、列方向の木をたどらずに値を取得する。
    /// `set`、`update`、`build` を行うと事前計算結果は無効になる。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、時間と追加領域は `O(K log(K + 1))`。
    pub fn prepare_prefix_folds(&mut self) {
        if self.root_ranks.is_none() {
            let root = self.layouts[1];
            // 論理列幅が登録列数の 4 倍以下なら、直引き表の追加領域も O(K)。
            if self.width < usize::MAX && self.width <= root.coord_len.saturating_mul(4) {
                let coordinates =
                    &self.y_coordinates[root.coord_start..root.coord_start + root.coord_len];
                let mut ranks = Vec::with_capacity(self.width + 1);
                let mut rank = 0;
                for endpoint in 0..=self.width {
                    while rank < coordinates.len() && coordinates[rank] < endpoint {
                        rank += 1;
                    }
                    ranks.push(u32::try_from(rank).expect("too many registered columns"));
                }
                self.root_ranks = Some(ranks);
            }
        }
        let mut prefix_data = Vec::with_capacity(self.y_coordinates.len());
        for layout in &self.layouts[1..] {
            let mut aggregate = M::id();
            for rank in 0..layout.coord_len {
                aggregate = M::op(
                    &aggregate,
                    &self.data[layout.data_start + layout.size + rank],
                );
                prefix_data.push(aggregate.clone());
            }
        }
        self.prefix_data = Some(prefix_data);
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
        // 全行が登録済みなら、論理座標と圧縮後の順位が一致する。
        let row_rank = if self.x_coordinates.len() == self.height {
            row
        } else {
            self.x_coordinates
                .binary_search(&row)
                .expect("point is not registered")
        };
        let mut node = self.row_size + row_rank;
        let layout = self.layouts[node];
        let coordinates =
            &self.y_coordinates[layout.coord_start..layout.coord_start + layout.coord_len];
        let col_rank = coordinates
            .binary_search(&col)
            .expect("point is not registered");
        self.prefix_data = None;
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
        if self.x_coordinates.len() == self.height {
            return self.inner_get(self.row_size + row, col);
        }
        match self.x_coordinates.binary_search(&row) {
            Ok(rank) => self.inner_get(self.row_size + rank, col),
            Err(_) => M::id(),
        }
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
    /// 登録点数を `K` とすると、時間 `O(log²(K + 1))`、追加領域 `O(1)`。
    pub fn fold(&self, rows: impl RangeBounds<usize>, columns: impl RangeBounds<usize>) -> M::S {
        let (top, bottom) = super::range_bounds::normalize(rows, self.height, "row");
        let (left, right) = super::range_bounds::normalize(columns, self.width, "column");
        if top == bottom || left == right {
            return M::id();
        }

        // 全行が登録済みなら、行端点の座標探索を省ける。
        let (top_rank, bottom_rank) = if self.x_coordinates.len() == self.height {
            (top, bottom)
        } else {
            (
                self.x_coordinates.partition_point(|&x| x < top),
                self.x_coordinates.partition_point(|&x| x < bottom),
            )
        };
        if top_rank == bottom_rank {
            return M::id();
        }
        let root = self.layouts[1];
        let root_coordinates =
            &self.y_coordinates[root.coord_start..root.coord_start + root.coord_len];
        let (left_rank, right_rank) = if let Some(ranks) = &self.root_ranks {
            (ranks[left] as usize, ranks[right] as usize)
        } else {
            (
                root_coordinates.partition_point(|&col| col < left),
                if right == self.width {
                    root.coord_len
                } else {
                    root_coordinates.partition_point(|&col| col < right)
                },
            )
        };
        if left_rank == right_rank {
            return M::id();
        }
        self.fold_ranked(
            1,
            (0, self.row_size),
            (top_rank, bottom_rank),
            (left_rank, right_rank),
        )
    }

    /// 同じ行範囲について、列の先頭からの二つの集約を同時に返す。
    ///
    /// `prepare_prefix_folds` 済みなら行方向の探索を共有する。未準備の
    /// 場合も通常の矩形集約を二回行い、同じ結果を返す。
    ///
    /// # Args
    /// - `rows` - 行範囲。`top..bottom`、`top..=bottom`、`..` などを指定できる。
    /// - `first_right`, `second_right` - 二つの列方向接頭辞の右端。
    ///
    /// # Returns
    /// 指定順に二つの接頭辞矩形の集約値を返す。
    ///
    /// # Panics
    /// 行範囲または列の右端が論理上の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// 登録点数を `K` とすると、事前計算済みの場合は `O(log(K + 1))`、
    /// 未準備の場合は `O(log²(K + 1))` 時間。
    pub fn fold_prefix_pair(
        &self,
        rows: impl RangeBounds<usize>,
        first_right: usize,
        second_right: usize,
    ) -> (M::S, M::S) {
        let (top, bottom) = super::range_bounds::normalize(rows, self.height, "row");
        assert!(
            first_right <= self.width && second_right <= self.width,
            "column range out of bounds"
        );
        let Some(prefix_data) = &self.prefix_data else {
            return (
                self.fold(top..bottom, 0..first_right),
                self.fold(top..bottom, 0..second_right),
            );
        };
        if top == bottom {
            return (M::id(), M::id());
        }
        let (top_rank, bottom_rank) = if self.x_coordinates.len() == self.height {
            (top, bottom)
        } else {
            (
                self.x_coordinates.partition_point(|&x| x < top),
                self.x_coordinates.partition_point(|&x| x < bottom),
            )
        };
        if top_rank == bottom_rank {
            return (M::id(), M::id());
        }
        let root = self.layouts[1];
        let coordinates = &self.y_coordinates[root.coord_start..root.coord_start + root.coord_len];
        let (first_rank, second_rank) = if let Some(ranks) = &self.root_ranks {
            (ranks[first_right] as usize, ranks[second_right] as usize)
        } else {
            (
                coordinates.partition_point(|&col| col < first_right),
                coordinates.partition_point(|&col| col < second_right),
            )
        };
        self.fold_prefix_pair_ranked(
            1,
            (0, self.row_size),
            (top_rank, bottom_rank),
            (first_rank, second_rank),
            prefix_data,
        )
    }

    /// 列順位を左右の子へ引き継ぎ、二つの接頭辞を行節点ごとに集約する。
    #[inline(always)]
    fn fold_prefix_pair_ranked(
        &self,
        node: usize,
        (row_begin, row_end): (usize, usize),
        (top_rank, bottom_rank): (usize, usize),
        (first_rank, second_rank): (usize, usize),
        prefix_data: &[M::S],
    ) -> (M::S, M::S) {
        if top_rank <= row_begin && row_end <= bottom_rank {
            let layout = self.layouts[node];
            let first = if first_rank == 0 {
                M::id()
            } else {
                prefix_data[layout.coord_start + first_rank - 1].clone()
            };
            let second = if second_rank == 0 {
                M::id()
            } else {
                prefix_data[layout.coord_start + second_rank - 1].clone()
            };
            return (first, second);
        }
        let mid = (row_begin + row_end) / 2;
        let layout = self.layouts[node];
        let first_map = self.child_ranks[layout.rank_start + first_rank];
        let second_map = self.child_ranks[layout.rank_start + second_rank];
        if bottom_rank <= mid {
            self.fold_prefix_pair_ranked(
                node * 2,
                (row_begin, mid),
                (top_rank, bottom_rank),
                (first_map as u32 as usize, second_map as u32 as usize),
                prefix_data,
            )
        } else if top_rank >= mid {
            self.fold_prefix_pair_ranked(
                node * 2 + 1,
                (mid, row_end),
                (top_rank, bottom_rank),
                ((first_map >> 32) as usize, (second_map >> 32) as usize),
                prefix_data,
            )
        } else {
            let (first_left, second_left) = self.fold_prefix_pair_ranked(
                node * 2,
                (row_begin, mid),
                (top_rank, bottom_rank),
                (first_map as u32 as usize, second_map as u32 as usize),
                prefix_data,
            );
            let (first_right, second_right) = self.fold_prefix_pair_ranked(
                node * 2 + 1,
                (mid, row_end),
                (top_rank, bottom_rank),
                ((first_map >> 32) as usize, (second_map >> 32) as usize),
                prefix_data,
            );
            (
                M::op(&first_left, &first_right),
                M::op(&second_left, &second_right),
            )
        }
    }

    /// 親の列順位を子へ写しながら、指定した行範囲を分解して集約する。
    #[inline(always)]
    fn fold_ranked(
        &self,
        node: usize,
        (row_begin, row_end): (usize, usize),
        (top_rank, bottom_rank): (usize, usize),
        (left_rank, right_rank): (usize, usize),
    ) -> M::S {
        if top_rank <= row_begin && row_end <= bottom_rank {
            return self.inner_fold_ranked(node, left_rank, right_rank);
        }
        let mid = (row_begin + row_end) / 2;
        let layout = self.layouts[node];
        let left_map = self.child_ranks[layout.rank_start + left_rank];
        let right_map = self.child_ranks[layout.rank_start + right_rank];
        if bottom_rank <= mid {
            self.fold_ranked(
                node * 2,
                (row_begin, mid),
                (top_rank, bottom_rank),
                (left_map as u32 as usize, right_map as u32 as usize),
            )
        } else if top_rank >= mid {
            self.fold_ranked(
                node * 2 + 1,
                (mid, row_end),
                (top_rank, bottom_rank),
                ((left_map >> 32) as usize, (right_map >> 32) as usize),
            )
        } else {
            let left_result = self.fold_ranked(
                node * 2,
                (row_begin, mid),
                (top_rank, bottom_rank),
                (left_map as u32 as usize, right_map as u32 as usize),
            );
            let right_result = self.fold_ranked(
                node * 2 + 1,
                (mid, row_end),
                (top_rank, bottom_rank),
                ((left_map >> 32) as usize, (right_map >> 32) as usize),
            );
            M::op(&left_result, &right_result)
        }
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

    /// 行節点内で、登録列座標の順位による半開区間を集約する。
    #[inline(always)]
    fn inner_fold_ranked(&self, node: usize, left_rank: usize, right_rank: usize) -> M::S {
        let layout = self.layouts[node];
        // 事前計算済みの接頭辞は、行節点ごとに 1 回の参照で得られる。
        if left_rank == 0 {
            if right_rank == 0 {
                return M::id();
            }
            if let Some(prefix_data) = &self.prefix_data {
                return prefix_data[layout.coord_start + right_rank - 1].clone();
            }
            if right_rank == layout.coord_len {
                return self.data[layout.data_start + 1].clone();
            }
        }

        let mut l = layout.size + left_rank;
        let mut r = layout.size + right_rank;
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
            assert_eq!(10, sut.fold(0..3, 0..5));
            assert_eq!(7, sut.fold(1..3, 2..5));
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
            assert_eq!(20, sut.fold(0..10, 0..10));
            assert_eq!(18, sut.fold(0..10, 8..9));
            assert_eq!(2, sut.fold(2..9, 0..8));
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
                                assert_eq!(expected, sut.fold(top..bottom, left..right));
                            }
                        }
                    }
                }
            }
        }
    }

    /// 構築後の接頭辞集約と、更新時のキャッシュ無効化を確認する。
    mod prefix_cache {
        use super::*;

        /// Scenario: 接頭辞二本の同時計算がすべての行範囲で素朴な和に一致する。
        /// - Given: 登録列が行ごとに異なる 5 × 7 の木がある。
        /// - When: 構築して接頭辞を準備する。
        /// - Then: 二本の接頭辞と任意の矩形集約が素朴な和に一致する。
        #[test]
        fn matches_naive_prefix_pairs_and_rectangles() {
            // Given
            let points = [(0, 1), (0, 5), (2, 0), (2, 4), (3, 5), (4, 2)];
            let mut sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(5, 7, points);
            let mut values = [[0_i64; 7]; 5];
            for (index, (row, col)) in points.into_iter().enumerate() {
                let value = index as i64 - 2;
                sut.set((row, col), value);
                values[row][col] = value;
            }
            // When
            sut.build();
            sut.prepare_prefix_folds();
            // Then
            for top in 0..=5 {
                for bottom in top..=5 {
                    for first_right in 0..=7 {
                        for second_right in 0..=7 {
                            let first = values[top..bottom]
                                .iter()
                                .flat_map(|row| &row[..first_right])
                                .sum::<i64>();
                            let second = values[top..bottom]
                                .iter()
                                .flat_map(|row| &row[..second_right])
                                .sum::<i64>();
                            assert_eq!(
                                (first, second),
                                sut.fold_prefix_pair(top..bottom, first_right, second_right)
                            );
                            let (left, right) = if first_right <= second_right {
                                (first_right, second_right)
                            } else {
                                (second_right, first_right)
                            };
                            assert_eq!(
                                values[top..bottom]
                                    .iter()
                                    .flat_map(|row| &row[left..right])
                                    .sum::<i64>(),
                                sut.fold(top..bottom, left..right)
                            );
                        }
                    }
                }
            }
        }

        /// Scenario: 点更新後も接頭辞二本の同時計算が現在値を返す。
        /// - Given: 構築済みの木で接頭辞を準備する。
        /// - When: 登録点を更新する。
        /// - Then: 古いキャッシュを参照せず、現在値を返す。
        #[test]
        fn invalidates_after_update() {
            // Given
            let mut sut = SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
                4,
                8,
                [(0, 1), (2, 3), (3, 6)],
            );
            sut.set((0, 1), 2);
            sut.set((2, 3), 5);
            sut.build();
            sut.prepare_prefix_folds();
            // When
            sut.update((2, 3), 11);
            // Then
            assert_eq!((13, 2), sut.fold_prefix_pair(0..4, 8, 3));
            assert_eq!(11, sut.fold(1..4, 3..4));
            sut.set((3, 6), 4);
            sut.build();
            sut.prepare_prefix_folds();
            assert_eq!((17, 2), sut.fold_prefix_pair(0..4, 8, 3));
            assert_eq!((17, 2), sut.fold_prefix_pair(..=3, 8, 3));
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
            let result = sut.fold(0..100, 0..200);
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

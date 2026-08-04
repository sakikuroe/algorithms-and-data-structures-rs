//! ダブリング (二分累乗) による、根付き木の最近共通祖先 (LCA) の計算を提供する
//! モジュールである。
//!
//! 各頂点について「2^k 個上の祖先」を表すテーブルを、k を1ずつ増やしながら
//! 前計算しておく。問い合わせの際は、まず両頂点の深さを揃え、次に共通祖先の
//! 手前まで二分探索的に登ることで、O(log V) で LCA を求める。

use std::collections;

use super::graph;

/// LCA 計算の結果を保持する。
pub struct Lca {
    /// `depth[v]` は、根から頂点 `v` までの距離 (辺数)。
    depth: Vec<usize>,
    /// `ancestor[k][v]` は、頂点 `v` から根の方向へ `2^k` 個進んだ祖先。
    /// 存在しない (根を超える) 場合は `None`。
    ancestor: Vec<Vec<Option<usize>>>,
}

impl Lca {
    /// 根から頂点 `v` までの深さ (辺数) を返す。
    ///
    /// # Args
    /// - `v`: 深さを求める頂点。
    ///
    /// # Returns
    /// `usize`: 根から `v` までの深さ。根自身の深さは0。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn depth(&self, v: usize) -> usize {
        self.depth[v]
    }

    /// 頂点 `v` から根の方向へ `k` 個進んだ祖先を返す。
    ///
    /// # Args
    /// - `v`: 起点となる頂点。
    /// - `k`: 根の方向へ進む歩数。
    ///
    /// # Returns
    /// `Option<usize>`: `k` 個上の祖先。根を超えてしまう (`k` が `v` の深さを
    /// 超える) 場合は `None`。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 -> 1 -> 2 の単純パス。
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(1, 2, ());
    ///
    /// let lca = g.lca(0);
    /// assert_eq!(Some(0), lca.kth_ancestor(2, 2));
    /// assert_eq!(None, lca.kth_ancestor(2, 3));
    /// ```
    pub fn kth_ancestor(&self, v: usize, k: usize) -> Option<usize> {
        if k > self.depth[v] {
            return None;
        }

        let mut v = v;
        // k を2進展開し、立っているビットに対応する幅だけ祖先テーブルを辿る。
        for (level, table) in self.ancestor.iter().enumerate() {
            if (k >> level) & 1 == 1 {
                v = table[v].expect("ancestor table must cover depth[v] levels");
            }
        }
        Some(v)
    }

    /// 頂点 `u` と `v` の最近共通祖先 (LCA) を返す。
    ///
    /// # Args
    /// - `u`: 1つ目の頂点。
    /// - `v`: 2つ目の頂点。
    ///
    /// # Returns
    /// `usize`: `u` と `v` の最近共通祖先。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 を根とし、0 の子が 1 と 2、1 の子が 3 である木。
    /// let mut g = Graph::new(4);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(0, 2, ());
    /// g.add_edge(1, 3, ());
    ///
    /// let lca = g.lca(0);
    /// assert_eq!(0, lca.lca(2, 3));
    /// assert_eq!(1, lca.lca(1, 3));
    /// ```
    pub fn lca(&self, u: usize, v: usize) -> usize {
        let (mut u, mut v) = if self.depth[u] >= self.depth[v] {
            (u, v)
        } else {
            (v, u)
        };

        // 深い方 (u) を、浅い方 (v) と同じ深さまで根の方向へ引き上げる。
        u = self
            .kth_ancestor(u, self.depth[u] - self.depth[v])
            .expect("depth difference must be within u's depth");
        if u == v {
            return u;
        }

        // 大きい歩幅から順に、u と v の祖先が一致しない範囲まで2頂点を
        // 同時に引き上げる。これにより、LCA の1つ手前まで一気に近付ける。
        for level in (0..self.ancestor.len()).rev() {
            if self.ancestor[level][u] != self.ancestor[level][v] {
                u = self.ancestor[level][u].unwrap();
                v = self.ancestor[level][v].unwrap();
            }
        }

        self.ancestor[0][u].unwrap()
    }

    /// 頂点 `u` と `v` の間の距離 (最短路の辺数) を返す。
    ///
    /// # Args
    /// - `u`: 1つ目の頂点。
    /// - `v`: 2つ目の頂点。
    ///
    /// # Returns
    /// `usize`: `u` と `v` の間の距離。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn distance(&self, u: usize, v: usize) -> usize {
        let w = self.lca(u, v);
        self.depth[u] + self.depth[v] - 2 * self.depth[w]
    }
}

impl<T> graph::Graph<T> {
    /// 頂点 `root` を根として、ダブリングによる LCA 計算の前処理を行う。
    ///
    /// # Args
    /// - `root`: 根とする頂点。`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `Lca`: 各頂点の深さと、LCA を求めるための祖先テーブル。
    ///
    /// # Constraints
    /// - グラフは `root` を根とする木 (連結、かつ辺数が頂点数-1) でなければ
    ///   ならない。無向辺は [`Graph::add_undirected_edge`] で張ったものを
    ///   想定する。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V log V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    /// g.add_undirected_edge(1, 3, ());
    ///
    /// let lca = g.lca(0);
    /// assert_eq!(1, lca.lca(1, 3));
    /// ```
    pub fn lca(&self, root: usize) -> Lca {
        let n = self.vertex_count();

        // BFS により、根からの深さと直接の親を確定させる。
        const UNVISITED: usize = usize::MAX;
        let mut depth = vec![UNVISITED; n];
        let mut parent = vec![None; n];
        depth[root] = 0;

        let mut queue = collections::VecDeque::new();
        queue.push_back(root);
        while let Some(u) = queue.pop_front() {
            for (v, _) in self.edges(u) {
                if depth[v] == UNVISITED {
                    depth[v] = depth[u] + 1;
                    parent[v] = Some(u);
                    queue.push_back(v);
                }
            }
        }

        // 木の深さの最大値 (高々 n-1) を2進数で表すのに必要な桁数だけ、
        // 祖先テーブルの段数を用意する。
        let mut height = 1;
        while (1_usize << height) < n {
            height += 1;
        }

        let mut ancestor = vec![vec![None; n]; height];
        ancestor[0] = parent;
        for level in 1..height {
            for v in 0..n {
                ancestor[level][v] = ancestor[level - 1][v].and_then(|p| ancestor[level - 1][p]);
            }
        }

        Lca { depth, ancestor }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 0 を根とし、0-1, 0-2, 1-3, 1-4 の辺を持つ木。
    ///
    /// ```text
    ///       0
    ///      / \
    ///     1   2
    ///    / \
    ///   3   4
    /// ```
    fn create_tree() -> graph::Graph<()> {
        let mut g = graph::Graph::new(5);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(0, 2, ());
        g.add_undirected_edge(1, 3, ());
        g.add_undirected_edge(1, 4, ());
        g
    }

    // lca (Graph::lca が返す Lca) のテスト: depth・kth_ancestor・lca・distance
    // の戻り値を検証する。
    mod lca {
        use super::*;

        /// Scenario: 各頂点の深さが、根からの辺数と一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: 各頂点の深さが期待通りになる。
        #[test]
        fn returns_depth_as_hop_count_from_root() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.depth(1));
            assert_eq!(2, result.depth(3));
        }

        /// Scenario: 共通の祖先を持つ葉同士の LCA は、その共通祖先になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点3と頂点4 (ともに頂点1の子) の LCA を求める。
        /// - Then: 頂点1が返る。
        #[test]
        fn returns_common_parent_for_sibling_leaves() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(1, result.lca(3, 4));
        }

        /// Scenario: 祖先と子孫の LCA は、その祖先自身になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点1と、その子孫である頂点3の LCA を求める。
        /// - Then: 頂点1が返る。
        #[test]
        fn returns_ancestor_itself_when_one_is_ancestor_of_other() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(1, result.lca(1, 3));
        }

        /// Scenario: 異なる部分木に属する頂点同士の LCA は、根になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点3 (頂点1の部分木) と頂点2 (根の直接の子) の LCA を求める。
        /// - Then: 根 (頂点0) が返る。
        #[test]
        fn returns_root_for_vertices_in_different_subtrees() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(0, result.lca(3, 2));
        }

        /// Scenario: 同じ頂点同士の LCA は、その頂点自身になる (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点3自身との LCA を求める。
        /// - Then: 頂点3が返る。
        #[test]
        fn returns_itself_for_same_vertex_pair() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(3, result.lca(3, 3));
        }

        /// Scenario: 距離は、両頂点の深さと LCA の深さから正しく求まる。
        /// - Given: 上記の木がある。
        /// - When: 頂点3と頂点4の距離を求める。
        /// - Then: 2 (3->1->4) になる。
        #[test]
        fn computes_distance_via_lca_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(2, result.distance(3, 4));
        }

        /// Scenario: k 個上の祖先が、根を超えない範囲で正しく求まる。
        /// - Given: 上記の木がある。
        /// - When: 頂点3から2個上の祖先を求める。
        /// - Then: 根 (頂点0) が返る。
        #[test]
        fn returns_kth_ancestor_within_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(Some(0), result.kth_ancestor(3, 2));
        }

        /// Scenario: 深さを超える歩数を指定すると `None` になる (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点3から3個上の祖先を求める (深さは2しかない)。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_k_exceeds_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(None, result.kth_ancestor(3, 3));
        }

        /// Scenario: 0個上の祖先は、自分自身になる (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点3から0個上の祖先を求める。
        /// - Then: 頂点3自身が返る。
        #[test]
        fn returns_itself_for_zero_steps() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(Some(3), result.kth_ancestor(3, 0));
        }

        /// Scenario: 頂点が1つだけの木でも正しく前処理できる (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: 深さは0であり、自身との LCA は自身になる。
        #[test]
        fn handles_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let result = sut.lca(0);
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(0, result.lca(0, 0));
        }
    }
}

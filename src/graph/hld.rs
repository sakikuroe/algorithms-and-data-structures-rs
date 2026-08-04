//! 重軽分解 (Heavy-Light Decomposition, HLD) を提供するモジュールである。
//!
//! 木を「重い辺」(各頂点から、部分木サイズが最大の子への辺) からなる連結な
//! パスの集まりに分解し、各頂点に、パスに沿って連続した番号 (`id`) を割り
//! 振る。この番号付けにより、任意の2頂点間のパスを O(log V) 本の連続した
//! 区間に分解できるため、区間に対応するデータ構造 (セグメント木など) と
//! 組み合わせることで、木上のパスクエリを高速に処理できる。
//!
//! 区間分解 ([`vertex_path_ranges`](Hld::vertex_path_ranges) /
//! [`edge_path_ranges`](Hld::edge_path_ranges)) は、区間同士および区間内部の
//! 結合順序を区別しない。そのため、可換なモノイド (総和・最小値・最大値・xor
//! など) によるパスクエリを想定しており、非可換な演算 (行列積など、経路の
//! 向きによって結果が変わる演算) には対応しない。

use std::mem;

use super::graph;

/// スタック上で処理待ちの頂点を表す。`Enter` は初訪問時、`Leave` はその頂点の
/// 子をすべて訪れ終えた後の処理を表す。
enum Frame {
    Enter(usize),
    Leave(usize),
}

/// HLD の結果を保持する。
pub struct Hld {
    /// `parent[v]` は、頂点 `v` の親。根では `None`。
    parent: Vec<Option<usize>>,
    /// `depth[v]` は、根から頂点 `v` までの距離 (辺数)。
    depth: Vec<usize>,
    /// `head[v]` は、頂点 `v` が属する連結パスのうち、最も浅い頂点。
    head: Vec<usize>,
    /// `id[v]` は、パスに沿って割り振られた頂点 `v` の番号 (`0..頂点数`)。
    /// 同じパスに属する頂点の間では、深さの昇順と一致する。
    id: Vec<usize>,
    /// `subtree_size[v]` は、頂点 `v` を根とする部分木のサイズ。
    subtree_size: Vec<usize>,
}

impl Hld {
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

    /// 頂点 `v` を根とする部分木のサイズを返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: `v` を根とする部分木に含まれる頂点数。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn subtree_size(&self, v: usize) -> usize {
        self.subtree_size[v]
    }

    /// 頂点 `v` に割り振られた番号を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: `0..頂点数` の範囲で割り振られた `v` の番号。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn vertex_id(&self, v: usize) -> usize {
        self.id[v]
    }

    /// 頂点 `v` を根とする部分木に対応する、番号の半開区間を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `(usize, usize)`: `v` を根とする部分木に属する頂点の番号が、
    /// ちょうど収まる半開区間 `[start, end)`。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 の子が 1, 2、1 の子が 3 である木。
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    /// g.add_undirected_edge(1, 3, ());
    ///
    /// let hld = g.hld(0);
    /// assert_eq!((0, 4), hld.subtree_range(0));
    /// assert_eq!(1, hld.subtree_range(2).1 - hld.subtree_range(2).0);
    /// ```
    pub fn subtree_range(&self, v: usize) -> (usize, usize) {
        (self.id[v], self.id[v] + self.subtree_size[v])
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
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
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
    /// let hld = g.hld(0);
    /// assert_eq!(1, hld.lca(1, 3));
    /// assert_eq!(0, hld.lca(2, 3));
    /// ```
    pub fn lca(&self, mut u: usize, mut v: usize) -> usize {
        // 異なるパスに属する間は、パスの根元がより深い側を、そのパスの
        // 親へ引き上げていく。同じパスに入れば、そのパス上を1歩も辿らずに
        // 残りの祖先関係が確定する。
        while self.head[u] != self.head[v] {
            if self.depth[self.head[u]] > self.depth[self.head[v]] {
                u = self.parent[self.head[u]].unwrap();
            } else {
                v = self.parent[self.head[v]].unwrap();
            }
        }
        if self.depth[u] <= self.depth[v] { u } else { v }
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
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn distance(&self, u: usize, v: usize) -> usize {
        let w = self.lca(u, v);
        self.depth[u] + self.depth[v] - 2 * self.depth[w]
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての頂点の番号を、
    /// 半開区間の列として返す。
    ///
    /// 頂点に値を持たせたパスクエリ (可換なモノイドに限る) で使う。
    ///
    /// # Args
    /// - `u`: パスの一方の端点。
    /// - `v`: パスのもう一方の端点。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パス上の頂点の番号をちょうど覆う、半開区間
    /// `[start, end)` の列。区間の個数は O(log V) 本になる。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    ///
    /// let hld = g.hld(0);
    /// let total_length: usize = hld
    ///     .vertex_path_ranges(0, 2)
    ///     .iter()
    ///     .map(|&(l, r)| r - l)
    ///     .sum();
    /// assert_eq!(3, total_length);
    /// ```
    pub fn vertex_path_ranges(&self, u: usize, v: usize) -> Vec<(usize, usize)> {
        self.path_ranges(u, v, false)
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての辺に対応する番号を、
    /// 半開区間の列として返す。
    ///
    /// 辺に値を持たせたパスクエリ (可換なモノイドに限る) で使う。各辺は、
    /// その子側の頂点の番号で表す。
    ///
    /// # Args
    /// - `u`: パスの一方の端点。
    /// - `v`: パスのもう一方の端点。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パス上の辺に対応する番号をちょうど覆う、
    /// 半開区間 `[start, end)` の列。区間の個数は O(log V) 本になる。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn edge_path_ranges(&self, u: usize, v: usize) -> Vec<(usize, usize)> {
        self.path_ranges(u, v, true)
    }

    /// [`vertex_path_ranges`](Self::vertex_path_ranges) と
    /// [`edge_path_ranges`](Self::edge_path_ranges) の共通部分を担う。
    ///
    /// # Args
    /// - `u`/`v`: パスの両端点。
    /// - `exclude_lca`: `true` の場合、最後にまとめる同一パス上の区間から
    ///   LCA 自身の番号を除く (辺クエリ用)。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パスを覆う半開区間の列。
    fn path_ranges(&self, mut u: usize, mut v: usize, exclude_lca: bool) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();

        // 異なるパスに属する間は、パスの根元がより深い側の区間を切り出し、
        // その親へ引き上げる。
        while self.head[u] != self.head[v] {
            if self.depth[self.head[u]] < self.depth[self.head[v]] {
                mem::swap(&mut u, &mut v);
            }
            ranges.push((self.id[self.head[u]], self.id[u] + 1));
            u = self.parent[self.head[u]].unwrap();
        }

        // 同じパスに入った時点で、浅い方が LCA になる。
        let (lca, deeper) = if self.id[u] <= self.id[v] {
            (u, v)
        } else {
            (v, u)
        };
        let start = if exclude_lca {
            self.id[lca] + 1
        } else {
            self.id[lca]
        };
        if start <= self.id[deeper] {
            ranges.push((start, self.id[deeper] + 1));
        }

        ranges
    }
}

impl<T> graph::Graph<T> {
    /// 頂点 `root` を根として、重軽分解 (HLD) の前処理を行う。
    ///
    /// # Args
    /// - `root`: 根とする頂点。`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `Hld`: 各頂点の深さ・所属パス・番号など、HLD の結果一式。
    ///
    /// # Constraints
    /// - グラフは `root` を根とする木でなければならない。無向辺は
    ///   [`Graph::add_undirected_edge`] で張ったものを想定する。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V)
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
    /// let hld = g.hld(0);
    /// assert_eq!(1, hld.lca(1, 3));
    /// ```
    pub fn hld(&self, root: usize) -> Hld {
        let n = self.vertex_count();

        // 第1段階: 深さ・親・部分木サイズを、帰りがけ順の反復 DFS で求める。
        let mut parent: Vec<Option<usize>> = vec![None; n];
        let mut depth = vec![0; n];
        let mut subtree_size = vec![1; n];
        let mut visited = vec![false; n];
        visited[root] = true;

        let mut stack = vec![Frame::Enter(root)];
        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Enter(u) => {
                    stack.push(Frame::Leave(u));
                    for (v, _) in self.edges(u) {
                        if !visited[v] {
                            visited[v] = true;
                            depth[v] = depth[u] + 1;
                            parent[v] = Some(u);
                            stack.push(Frame::Enter(v));
                        }
                    }
                }
                Frame::Leave(u) => {
                    for (v, _) in self.edges(u) {
                        if parent[v] == Some(u) {
                            subtree_size[u] += subtree_size[v];
                        }
                    }
                }
            }
        }

        // 第2段階: 各頂点について、部分木サイズが最大の子 (重い子) を求める。
        let mut heavy_child: Vec<Option<usize>> = vec![None; n];
        for (u, heavy_child) in heavy_child.iter_mut().enumerate() {
            for (v, _) in self.edges(u) {
                if parent[v] == Some(u) {
                    let is_heavier = match *heavy_child {
                        Some(current) => subtree_size[v] > subtree_size[current],
                        None => true,
                    };
                    if is_heavier {
                        *heavy_child = Some(v);
                    }
                }
            }
        }

        // 第3段階: 重い子を優先して辿ることで、パスに沿って連続した番号を
        // 割り振る。スタックが後入れ先出しであることを利用し、軽い子を先に、
        // 重い子を最後に積むことで、重い子を次に処理させ、同じパスの番号を
        // 連続させる。
        let mut id = vec![0; n];
        let mut head = vec![0; n];
        let mut counter = 0;
        let mut stack = vec![(root, root)];
        while let Some((u, chain_head)) = stack.pop() {
            id[u] = counter;
            head[u] = chain_head;
            counter += 1;

            for (v, _) in self.edges(u) {
                if parent[v] == Some(u) && Some(v) != heavy_child[u] {
                    // 軽い子は、そこを頭とする新しいパスを始める。
                    stack.push((v, v));
                }
            }
            if let Some(v) = heavy_child[u] {
                stack.push((v, chain_head));
            }
        }

        Hld {
            parent,
            depth,
            head,
            id,
            subtree_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 0 を根とし、0-1, 0-2, 1-3, 1-4, 3-5 の辺を持つ木。
    ///
    /// ```text
    ///         0
    ///        / \
    ///       1   2
    ///      / \
    ///     3   4
    ///     |
    ///     5
    /// ```
    /// 頂点1の部分木サイズが4 (1,3,4,5) と最大であるため、0からの重い辺は
    /// 0->1 になる。同様に1からの重い辺は3の部分木サイズが2 (3,5) で
    /// 4より大きいため 1->3 になる。
    fn create_tree() -> graph::Graph<()> {
        let mut g = graph::Graph::new(6);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(0, 2, ());
        g.add_undirected_edge(1, 3, ());
        g.add_undirected_edge(1, 4, ());
        g.add_undirected_edge(3, 5, ());
        g
    }

    // hld (Graph::hld が返す Hld) のテスト: 各アクセサの戻り値を検証する。
    mod hld {
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
            let result = sut.hld(0);
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.depth(1));
            assert_eq!(3, result.depth(5));
        }

        /// Scenario: 部分木サイズは、実際にその頂点の子孫の個数
        /// (自身を含む) と一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: 頂点1の部分木サイズは4 (1,3,4,5)、頂点3の部分木サイズは
        ///   2 (3,5) になる。
        #[test]
        fn returns_subtree_size_matching_descendant_count() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            // Then
            assert_eq!(4, result.subtree_size(1));
            assert_eq!(2, result.subtree_size(3));
            assert_eq!(1, result.subtree_size(4));
        }

        /// Scenario: 部分木に対応する区間の幅が、部分木サイズと一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点1の部分木の区間を求める。
        /// - Then: 区間の幅が、頂点1の部分木サイズ (4) と一致する。
        #[test]
        fn returns_subtree_range_with_width_matching_subtree_size() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            let (start, end) = result.subtree_range(1);
            // Then
            assert_eq!(4, end - start);
        }

        /// Scenario: 共通の祖先を持つ頂点同士の LCA は、その共通祖先になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点4と頂点5 (ともに頂点1の子孫) の LCA を求める。
        /// - Then: 頂点1が返る。
        #[test]
        fn returns_common_ancestor_as_lca() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            // Then
            assert_eq!(1, result.lca(4, 5));
        }

        /// Scenario: 異なる部分木に属する頂点同士の LCA は、根になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点5 (頂点1の部分木) と頂点2 (根の直接の子) の LCA を
        ///   求める。
        /// - Then: 根 (頂点0) が返る。
        #[test]
        fn returns_root_for_vertices_in_different_subtrees() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            // Then
            assert_eq!(0, result.lca(5, 2));
        }

        /// Scenario: 距離は、両頂点の深さと LCA の深さから正しく求まる。
        /// - Given: 上記の木がある。
        /// - When: 頂点5と頂点4の距離を求める。
        /// - Then: 3 (5->3->1->4) になる。
        #[test]
        fn computes_distance_via_lca_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            // Then
            assert_eq!(3, result.distance(5, 4));
        }

        /// Scenario: 頂点パスの区間分解は、パス上のすべての頂点をちょうど
        /// 1回ずつ覆う。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から頂点4へのパスの区間分解を求める
        ///   (パスは 5-3-1-4 の4頂点からなる)。
        /// - Then: 区間の幅の合計が4になり、実際に5,3,1,4の番号のみを覆う。
        #[test]
        fn vertex_path_ranges_cover_every_vertex_on_path_exactly_once() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            let ranges = result.vertex_path_ranges(5, 4);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(4, total_width);

            let mut covered = ranges
                .iter()
                .flat_map(|&(l, r)| l..r)
                .collect::<Vec<usize>>();
            covered.sort_unstable();
            let mut expected = [5, 3, 1, 4]
                .iter()
                .map(|&v| result.vertex_id(v))
                .collect::<Vec<usize>>();
            expected.sort_unstable();
            assert_eq!(expected, covered);
        }

        /// Scenario: 辺パスの区間分解は、頂点パスの区間分解よりちょうど1
        /// 少ない個数の番号を覆う (LCA 自身の番号を含まないため)。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から頂点4への辺の区間分解を求める
        ///   (パス上の辺は 5-3, 3-1, 1-4 の3本)。
        /// - Then: 区間の幅の合計が3になる。
        #[test]
        fn edge_path_ranges_exclude_lca_itself() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            let ranges = result.edge_path_ranges(5, 4);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(3, total_width);
        }

        /// Scenario: 一方が他方の祖先である場合、辺パスの区間分解は
        /// ちょうどその間の辺の本数を覆う (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点1 (祖先) から頂点5 (子孫) への辺の区間分解を求める
        ///   (パス上の辺は 1-3, 3-5 の2本)。
        /// - Then: 区間の幅の合計が2になる。
        #[test]
        fn edge_path_ranges_handle_ancestor_descendant_pair() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            let ranges = result.edge_path_ranges(1, 5);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(2, total_width);
        }

        /// Scenario: 同じ頂点同士のパスでは、頂点パスはその頂点自身のみを
        /// 覆い、辺パスは何も覆わない (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点3自身へのパスの区間分解を求める。
        /// - Then: 頂点パスの幅は1になり、辺パスの幅は0になる。
        #[test]
        fn handles_same_vertex_pair() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.hld(0);
            let vertex_width = result
                .vertex_path_ranges(3, 3)
                .iter()
                .map(|&(l, r)| r - l)
                .sum::<usize>();
            let edge_width = result
                .edge_path_ranges(3, 3)
                .iter()
                .map(|&(l, r)| r - l)
                .sum::<usize>();
            // Then
            assert_eq!(1, vertex_width);
            assert_eq!(0, edge_width);
        }

        /// Scenario: 頂点が1つだけの木でも正しく前処理できる (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: 深さは0、部分木サイズは1になる。
        #[test]
        fn handles_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let result = sut.hld(0);
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.subtree_size(0));
        }
    }
}

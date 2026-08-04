//! 橋・関節点の列挙を提供するモジュールである。
//!
//! いずれも lowlink (最小到達可能な行きがけ順) を用いた深さ優先探索により、
//! 共通の1回の探索でまとめて求める。無向グラフを対象とするため、
//! [`Graph::add_undirected_edge`](super::graph::Graph::add_undirected_edge) で
//! 両方向の辺を張ったグラフに対して使うことを想定する。
//!
//! 多重辺があっても正しく動作するよう、親頂点への辺のうち1本だけを探索木の
//! 親子関係として除外し、残りの辺は (同じ頂点へ向かうものであっても) 通常の
//! 後退辺として扱う。単純に「親と同じ頂点への辺をすべて除外する」実装では、
//! 親子間に多重辺がある場合に誤って橋・関節点を検出してしまう。

use super::graph;

/// 橋・関節点の列挙結果を保持する。
pub struct LowLink {
    /// 橋となる辺を `(小さい方の頂点, 大きい方の頂点)` の組として保持する。
    bridges: Vec<(usize, usize)>,
    /// 関節点となる頂点の一覧。
    articulation_points: Vec<usize>,
}

impl LowLink {
    /// 橋となる辺の一覧を返す。
    ///
    /// # Returns
    /// `&[(usize, usize)]`: 橋となる辺を `(小さい方の頂点, 大きい方の頂点)` の
    /// 組として並べた一覧。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn bridges(&self) -> &[(usize, usize)] {
        &self.bridges
    }

    /// 関節点となる頂点の一覧を返す。
    ///
    /// # Returns
    /// `&[usize]`: 関節点となる頂点の一覧。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn articulation_points(&self) -> &[usize] {
        &self.articulation_points
    }
}

/// スタック上で処理待ちの頂点を表す。`parent` は探索木上の親頂点である
/// (root では `None`)。
enum Frame {
    Enter { u: usize, parent: Option<usize> },
    Leave { u: usize, parent: Option<usize> },
}

impl<T> graph::Graph<T> {
    /// 橋・関節点をまとめて列挙する。
    ///
    /// # Returns
    /// `LowLink`: 橋となる辺の一覧と、関節点となる頂点の一覧。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1-2 の三角形に、橋 1-3 を介して 3-4-5 の三角形がぶら下がる形。
    /// let mut g = Graph::new(6);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    /// g.add_undirected_edge(2, 0, ());
    /// g.add_undirected_edge(1, 3, ());
    /// g.add_undirected_edge(3, 4, ());
    /// g.add_undirected_edge(4, 5, ());
    /// g.add_undirected_edge(5, 3, ());
    ///
    /// let low_link = g.low_link();
    /// assert_eq!(&[(1, 3)], low_link.bridges());
    /// assert!(low_link.articulation_points().contains(&1));
    /// assert!(low_link.articulation_points().contains(&3));
    /// ```
    pub fn low_link(&self) -> LowLink {
        let n = self.vertex_count();
        // order[v] は行きがけ順 (訪問順)。low[v] は、v とその子孫だけを使って
        // 後退辺を1本辿ることで到達できる、最小の行きがけ順である。
        let mut order: Vec<Option<usize>> = vec![None; n];
        let mut low = vec![0_usize; n];
        let mut is_articulation = vec![false; n];
        let mut bridges = Vec::new();
        let mut next_order = 0_usize;

        for root in 0..n {
            if order[root].is_some() {
                continue;
            }

            // root が関節点かどうかは「DFS 木における子が2本以上あるか」で
            // 決まるため、他の頂点とは別に数える。
            let mut root_children = 0_usize;
            let mut stack = vec![Frame::Enter {
                u: root,
                parent: None,
            }];

            while let Some(frame) = stack.pop() {
                match frame {
                    Frame::Enter { u, parent } => {
                        if order[u].is_some() {
                            // u へ至る辺は複数回スタックに積まれることがある
                            // (別の頂点から見て、まだ未訪問だった時点で
                            // 積まれたものが後から出てくる場合)。すでに
                            // 訪問済みであれば、二重に訪問木のノードとして
                            // 扱わないよう読み捨てる。
                            continue;
                        }
                        order[u] = Some(next_order);
                        low[u] = next_order;
                        next_order += 1;
                        stack.push(Frame::Leave { u, parent });

                        // 親頂点への辺のうち、探索木を辿ってきた1本だけを
                        // 除外する。多重辺がある場合、残りは後退辺として扱う。
                        let mut skipped_parent_edge = false;
                        for (v, _) in self.edges(u) {
                            if !skipped_parent_edge && Some(v) == parent {
                                skipped_parent_edge = true;
                                continue;
                            }
                            match order[v] {
                                Some(ov) => {
                                    // v は u の祖先 (または u 自身) であり、
                                    // この辺は後退辺である。
                                    low[u] = low[u].min(ov);
                                }
                                None => {
                                    stack.push(Frame::Enter {
                                        u: v,
                                        parent: Some(u),
                                    });
                                }
                            }
                        }
                    }
                    Frame::Leave { u, parent } => {
                        let Some(p) = parent else { continue };
                        low[p] = low[p].min(low[u]);

                        // u が親を経由せずに、親より浅い頂点へ到達できない
                        // ならば、辺 (p, u) は橋である。
                        if low[u] > order[p].unwrap() {
                            let (a, b) = if p < u { (p, u) } else { (u, p) };
                            bridges.push((a, b));
                        }

                        if p == root {
                            root_children += 1;
                        } else if low[u] >= order[p].unwrap() {
                            // u が p より浅い頂点へ、p を経由せずに到達できない
                            // ならば、p を取り除くと u 側が切り離される。
                            is_articulation[p] = true;
                        }
                    }
                }
            }

            if root_children >= 2 {
                is_articulation[root] = true;
            }
        }

        let articulation_points = (0..n).filter(|&v| is_articulation[v]).collect();

        LowLink {
            bridges,
            articulation_points,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // low_link のテスト: 戻り値 (LowLink) が保持する橋・関節点を検証する。
    mod low_link {
        use super::*;

        /// Background: 0-1-2 の三角形に、橋 1-3 を介して 3-4-5 の三角形が
        /// ぶら下がる無向グラフ。
        fn create_two_triangles_graph() -> graph::Graph<()> {
            let mut g = graph::Graph::new(6);
            g.add_undirected_edge(0, 1, ());
            g.add_undirected_edge(1, 2, ());
            g.add_undirected_edge(2, 0, ());
            g.add_undirected_edge(1, 3, ());
            g.add_undirected_edge(3, 4, ());
            g.add_undirected_edge(4, 5, ());
            g.add_undirected_edge(5, 3, ());
            g
        }

        /// Scenario: 三角形どうしをつなぐ辺だけが橋になる。
        /// - Given: 2つの三角形を1本の辺 (1, 3) でつないだグラフがある。
        /// - When: 橋・関節点を求める。
        /// - Then: 橋は (1, 3) の1本だけである。
        #[test]
        fn finds_only_connecting_edge_as_bridge() {
            // Given
            let sut = create_two_triangles_graph();
            // When
            let result = sut.low_link();
            // Then
            assert_eq!(&[(1, 3)], result.bridges());
        }

        /// Scenario: 橋の両端点が関節点になる。
        /// - Given: 2つの三角形を1本の辺 (1, 3) でつないだグラフがある。
        /// - When: 橋・関節点を求める。
        /// - Then: 頂点 1 と頂点 3 が関節点として含まれる。
        #[test]
        fn finds_bridge_endpoints_as_articulation_points() {
            // Given
            let sut = create_two_triangles_graph();
            // When
            let result = sut.low_link();
            // Then
            assert!(result.articulation_points().contains(&1));
            assert!(result.articulation_points().contains(&3));
        }

        /// Scenario: 三角形の内部の頂点は関節点ではない。
        /// - Given: 2つの三角形を1本の辺 (1, 3) でつないだグラフがある。
        /// - When: 橋・関節点を求める。
        /// - Then: 三角形の内部の頂点 (0, 2, 4, 5) は関節点に含まれない。
        #[test]
        fn does_not_treat_triangle_interior_vertices_as_articulation_points() {
            // Given
            let sut = create_two_triangles_graph();
            // When
            let result = sut.low_link();
            // Then
            for v in [0, 2, 4, 5] {
                assert!(!result.articulation_points().contains(&v));
            }
        }

        /// Scenario: 単純な閉路には橋も関節点も存在しない (境界値)。
        /// - Given: 0-1-2-0 の3閉路がある。
        /// - When: 橋・関節点を求める。
        /// - Then: 橋・関節点はいずれも空である。
        #[test]
        fn finds_no_bridges_or_articulation_points_in_simple_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            // When
            let result = sut.low_link();
            // Then
            assert!(result.bridges().is_empty());
            assert!(result.articulation_points().is_empty());
        }

        /// Scenario: 単純パスでは、すべての辺が橋になり、両端点以外の頂点が
        /// 関節点になる。
        /// - Given: `0-1-2-3` の単純パスがある。
        /// - When: 橋・関節点を求める。
        /// - Then: 3本の辺すべてが橋であり、頂点 1, 2 が関節点になる。
        #[test]
        fn treats_every_edge_as_bridge_in_simple_path() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            // When
            let result = sut.low_link();
            // Then
            assert_eq!(3, result.bridges().len());
            assert!(result.articulation_points().contains(&1));
            assert!(result.articulation_points().contains(&2));
        }

        /// Scenario: 親子間に多重辺があると、その辺は橋にならない。
        /// - Given: `0-1` の間に2本の辺があるグラフがある。
        /// - When: 橋・関節点を求める。
        /// - Then: 橋は存在せず、頂点 0, 1 は関節点でもない。
        #[test]
        fn does_not_treat_multi_edge_between_parent_and_child_as_bridge() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(0, 1, ());
            // When
            let result = sut.low_link();
            // Then
            assert!(result.bridges().is_empty());
            assert!(result.articulation_points().is_empty());
        }
    }
}

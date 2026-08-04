//! 二部グラフ判定を提供するモジュールである。
//!
//! BFS により各頂点を2色で塗り分け、隣接する頂点が同じ色にならないかを調べる。
//! 無向グラフを対象とするため、[`Graph::add_undirected_edge`](super::graph::Graph::add_undirected_edge)
//! で両方向の辺を張ったグラフに対して使うことを想定する。連結でないグラフも
//! 扱える。

use super::graph;
use std::collections;

impl<T> graph::Graph<T> {
    /// 二部グラフかどうかを判定し、そうであれば頂点の2色彩色を返す。
    ///
    /// # Returns
    /// `Option<Vec<bool>>`: 二部グラフである場合、各頂点の色を `bool` で表した
    /// 彩色 (隣接する頂点は必ず異なる色になる)。二部グラフでない場合は `None`。
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
    /// // 0-1-2-3-0 の4閉路は二部グラフである。
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    /// g.add_undirected_edge(2, 3, ());
    /// g.add_undirected_edge(3, 0, ());
    ///
    /// let coloring = g.bipartite_coloring().unwrap();
    /// assert_ne!(coloring[0], coloring[1]);
    /// assert_eq!(coloring[0], coloring[2]);
    /// ```
    ///
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1-2-0 の3閉路 (奇閉路) は二部グラフではない。
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    /// g.add_undirected_edge(2, 0, ());
    ///
    /// assert!(g.bipartite_coloring().is_none());
    /// ```
    pub fn bipartite_coloring(&self) -> Option<Vec<bool>> {
        let n = self.vertex_count();
        let mut color: Vec<Option<bool>> = vec![None; n];

        for start in 0..n {
            if color[start].is_some() {
                continue;
            }

            color[start] = Some(false);
            let mut queue = collections::VecDeque::new();
            queue.push_back(start);

            while let Some(u) = queue.pop_front() {
                let cu = color[u].unwrap();
                for (v, _) in self.edges(u) {
                    match color[v] {
                        None => {
                            color[v] = Some(!cu);
                            queue.push_back(v);
                        }
                        Some(cv) if cv == cu => {
                            // 隣接する頂点が同じ色になってしまうため、奇閉路が
                            // 存在し二部グラフではない。
                            return None;
                        }
                        Some(_) => {}
                    }
                }
            }
        }

        Some(color.into_iter().map(|c| c.unwrap()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // bipartite_coloring のテスト: 戻り値そのものを検証する。
    mod bipartite_coloring {
        use super::*;

        /// Scenario: 偶閉路は二部グラフであり、隣接頂点が異なる色に塗られる。
        /// - Given: 0-1-2-3-0 の4閉路がある。
        /// - When: 二部グラフ判定を行う。
        /// - Then: 彩色が得られ、隣接する頂点はすべて異なる色である。
        #[test]
        fn colors_adjacent_vertices_differently_for_even_cycle() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            sut.add_undirected_edge(3, 0, ());
            // When
            let result = sut.bipartite_coloring();
            // Then
            let coloring = result.unwrap();
            assert_ne!(coloring[0], coloring[1]);
            assert_ne!(coloring[1], coloring[2]);
            assert_ne!(coloring[2], coloring[3]);
            assert_ne!(coloring[3], coloring[0]);
        }

        /// Scenario: 奇閉路は二部グラフではない。
        /// - Given: 0-1-2-0 の3閉路がある。
        /// - When: 二部グラフ判定を行う。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_odd_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            // When
            let result = sut.bipartite_coloring();
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 辺を持たないグラフは二部グラフである (境界値)。
        /// - Given: 3頂点、辺を持たないグラフがある。
        /// - When: 二部グラフ判定を行う。
        /// - Then: 彩色が得られる (どの色でもよい)。
        #[test]
        fn returns_some_for_graph_without_edges() {
            // Given
            let sut = graph::Graph::<()>::new(3);
            // When
            let result = sut.bipartite_coloring();
            // Then
            assert!(result.is_some());
        }

        /// Scenario: 連結でないグラフでも、各連結成分が独立に判定される。
        /// - Given: 偶閉路の成分と、孤立した頂点からなるグラフがある。
        /// - When: 二部グラフ判定を行う。
        /// - Then: 彩色が得られる。
        #[test]
        fn handles_disconnected_graph() {
            // Given
            let mut sut = graph::Graph::new(5);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            sut.add_undirected_edge(3, 0, ());
            // 頂点4は孤立している。
            // When
            let result = sut.bipartite_coloring();
            // Then
            assert!(result.is_some());
        }

        /// Scenario: 自己ループがあると二部グラフではない (境界値)。
        /// - Given: 頂点 0 に自己ループがあるグラフがある。
        /// - When: 二部グラフ判定を行う。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_self_loop() {
            // Given
            let mut sut = graph::Graph::new(1);
            sut.add_edge(0, 0, ());
            // When
            let result = sut.bipartite_coloring();
            // Then
            assert!(result.is_none());
        }
    }
}

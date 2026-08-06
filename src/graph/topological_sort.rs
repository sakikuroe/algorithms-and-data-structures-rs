//! トポロジカルソートを提供するモジュールである。
//!
//! Kahn のアルゴリズムを用いる。入次数が 0 の頂点から順にキューへ積んでいき、
//! 取り出した頂点から出る辺の先の入次数を減らす、という操作を繰り返す。
//! 最終的にすべての頂点を取り出せれば有向非巡回グラフ (DAG) であり、
//! そうでなければ閉路が存在する。

use super::graph;
use std::collections;

impl<T> graph::Graph<T> {
    /// トポロジカルソートを行う。
    ///
    /// # Returns
    /// `Option<Vec<usize>>`: 閉路が存在しない場合、トポロジカル順序に並んだ
    /// 頂点列 (辺 `u -> v` があれば、`u` は `v` より前に現れる) であり、
    /// 閉路が存在する場合は `None` である。
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
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(0, 2, ());
    /// g.add_edge(1, 2, ());
    ///
    /// assert_eq!(Some(vec![0, 1, 2]), g.topological_sort());
    /// ```
    ///
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 -> 1 -> 0 は閉路をなす。
    /// let mut g = Graph::new(2);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(1, 0, ());
    ///
    /// assert_eq!(None, g.topological_sort());
    /// ```
    pub fn topological_sort(&self) -> Option<Vec<usize>> {
        let n = self.vertex_count();

        // 各頂点の入次数を数える。
        let mut in_degree = vec![0_usize; n];
        for u in 0..n {
            for (v, _) in self.edges(u) {
                in_degree[v] += 1;
            }
        }

        // 入次数が 0 の頂点から順に取り出す。頂点番号の昇順で安定させるため、
        // 優先度キューではなく単純な FIFO キューを使う。
        let mut queue = collections::VecDeque::new();
        for (v, &d) in in_degree.iter().enumerate() {
            if d == 0 {
                queue.push_back(v);
            }
        }

        let mut order = Vec::with_capacity(n);
        while let Some(u) = queue.pop_front() {
            order.push(u);
            for (v, _) in self.edges(u) {
                in_degree[v] -= 1;
                if in_degree[v] == 0 {
                    queue.push_back(v);
                }
            }
        }

        // 取り出せた頂点数が全頂点数に満たない場合、閉路上の頂点が残っている。
        if order.len() == n { Some(order) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // topological_sort のテスト: 戻り値そのものを検証する。
    mod topological_sort {
        use super::*;

        /// Scenario: DAG では、すべての辺が前から後ろへ向くような順序が得られる。
        /// - Given: `0->1`, `0->2`, `1->2` の辺を持つ DAG がある。
        /// - When: トポロジカルソートを行う。
        /// - Then: `[0, 1, 2]` が返る。
        #[test]
        fn returns_order_respecting_all_edges_for_dag() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(0, 2, ());
            sut.add_edge(1, 2, ());
            // When
            let result = sut.topological_sort();
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result);
        }

        /// Scenario: 閉路が存在する場合は `None` が返る。
        /// - Given: `0->1->0` が閉路をなすグラフがある。
        /// - When: トポロジカルソートを行う。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_cycle_exists() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 0, ());
            // When
            let result = sut.topological_sort();
            // Then
            assert_eq!(None, result);
        }

        /// Scenario: 辺を持たないグラフでは、頂点番号の昇順がそのまま順序になる
        /// (境界値)。
        /// - Given: 3頂点、辺を持たないグラフがある。
        /// - When: トポロジカルソートを行う。
        /// - Then: `[0, 1, 2]` が返る。
        #[test]
        fn returns_ascending_order_for_graph_without_edges() {
            // Given
            let sut = graph::Graph::<()>::new(3);
            // When
            let result = sut.topological_sort();
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result);
        }

        /// Scenario: 一部の頂点だけが閉路をなす場合も、閉路として検出される。
        /// - Given: `0->1`, `1->2->1` (2 と 1 が閉路) の辺を持つグラフがある。
        /// - When: トポロジカルソートを行う。
        /// - Then: `None` が返る。
        #[test]
        fn detects_cycle_involving_only_some_vertices() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 1, ());
            // When
            let result = sut.topological_sort();
            // Then
            assert_eq!(None, result);
        }

        /// Scenario: 自己ループは閉路として検出される (境界値)。
        /// - Given: 頂点 0 に自己ループがあるグラフがある。
        /// - When: トポロジカルソートを行う。
        /// - Then: `None` が返る。
        #[test]
        fn detects_self_loop_as_cycle() {
            // Given
            let mut sut = graph::Graph::new(1);
            sut.add_edge(0, 0, ());
            // When
            let result = sut.topological_sort();
            // Then
            assert_eq!(None, result);
        }
    }
}

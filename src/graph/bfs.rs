//! 幅優先探索 (BFS) を提供するモジュールである。
//!
//! 辺のペイロード `T` には一切関与しないため、重みなしグラフ (`T = ()`) は
//! もちろん、重み付きグラフ (`T = u32` など) に対しても、辺の本数を距離として
//! 数える探索がそのまま使える。

use super::graph;
use std::collections;

/// BFS の結果を保持する。
pub struct Bfs {
    /// `dist[v]` は、最も近い始点から `v` までの距離 (辺数)。到達できない場合は `None`。
    dist: Vec<Option<usize>>,
    /// `prev[v]` は、始点から `v` への最短路上で `v` の直前に訪れた頂点。
    /// `v` が始点である場合、または到達できない場合は `None`。
    prev: Vec<Option<usize>>,
}

impl Bfs {
    /// 最も近い始点から頂点 `v` までの距離 (辺数) を返す。
    ///
    /// # Args
    /// - `v`: 距離を求める頂点
    ///
    /// # Returns
    /// `Option<usize>`: `v` までの距離であり、いずれの始点からも到達できない場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn distance(&self, v: usize) -> Option<usize> {
        self.dist[v]
    }

    /// いずれかの始点から頂点 `v` に至る最短路を1つ、始点から `v` へ向かう
    /// 順の頂点列として返す。
    ///
    /// # Args
    /// - `v`: 経路の終点とする頂点
    ///
    /// # Returns
    /// `Option<Vec<usize>>`: 始点から `v` に至る頂点列であり、`v` にいずれの始点からも
    /// 到達できない場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(経路長)
    pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
        // 到達不能な場合は経路も存在しない。
        self.dist[v]?;

        let mut path = vec![v];
        let mut cur = v;
        while let Some(p) = self.prev[cur] {
            path.push(p);
            cur = p;
        }
        path.reverse();

        Some(path)
    }
}

impl<T> graph::Graph<T> {
    /// 複数の始点からの幅優先探索を行う。
    ///
    /// # Args
    /// - `starts`: 探索の始点となる頂点の列であり、`0..vertex_count()` の範囲でなければ
    ///   ならない。
    ///
    /// # Returns
    /// `Bfs`: 各頂点までの距離 (辺数) と、経路復元に必要な情報を保持する。
    ///
    /// # Panics
    /// - `starts` に `0..vertex_count()` の範囲外の頂点が含まれる場合にパニックする。
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
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    ///
    /// let bfs = g.bfs(&[0]);
    /// assert_eq!(Some(2), bfs.distance(2));
    /// assert_eq!(None, bfs.distance(3));
    /// assert_eq!(Some(vec![0, 1, 2]), bfs.path_to(2));
    /// ```
    pub fn bfs(&self, starts: &[usize]) -> Bfs {
        let mut dist = vec![None; self.vertex_count()];
        let mut prev = vec![None; self.vertex_count()];
        let mut queue = collections::VecDeque::new();

        // すべての始点を距離 0 としてキューに積む。
        for &s in starts {
            if dist[s].is_none() {
                dist[s] = Some(0);
                queue.push_back(s);
            }
        }

        while let Some(u) = queue.pop_front() {
            for (v, _) in self.edges(u) {
                // 未訪問の頂点だけを、u からの距離 + 1 として確定させる。
                if dist[v].is_none() {
                    dist[v] = Some(dist[u].unwrap() + 1);
                    prev[v] = Some(u);
                    queue.push_back(v);
                }
            }
        }

        Bfs { dist, prev }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 0-1-2-3 の単純パスからなる無向グラフ。
    fn create_path_graph() -> graph::Graph<()> {
        let mut g = graph::Graph::new(4);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(1, 2, ());
        g.add_undirected_edge(2, 3, ());
        g
    }

    // bfs のテスト: 戻り値 (Bfs) が保持する距離・経路を検証する。
    mod bfs {
        use super::*;

        /// Scenario: 単純パスでは、始点からの距離が辺数と一致する。
        /// - Given: 0-1-2-3 の単純パスからなる無向グラフがある。
        /// - When: 頂点 0 を始点に BFS を行う。
        /// - Then: 各頂点までの距離が、始点からの辺数と一致する。
        #[test]
        fn returns_hop_count_as_distance_for_simple_path() {
            // Given
            let sut = create_path_graph();
            // When
            let result = sut.bfs(&[0]);
            // Then
            assert_eq!(Some(0), result.distance(0));
            assert_eq!(Some(1), result.distance(1));
            assert_eq!(Some(2), result.distance(2));
            assert_eq!(Some(3), result.distance(3));
        }

        /// Scenario: 到達できない頂点への距離は `None` になる。
        /// - Given: 孤立した頂点を含むグラフがある。
        /// - When: 孤立していない頂点を始点に BFS を行う。
        /// - Then: 孤立した頂点への距離は `None` になる。
        #[test]
        fn returns_none_for_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<()>::new(2);
            // When
            let result = sut.bfs(&[0]);
            // Then
            assert_eq!(None, result.distance(1));
        }

        /// Scenario: 複数始点の BFS では、各頂点は最も近い始点からの距離になる。
        /// - Given: 0-1-2-3 の単純パスからなる無向グラフがある。
        /// - When: 頂点 0 と頂点 3 を始点に BFS を行う。
        /// - Then: 中間の頂点 1, 2 は、それぞれ最も近い始点からの距離になる。
        #[test]
        fn returns_distance_from_nearest_start_for_multiple_starts() {
            // Given
            let sut = create_path_graph();
            // When
            let result = sut.bfs(&[0, 3]);
            // Then
            assert_eq!(Some(0), result.distance(0));
            assert_eq!(Some(1), result.distance(1));
            assert_eq!(Some(1), result.distance(2));
            assert_eq!(Some(0), result.distance(3));
        }

        /// Scenario: 始点自身への経路は、始点のみからなる長さ1の頂点列になる。
        /// - Given: 0-1-2-3 の単純パスからなる無向グラフがある。
        /// - When: 頂点 0 を始点に BFS を行い、頂点 0 への経路を求める。
        /// - Then: `[0]` が返る。
        #[test]
        fn returns_singleton_path_for_start_vertex() {
            // Given
            let sut = create_path_graph();
            // When
            let result = sut.bfs(&[0]);
            // Then
            assert_eq!(Some(vec![0]), result.path_to(0));
        }

        /// Scenario: 到達できない頂点への経路は `None` になる。
        /// - Given: 孤立した頂点を含むグラフがある。
        /// - When: 孤立していない頂点を始点に BFS を行い、孤立した頂点への経路を求める。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_path_for_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<()>::new(2);
            // When
            let result = sut.bfs(&[0]);
            // Then
            assert_eq!(None, result.path_to(1));
        }

        /// Scenario: 閉路があっても、各頂点への距離は最短のもので確定する。
        /// - Given: 0-1-2-0 の三角形と、2-3 の辺からなる無向グラフがある。
        /// - When: 頂点 0 を始点に BFS を行う。
        /// - Then: 頂点 3 への距離は 2 (0->2->3、または 0->1->2->3 のうち短い方) になる。
        #[test]
        fn returns_shortest_distance_even_with_cycle() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            sut.add_undirected_edge(2, 3, ());
            // When
            let result = sut.bfs(&[0]);
            // Then
            assert_eq!(Some(2), result.distance(3));
        }
    }
}

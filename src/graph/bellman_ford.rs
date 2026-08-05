//! Bellman-Ford 法による単一始点最短路を提供するモジュールである。
//!
//! Dijkstra 法と異なり負の重みを持つ辺を扱えるが、時間計算量は O(VE) と大きい。
//! 負閉路が存在する場合、その影響を受ける頂点への最短コストは定義できない
//! (いくらでも小さくできる) ため、そのことを検出できるようにしている。

use super::graph;
use std::collections;

/// Bellman-Ford 法の結果を保持する。
pub struct BellmanFord<W> {
    /// `dist[v]` は、最も近い始点から `v` までの最短コスト。到達できない場合、
    /// または負閉路の影響を受ける場合は `None`。
    dist: Vec<Option<W>>,
    /// `prev[v]` は、最短路上で `v` の直前に訪れた頂点。
    prev: Vec<Option<usize>>,
    /// `affected[v]` は、`v` への経路上に負閉路が存在し、最短コストが
    /// 定義できないかどうかを表す。
    affected: Vec<bool>,
}

impl<W: Copy> BellmanFord<W> {
    /// 最も近い始点から頂点 `v` までの最短コストを返す。
    ///
    /// # Args
    /// - `v`: コストを求める頂点
    ///
    /// # Returns
    /// `Option<W>`: `v` までの最短コストであり、いずれの始点からも到達できない
    /// 場合、または `v` が負閉路の影響を受ける場合は `None`
    /// ([`is_affected_by_negative_cycle`](Self::is_affected_by_negative_cycle) で
    /// 両者を区別できる)。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn distance(&self, v: usize) -> Option<W> {
        self.dist[v]
    }

    /// 頂点 `v` への経路上に負閉路が存在し、最短コストが定義できないかどうかを
    /// 返す。
    ///
    /// # Args
    /// - `v`: 判定したい頂点
    ///
    /// # Returns
    /// `bool`: 負閉路の影響を受ける場合は `true`
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn is_affected_by_negative_cycle(&self, v: usize) -> bool {
        self.affected[v]
    }

    /// いずれかの始点から頂点 `v` に至る最短路を1つ、始点から `v` へ向かう
    /// 順の頂点列として返す。
    ///
    /// # Args
    /// - `v`: 経路の終点とする頂点
    ///
    /// # Returns
    /// `Option<Vec<usize>>`: 始点から `v` に至る頂点列であり、`v` にいずれの
    /// 始点からも到達できない場合、または `v` が負閉路の影響を受ける場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(経路長)
    pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
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

impl<T: Copy + Ord + std::ops::Add<Output = T>> graph::Graph<T> {
    /// 辺のペイロード自体を重みとして扱い、Bellman-Ford 法で単一始点最短路を
    /// 求める。
    ///
    /// # Args
    /// - `starts`: `(始点, 初期コスト)` の組の列であり、同じ頂点が複数回現れた
    ///   場合、最も小さい初期コストが採用される。
    ///
    /// # Returns
    /// `BellmanFord<T>`: 各頂点までの最短コストと、負閉路の影響の有無、
    /// 経路復元に必要な情報
    ///
    /// # Panics
    /// - `starts` に `0..vertex_count()` の範囲外の頂点が含まれる場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, 5_i64);
    /// g.add_edge(1, 2, -8_i64);
    ///
    /// let bf = g.bellman_ford(&[(0, 0)]);
    /// assert_eq!(Some(-3), bf.distance(2));
    /// assert!(!bf.is_affected_by_negative_cycle(2));
    /// ```
    pub fn bellman_ford(&self, starts: &[(usize, T)]) -> BellmanFord<T> {
        self.bellman_ford_by(starts, |&w| w)
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、Bellman-Ford 法で
    /// 単一始点最短路を求める。
    ///
    /// # Args
    /// - `starts`: `(始点, 初期コスト)` の組の列であり、同じ頂点が複数回現れた
    ///   場合、最も小さい初期コストが採用される。
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す関数
    ///
    /// # Returns
    /// `BellmanFord<W>`: 各頂点までの最短コストと、負閉路の影響の有無、
    /// 経路復元に必要な情報
    ///
    /// # Panics
    /// - `starts` に `0..vertex_count()` の範囲外の頂点が含まれる場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V)
    pub fn bellman_ford_by<W, F>(&self, starts: &[(usize, W)], weight_of: F) -> BellmanFord<W>
    where
        W: Copy + Ord + std::ops::Add<Output = W>,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();
        let mut dist: Vec<Option<W>> = vec![None; n];
        let mut prev = vec![None; n];

        for &(s, cost) in starts {
            let is_better = match dist[s] {
                Some(d) => cost < d,
                None => true,
            };
            if is_better {
                dist[s] = Some(cost);
            }
        }

        // 最短路は高々 n-1 本の辺からなるため、全辺の緩和を n-1 回繰り返せば
        // 負閉路が無い限り確定する。緩和が起きなくなった時点で早期終了する。
        for _ in 0..n.saturating_sub(1) {
            let mut updated = false;
            for u in 0..n {
                let Some(du) = dist[u] else { continue };
                for (v, payload) in self.edges(u) {
                    let nd = du + weight_of(payload);
                    let is_better = match dist[v] {
                        Some(cur) => nd < cur,
                        None => true,
                    };
                    if is_better {
                        dist[v] = Some(nd);
                        prev[v] = Some(u);
                        updated = true;
                    }
                }
            }
            if !updated {
                break;
            }
        }

        // もう1回だけ緩和を試み、まだ更新できる頂点を負閉路の起点とする。
        // n-1 回で確定するのは負閉路が無い場合のみであるため、ここでまだ更新
        // できる頂点があれば、それは負閉路上、またはそこから到達可能である。
        let mut affected = vec![false; n];
        let mut queue = collections::VecDeque::new();
        for u in 0..n {
            let Some(du) = dist[u] else { continue };
            for (v, payload) in self.edges(u) {
                let nd = du + weight_of(payload);
                let is_better = match dist[v] {
                    Some(cur) => nd < cur,
                    None => true,
                };
                if is_better && !affected[v] {
                    affected[v] = true;
                    queue.push_back(v);
                }
            }
        }
        // 負閉路の影響を、そこから到達可能な頂点全体へ伝播させる。
        while let Some(u) = queue.pop_front() {
            for (v, _) in self.edges(u) {
                if !affected[v] {
                    affected[v] = true;
                    queue.push_back(v);
                }
            }
        }
        for v in 0..n {
            if affected[v] {
                dist[v] = None;
                prev[v] = None;
            }
        }

        BellmanFord {
            dist,
            prev,
            affected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // bellman_ford のテスト: 戻り値 (BellmanFord) が保持する最短コスト・経路を検証する。
    mod bellman_ford {
        use super::*;

        /// Scenario: 負の重みを含む辺があっても、最短コストを正しく求められる。
        /// - Given: `0->1->2` の経路上に負の重みを持つ辺があるグラフがある。
        /// - When: 頂点 0 を始点に Bellman-Ford 法を行う。
        /// - Then: 頂点 2 への最短コストは、各辺の重みの総和になる。
        #[test]
        fn computes_distance_with_negative_edge_weight() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, 5_i64);
            sut.add_edge(1, 2, -8_i64);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert_eq!(Some(-3), result.distance(2));
        }

        /// Scenario: 到達できない頂点への最短コストは `None` になる。
        /// - Given: 孤立した頂点を含むグラフがある。
        /// - When: 孤立していない頂点を始点に Bellman-Ford 法を行う。
        /// - Then: 孤立した頂点への最短コストは `None` になり、負閉路の影響でもない。
        #[test]
        fn returns_none_for_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<i64>::new(2);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert_eq!(None, result.distance(1));
            assert!(!result.is_affected_by_negative_cycle(1));
        }

        /// Scenario: 負閉路上の頂点は、負閉路の影響を受けると判定され、
        /// 最短コストは `None` になる。
        /// - Given: `0->1->0` が負閉路をなすグラフがある。
        /// - When: 頂点 0 を始点に Bellman-Ford 法を行う。
        /// - Then: 頂点 0, 1 はいずれも負閉路の影響を受けると判定される。
        #[test]
        fn detects_vertices_on_negative_cycle() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, -1_i64);
            sut.add_edge(1, 0, -1_i64);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert!(result.is_affected_by_negative_cycle(0));
            assert!(result.is_affected_by_negative_cycle(1));
            assert_eq!(None, result.distance(0));
            assert_eq!(None, result.distance(1));
        }

        /// Scenario: 負閉路から到達可能な頂点も、負閉路の影響を受けると判定される。
        /// - Given: `0->1->0` の負閉路と、そこから伸びる `1->2` の辺を持つグラフがある。
        /// - When: 頂点 0 を始点に Bellman-Ford 法を行う。
        /// - Then: 負閉路上にない頂点 2 も、影響を受けると判定される。
        #[test]
        fn propagates_negative_cycle_effect_to_reachable_vertices() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, -1_i64);
            sut.add_edge(1, 0, -1_i64);
            sut.add_edge(1, 2, 1_i64);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert!(result.is_affected_by_negative_cycle(2));
            assert_eq!(None, result.distance(2));
        }

        /// Scenario: 負閉路から到達できない頂点は、負閉路の影響を受けない。
        /// - Given: `0->1->0` の負閉路と、それとは無関係な孤立頂点 2 を持つグラフがある。
        /// - When: 頂点 0 を始点に Bellman-Ford 法を行う。
        /// - Then: 頂点 2 は影響を受けないが、そもそも到達できないため距離は `None` である。
        #[test]
        fn does_not_affect_vertex_unreachable_from_negative_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, -1_i64);
            sut.add_edge(1, 0, -1_i64);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert!(!result.is_affected_by_negative_cycle(2));
            assert_eq!(None, result.distance(2));
        }

        /// Scenario: 最短経路が経路として復元できる。
        /// - Given: `0->1->2` の経路 (5, -8) を持つグラフがある。
        /// - When: 頂点 0 を始点に Bellman-Ford 法を行い、頂点 2 への経路を求める。
        /// - Then: `[0, 1, 2]` が返る。
        #[test]
        fn returns_shortest_path() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, 5_i64);
            sut.add_edge(1, 2, -8_i64);
            // When
            let result = sut.bellman_ford(&[(0, 0)]);
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result.path_to(2));
        }
    }
}

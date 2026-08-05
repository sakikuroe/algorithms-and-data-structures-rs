//! Dijkstra 法による単一始点最短路を提供するモジュールである。
//!
//! 非負の重みを持つグラフを対象とする。始点は `(頂点, 初期コスト)` の組で
//! 受け取ることで、単位元 (加法の 0 に相当する値) を専用の trait や引数として
//! 扱う必要がなくなる。単一始点の場合は初期コストとして `0` を渡せばよく、
//! 複数始点でそれぞれ異なる初期コストを持たせる (仮想始点を張るテクニック)
//! ことも自然に表現できる。

use super::graph;
use std::{cmp, collections};

/// Dijkstra 法の結果を保持する。
pub struct Dijkstra<W> {
    /// `dist[v]` は、最も近い始点から `v` までの最短コスト。到達できない場合は
    /// `None`。
    dist: Vec<Option<W>>,
    /// `prev[v]` は、最短路上で `v` の直前に訪れた頂点。`v` が始点である場合、
    /// または到達できない場合は `None`。
    prev: Vec<Option<usize>>,
}

impl<W: Copy> Dijkstra<W> {
    /// 最も近い始点から頂点 `v` までの最短コストを返す。
    ///
    /// # Args
    /// - `v`: コストを求める頂点
    ///
    /// # Returns
    /// `Option<W>`: `v` までの最短コストであり、いずれの始点からも到達できない
    /// 場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn distance(&self, v: usize) -> Option<W> {
        self.dist[v]
    }

    /// いずれかの始点から頂点 `v` に至る最短路を1つ、始点から `v` へ向かう
    /// 順の頂点列として返す。
    ///
    /// # Args
    /// - `v`: 経路の終点とする頂点
    ///
    /// # Returns
    /// `Option<Vec<usize>>`: 始点から `v` に至る頂点列であり、`v` にいずれの始点
    /// からも到達できない場合は `None` である。
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
    /// 辺のペイロード自体を重みとして扱い、Dijkstra 法で単一始点最短路を求める。
    ///
    /// 重みの型 `T` を自分自身から取り出すだけでよい場合の簡易版であり、
    /// 重み以外の情報 (色・ラベルなど) も併せ持つペイロードから重みを取り出したい
    /// 場合は [`dijkstra_by`](Self::dijkstra_by) を使う。
    ///
    /// # Args
    /// - `starts`: `(始点, 初期コスト)` の組の列であり、同じ頂点が複数回現れた
    ///   場合、最も小さい初期コストが採用される。
    ///
    /// # Returns
    /// `Dijkstra<T>`: 各頂点までの最短コストと、経路復元に必要な情報
    ///
    /// # Panics
    /// - `starts` に `0..vertex_count()` の範囲外の頂点が含まれる場合にパニックする。
    ///
    /// # Constraints
    /// - すべての辺の重みが非負でなければならない。負の重みを含む場合、
    ///   [`bellman_ford`](Self::bellman_ford) を使う必要がある。
    ///
    /// # Complexity
    /// - 時間計算量: O((V + E) log V)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, 5_u32);
    /// g.add_edge(1, 2, 3_u32);
    /// g.add_edge(0, 2, 100_u32);
    ///
    /// let dijkstra = g.dijkstra(&[(0, 0)]);
    /// assert_eq!(Some(8), dijkstra.distance(2));
    /// assert_eq!(Some(vec![0, 1, 2]), dijkstra.path_to(2));
    /// ```
    pub fn dijkstra(&self, starts: &[(usize, T)]) -> Dijkstra<T> {
        self.dijkstra_by(starts, |&w| w)
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、Dijkstra 法で
    /// 単一始点最短路を求める。
    ///
    /// ペイロード `T` が重み以外の情報 (色・ラベル・元の入力での辺番号など) も
    /// 併せ持つ場合に使う。ペイロード自体が重みである場合は
    /// [`dijkstra`](Self::dijkstra) の方が簡潔に書ける。
    ///
    /// # Args
    /// - `starts`: `(始点, 初期コスト)` の組の列であり、同じ頂点が複数回現れた
    ///   場合、最も小さい初期コストが採用される。
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す関数
    ///
    /// # Returns
    /// `Dijkstra<W>`: 各頂点までの最短コストと、経路復元に必要な情報
    ///
    /// # Panics
    /// - `starts` に `0..vertex_count()` の範囲外の頂点が含まれる場合にパニックする。
    ///
    /// # Constraints
    /// - `weight_of` はすべての辺に対して非負の値を返さなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O((V + E) log V)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // ペイロードが (重み, 元の辺番号) の組であるケース。
    /// let mut g = Graph::new(2);
    /// g.add_edge(0, 1, (7_u32, 0_usize));
    ///
    /// let dijkstra = g.dijkstra_by(&[(0, 0_u32)], |&(w, _id)| w);
    /// assert_eq!(Some(7), dijkstra.distance(1));
    /// ```
    pub fn dijkstra_by<W, F>(&self, starts: &[(usize, W)], weight_of: F) -> Dijkstra<W>
    where
        W: Copy + Ord + std::ops::Add<Output = W>,
        F: Fn(&T) -> W,
    {
        let mut dist: Vec<Option<W>> = vec![None; self.vertex_count()];
        let mut prev = vec![None; self.vertex_count()];
        // (コスト, 頂点) の組を、コストが小さい順に取り出すための優先度キュー。
        // `BinaryHeap` は最大値を先頭に取るため、`Reverse` で反転させる。
        let mut heap = collections::BinaryHeap::new();

        for &(s, cost) in starts {
            let is_better = match dist[s] {
                Some(d) => cost < d,
                None => true,
            };
            if is_better {
                dist[s] = Some(cost);
                heap.push(cmp::Reverse((cost, s as u32)));
            }
        }

        while let Some(cmp::Reverse((d, u))) = heap.pop() {
            let u = u as usize;
            // 確定済みの最短コストより大きい古いエントリは読み捨てる。
            if dist[u] != Some(d) {
                continue;
            }

            for (v, payload) in self.edges(u) {
                let nd = d + weight_of(payload);
                let is_better = match dist[v] {
                    Some(cur) => nd < cur,
                    None => true,
                };
                if is_better {
                    dist[v] = Some(nd);
                    prev[v] = Some(u);
                    heap.push(cmp::Reverse((nd, v as u32)));
                }
            }
        }

        Dijkstra { dist, prev }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 0 -> 1 -> 2 の単純パスに、0 -> 2 への遠回りな直接辺を加えた
    /// 有向グラフ。
    fn create_graph() -> graph::Graph<u32> {
        let mut g = graph::Graph::new(3);
        g.add_edge(0, 1, 5);
        g.add_edge(1, 2, 3);
        g.add_edge(0, 2, 100);
        g
    }

    // dijkstra のテスト: 戻り値 (Dijkstra) が保持する最短コスト・経路を検証する。
    mod dijkstra {
        use super::*;

        /// Scenario: 直接辺より、中継を挟む経路の方が短い場合、短い方が採用される。
        /// - Given: `0->2` の直接辺 (100) と `0->1->2` の経路 (5+3=8) を持つグラフがある。
        /// - When: 頂点 0 を始点に Dijkstra 法を行う。
        /// - Then: 頂点 2 への最短コストは 8 になる。
        #[test]
        fn chooses_shorter_path_over_direct_edge() {
            // Given
            let sut = create_graph();
            // When
            let result = sut.dijkstra(&[(0, 0)]);
            // Then
            assert_eq!(Some(8), result.distance(2));
        }

        /// Scenario: 到達できない頂点への最短コストは `None` になる。
        /// - Given: 孤立した頂点を含むグラフがある。
        /// - When: 孤立していない頂点を始点に Dijkstra 法を行う。
        /// - Then: 孤立した頂点への最短コストは `None` になる。
        #[test]
        fn returns_none_for_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<u32>::new(2);
            // When
            let result = sut.dijkstra(&[(0, 0)]);
            // Then
            assert_eq!(None, result.distance(1));
        }

        /// Scenario: 複数始点では、それぞれ異なる初期コストが正しく考慮される。
        /// - Given: `0->1->2` の経路 (辺重み 5, 3) を持つグラフがある。
        /// - When: 頂点 0 (初期コスト 0) と頂点 2 (初期コスト 1) を始点にする。
        /// - Then: 頂点 2 への最短コストは、直接の初期コスト 1 が採用される
        ///   (0 経由の 8 より小さいため)。
        #[test]
        fn uses_given_initial_cost_per_start() {
            // Given
            let sut = create_graph();
            // When
            let result = sut.dijkstra(&[(0, 0), (2, 1)]);
            // Then
            assert_eq!(Some(1), result.distance(2));
        }

        /// Scenario: 最短経路が経路として復元できる。
        /// - Given: `0->2` の直接辺 (100) と `0->1->2` の経路 (8) を持つグラフがある。
        /// - When: 頂点 0 を始点に Dijkstra 法を行い、頂点 2 への経路を求める。
        /// - Then: `[0, 1, 2]` が返る (直接辺は使われない)。
        #[test]
        fn returns_shortest_path_not_direct_edge() {
            // Given
            let sut = create_graph();
            // When
            let result = sut.dijkstra(&[(0, 0)]);
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result.path_to(2));
        }

        /// Scenario: 重み 0 の辺のみからなるグラフでも正しく動作する (境界値)。
        /// - Given: すべての辺の重みが 0 であるグラフがある。
        /// - When: Dijkstra 法を行う。
        /// - Then: 到達可能な頂点への最短コストはすべて 0 になる。
        #[test]
        fn handles_all_zero_weights() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, 0_u32);
            // When
            let result = sut.dijkstra(&[(0, 0)]);
            // Then
            assert_eq!(Some(0), result.distance(1));
        }
    }

    // dijkstra_by のテスト: 戻り値 (Dijkstra) が保持する最短コストを検証する。
    mod dijkstra_by {
        use super::*;

        /// Scenario: ペイロードから射影した重みを使って最短コストを求められる。
        /// - Given: ペイロードが `(重み, 元の辺番号)` の組であるグラフがある。
        /// - When: 重みだけを取り出す関数を渡して Dijkstra 法を行う。
        /// - Then: 射影した重みに基づく最短コストが得られる。
        #[test]
        fn computes_distance_using_projected_weight() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, (7_u32, 0_usize));
            // When
            let result = sut.dijkstra_by(&[(0, 0_u32)], |&(w, _id)| w);
            // Then
            assert_eq!(Some(7), result.distance(1));
        }
    }
}

//! Johnson 法による全点対最短路を提供するモジュールである。
//!
//! 負の重みを持つ辺があっても (負閉路さえ無ければ) 扱える点は Bellman-Ford 法や
//! Floyd-Warshall 法と同じだが、疎なグラフ (辺数が頂点数の2乗より十分小さい)
//! では `O(V^3)` の Floyd-Warshall 法より高速に動作する。
//!
//! 手順は次の2段階からなる。まず、全頂点を始点とする Bellman-Ford 法により、
//! 各頂点の「ポテンシャル」を求める (これは、全頂点へ重み0の辺を張った仮想始点
//! からの最短距離に等しい)。次に、辺の重みをポテンシャルの差で補正して
//! 非負にした上で、補正後のグラフに対して全頂点から Dijkstra 法を行う。

use super::{dijkstra, graph};

/// Johnson 法の結果を保持する。
pub struct Johnson<W> {
    /// `per_source[u]` は、補正後の重みを用いて頂点 `u` を始点として行った
    /// Dijkstra 法の結果である。
    per_source: Vec<dijkstra::Dijkstra<W>>,
    /// `potential[v]` は、頂点 `v` のポテンシャルである。
    potential: Vec<W>,
}

impl<W: Copy + Ord + std::ops::Add<Output = W> + std::ops::Sub<Output = W>> Johnson<W> {
    /// 頂点 `u` から頂点 `v` までの最短コストを返す。
    ///
    /// # Args
    /// - `u`: 経路の始点
    /// - `v`: 経路の終点
    ///
    /// # Returns
    /// `Option<W>`: `u` から `v` までの最短コストであり、到達できない場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn distance(&self, u: usize, v: usize) -> Option<W> {
        // 補正後の重みで求めた距離から、ポテンシャルの分だけ差し引いて
        // 実際の重みに基づく距離へ戻す。
        self.per_source[u]
            .distance(v)
            .map(|d| d - self.potential[u] + self.potential[v])
    }

    /// 頂点 `u` から頂点 `v` に至る最短路を1つ、`u` から `v` へ向かう順の
    /// 頂点列として返す。
    ///
    /// 経路そのものは辺の重みの補正によらず変わらないため、Dijkstra 法の結果を
    /// そのまま使える。
    ///
    /// # Args
    /// - `u`: 経路の始点
    /// - `v`: 経路の終点
    ///
    /// # Returns
    /// `Option<Vec<usize>>`: `u` から `v` に至る頂点列であり、到達できない場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(経路長)
    pub fn path(&self, u: usize, v: usize) -> Option<Vec<usize>> {
        self.per_source[u].path_to(v)
    }
}

impl<T: Copy + Ord + std::ops::Add<Output = T> + std::ops::Sub<Output = T>> graph::Graph<T> {
    /// 辺のペイロード自体を重みとして扱い、Johnson 法で全点対最短路を求める。
    ///
    /// # Args
    /// - `zero`: 重みの加法単位元 (通常は `0`)
    ///
    /// # Returns
    /// `Option<Johnson<T>>`: 全点対最短路であり、負閉路が存在する場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE + V(V+E) log V)
    ///   - V は頂点数、E は辺数である。前半は全頂点始点の Bellman-Ford 法、
    ///     後半は全頂点始点の Dijkstra 法のコストである。
    /// - 空間計算量: O(V^2)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, 5_i64);
    /// g.add_edge(1, 2, -2_i64);
    ///
    /// let johnson = g.johnson(0).unwrap();
    /// assert_eq!(Some(3), johnson.distance(0, 2));
    /// ```
    pub fn johnson(&self, zero: T) -> Option<Johnson<T>> {
        self.johnson_by(zero, |&w| w)
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、Johnson 法で
    /// 全点対最短路を求める。
    ///
    /// # Args
    /// - `zero`: 重みの加法単位元 (通常は `0`)
    /// - `weight_of`: 辺のペイロードから、加算・減算・比較可能な重みを取り出す
    ///   関数。
    ///
    /// # Returns
    /// `Option<Johnson<W>>`: 全点対最短路であり、負閉路が存在する場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE + V(V+E) log V)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V^2)
    pub fn johnson_by<W, F>(&self, zero: W, weight_of: F) -> Option<Johnson<W>>
    where
        W: Copy + Ord + std::ops::Add<Output = W> + std::ops::Sub<Output = W>,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();

        // 全頂点へ重み0の辺を張った仮想始点からの最短距離は、全頂点を初期
        // コスト zero の始点とする多始点 Bellman-Ford 法と一致する。
        // potential[v] を、この仮想始点から v までの最短コストとして求める
        // (これを「ポテンシャル」と呼ぶ)。
        let starts = (0..n).map(|v| (v, zero)).collect::<Vec<(usize, W)>>();
        let bf = self.bellman_ford_by(&starts, &weight_of);
        // 負閉路があると、ポテンシャルそのものが定義できない
        // (Bellman-Ford 法の distance が None になる) ため、失敗として返す。
        if (0..n).any(|v| bf.is_affected_by_negative_cycle(v)) {
            return None;
        }
        let potential = (0..n).map(|v| bf.distance(v).unwrap()).collect::<Vec<W>>();

        // 辺の重みを w'(u,v) = w(u,v) + h(u) - h(v) と付け替える。負閉路が
        // 無ければ、この付け替えにより w' は必ず非負になる (Johnson 法の要点)。
        // 非負になれば、以降は Dijkstra 法がそのまま使える。
        let mut reweighted = graph::Graph::new(n);
        for u in 0..n {
            for (v, payload) in self.edges(u) {
                reweighted.add_edge(u, v, weight_of(payload) + potential[u] - potential[v]);
            }
        }

        // 補正後のグラフに対し、全頂点それぞれを始点とする Dijkstra 法を行い、
        // 全点対の最短路を求める。実際の重みに基づく距離への戻し方は
        // [`Johnson::distance`] を参照。
        let per_source = (0..n)
            .map(|s| reweighted.dijkstra(&[(s, zero)]))
            .collect::<Vec<dijkstra::Dijkstra<W>>>();

        Some(Johnson {
            per_source,
            potential,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // johnson のテスト: 戻り値 (Johnson) が保持する最短コスト・経路を検証する。
    mod johnson {
        use super::*;

        /// Scenario: 負の重みを含む辺があっても、最短コストを正しく求められる。
        /// - Given: `0->1->2` の経路上に負の重みを持つ辺があるグラフがある。
        /// - When: Johnson 法を行う。
        /// - Then: 頂点 0 から頂点 2 への最短コストは、各辺の重みの総和になる。
        #[test]
        fn computes_distance_with_negative_edge_weight() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, 5_i64);
            sut.add_edge(1, 2, -2_i64);
            // When
            let result = sut.johnson(0).unwrap();
            // Then
            assert_eq!(Some(3), result.distance(0, 2));
        }

        /// Scenario: 到達できない頂点対の最短コストは `None` になる。
        /// - Given: 有向辺 `0->1` のみを持つグラフがある。
        /// - When: Johnson 法を行う。
        /// - Then: 頂点 1 から頂点 0 への最短コストは `None` になる。
        #[test]
        fn returns_none_for_unreachable_pair() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, 1_i64);
            // When
            let result = sut.johnson(0).unwrap();
            // Then
            assert_eq!(None, result.distance(1, 0));
        }

        /// Scenario: 負閉路が存在する場合は `None` が返る。
        /// - Given: `0->1->0` が負閉路をなすグラフがある。
        /// - When: Johnson 法を行う。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_negative_cycle_exists() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, -1_i64);
            sut.add_edge(1, 0, -1_i64);
            // When
            let result = sut.johnson(0);
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 最短経路が経路として復元できる。
        /// - Given: `0->1->2` の経路 (5, -2) を持つグラフがある。
        /// - When: Johnson 法を行い、頂点 0 から頂点 2 への経路を求める。
        /// - Then: `[0, 1, 2]` が返る。
        #[test]
        fn returns_shortest_path() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, 5_i64);
            sut.add_edge(1, 2, -2_i64);
            // When
            let result = sut.johnson(0).unwrap();
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result.path(0, 2));
        }

        /// Scenario: 自分自身への距離は単位元になる (境界値)。
        /// - Given: 辺を持たないグラフがある。
        /// - When: Johnson 法を行う。
        /// - Then: 各頂点から自分自身への距離は指定した単位元 (0) になる。
        #[test]
        fn returns_zero_for_self_distance() {
            // Given
            let sut = graph::Graph::<i64>::new(3);
            // When
            let result = sut.johnson(0).unwrap();
            // Then
            assert_eq!(Some(0), result.distance(0, 0));
        }
    }
}

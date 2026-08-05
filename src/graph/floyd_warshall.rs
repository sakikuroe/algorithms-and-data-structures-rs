//! Floyd-Warshall 法による全点対最短路を提供するモジュールである。
//!
//! `O(V^3)` の動的計画法であり、負の重みを持つ辺があっても (負閉路さえ無ければ)
//! そのまま使える。密なグラフ (辺数が頂点数の2乗に近い) では、全頂点から
//! Dijkstra 法を行うよりも高速である。

use super::graph;

/// Floyd-Warshall 法の結果を保持する。
pub struct FloydWarshall<W> {
    /// `dist[u][v]` は、`u` から `v` までの最短コスト。到達できない場合は `None`。
    dist: Vec<Vec<Option<W>>>,
    /// `on_negative_cycle[v]` は、`v` が負閉路上にあるかどうかを表す。
    on_negative_cycle: Vec<bool>,
}

impl<W: Copy> FloydWarshall<W> {
    /// 頂点 `u` から頂点 `v` までの最短コストを返す。
    ///
    /// # Args
    /// - `u`: 経路の始点
    /// - `v`: 経路の終点
    ///
    /// # Returns
    /// `Option<W>`: `u` から `v` までの最短コストであり、到達できない場合、
    /// または経路上に負閉路が存在し最短コストが定義できない場合は `None`
    /// ([`is_on_negative_cycle`](Self::is_on_negative_cycle) で判定できる)。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn distance(&self, u: usize, v: usize) -> Option<W> {
        self.dist[u][v]
    }

    /// 頂点 `v` が負閉路上にあるかどうかを返す。
    ///
    /// # Args
    /// - `v`: 判定したい頂点
    ///
    /// # Returns
    /// `bool`: `v` を経由する負閉路が存在する場合は `true`
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn is_on_negative_cycle(&self, v: usize) -> bool {
        self.on_negative_cycle[v]
    }
}

impl<T: Copy + Ord + std::ops::Add<Output = T>> graph::Graph<T> {
    /// 辺のペイロード自体を重みとして扱い、Floyd-Warshall 法で全点対最短路を
    /// 求める。
    ///
    /// # Args
    /// - `zero`: 重みの加法単位元 (自分自身への距離として使う値であり、通常は
    ///   `0`)
    ///
    /// # Returns
    /// `FloydWarshall<T>`: すべての頂点対についての最短コストと、負閉路の
    /// 有無
    ///
    /// # Complexity
    /// - 時間計算量: O(V^3)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V^2)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, 5_i64);
    /// g.add_edge(1, 2, 3_i64);
    ///
    /// let fw = g.floyd_warshall(0);
    /// assert_eq!(Some(8), fw.distance(0, 2));
    /// assert_eq!(Some(0), fw.distance(0, 0));
    /// assert_eq!(None, fw.distance(2, 0));
    /// ```
    pub fn floyd_warshall(&self, zero: T) -> FloydWarshall<T> {
        self.floyd_warshall_by(zero, |&w| w)
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、
    /// Floyd-Warshall 法で全点対最短路を求める。
    ///
    /// # Args
    /// - `zero`: 重みの加法単位元 (自分自身への距離として使う値であり、通常は
    ///   `0`)
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す関数
    ///
    /// # Returns
    /// `FloydWarshall<W>`: すべての頂点対についての最短コストと、負閉路の
    /// 有無
    ///
    /// # Complexity
    /// - 時間計算量: O(V^3)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V^2)
    pub fn floyd_warshall_by<W, F>(&self, zero: W, weight_of: F) -> FloydWarshall<W>
    where
        W: Copy + Ord + std::ops::Add<Output = W>,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();
        let mut dist = vec![vec![None; n]; n];
        for (u, row) in dist.iter_mut().enumerate() {
            // 自分自身への距離は単位元 (0 に相当する値) とする。自己ループの
            // 重みが負であっても、後段の負閉路検出で拾われる。
            row[u] = Some(zero);
        }
        for u in 0..n {
            for (v, payload) in self.edges(u) {
                let w = weight_of(payload);
                let is_better = match dist[u][v] {
                    Some(cur) => w < cur,
                    None => true,
                };
                if is_better {
                    dist[u][v] = Some(w);
                }
            }
        }

        for k in 0..n {
            for i in 0..n {
                let Some(dik) = dist[i][k] else { continue };
                for j in 0..n {
                    let Some(dkj) = dist[k][j] else { continue };
                    let via_k = dik + dkj;
                    let is_better = match dist[i][j] {
                        Some(cur) => via_k < cur,
                        None => true,
                    };
                    if is_better {
                        dist[i][j] = Some(via_k);
                    }
                }
            }
        }

        // dist[v][v] < zero (自己ループ由来の初期値より小さくなった) 頂点は
        // 負閉路上にある。そこからさらに到達可能な頂点にも影響が伝播する。
        let mut on_negative_cycle = vec![false; n];
        for v in 0..n {
            if let Some(d) = dist[v][v]
                && d < zero
            {
                on_negative_cycle[v] = true;
            }
        }
        for k in 0..n {
            if !on_negative_cycle[k] {
                continue;
            }
            for i in 0..n {
                if dist[i][k].is_some() {
                    on_negative_cycle[i] = true;
                }
            }
            for j in 0..n {
                if dist[k][j].is_some() {
                    on_negative_cycle[j] = true;
                }
            }
        }
        for i in 0..n {
            for j in 0..n {
                if on_negative_cycle[i] || on_negative_cycle[j] {
                    dist[i][j] = None;
                }
            }
        }

        FloydWarshall {
            dist,
            on_negative_cycle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // floyd_warshall のテスト: 戻り値 (FloydWarshall) が保持する最短コストを検証する。
    mod floyd_warshall {
        use super::*;

        /// Scenario: 中継を挟む経路のコストが正しく求められる。
        /// - Given: `0->1->2` の経路 (重み 5, 3) を持つグラフがある。
        /// - When: Floyd-Warshall 法を行う。
        /// - Then: 頂点 0 から頂点 2 への最短コストは 8 になる。
        #[test]
        fn computes_distance_through_intermediate_vertex() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, 5_i64);
            sut.add_edge(1, 2, 3_i64);
            // When
            let result = sut.floyd_warshall(0);
            // Then
            assert_eq!(Some(8), result.distance(0, 2));
        }

        /// Scenario: 自分自身への距離は単位元になる (境界値)。
        /// - Given: 辺を持たないグラフがある。
        /// - When: Floyd-Warshall 法を行う。
        /// - Then: 各頂点から自分自身への距離は指定した単位元 (0) になる。
        #[test]
        fn returns_zero_for_self_distance() {
            // Given
            let sut = graph::Graph::<i64>::new(3);
            // When
            let result = sut.floyd_warshall(0);
            // Then
            assert_eq!(Some(0), result.distance(0, 0));
        }

        /// Scenario: 到達できない頂点対の最短コストは `None` になる。
        /// - Given: 有向辺 `0->1` のみを持つグラフがある。
        /// - When: Floyd-Warshall 法を行う。
        /// - Then: 頂点 1 から頂点 0 への最短コストは `None` になる。
        #[test]
        fn returns_none_for_unreachable_pair() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, 1_i64);
            // When
            let result = sut.floyd_warshall(0);
            // Then
            assert_eq!(None, result.distance(1, 0));
        }

        /// Scenario: 負閉路上の頂点は、負閉路の影響を受けると判定され、
        /// 最短コストは `None` になる。
        /// - Given: `0->1->0` が負閉路をなすグラフがある。
        /// - When: Floyd-Warshall 法を行う。
        /// - Then: 頂点 0, 1 はいずれも負閉路上にあると判定される。
        #[test]
        fn detects_vertices_on_negative_cycle() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, -1_i64);
            sut.add_edge(1, 0, -1_i64);
            // When
            let result = sut.floyd_warshall(0);
            // Then
            assert!(result.is_on_negative_cycle(0));
            assert!(result.is_on_negative_cycle(1));
            assert_eq!(None, result.distance(0, 0));
        }
    }
}

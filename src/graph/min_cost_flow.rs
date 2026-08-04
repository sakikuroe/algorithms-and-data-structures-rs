//! ポテンシャルを用いた successive shortest path 法による、最小費用流の計算を
//! 提供するモジュールである。
//!
//! 「始点から終点への、残余グラフ上でコストが最小となる経路」を1本ずつ選んで
//! 流すことを、それ以上流せなくなるまで繰り返す。これを最大流まで続けた結果が、
//! 最大流量における最小コストになる (各増加が最小コストの経路を使うため、
//! 流量を増やすたびに単調に最適性が保たれる)。
//!
//! 負のコストを持つ辺があってもダイクストラ法をそのまま使えるように、頂点
//! ごとのポテンシャル `h` を保持し、コストを「還元コスト」
//! `cost(u, v) + h[u] - h[v]` に置き換えて扱う。最初のポテンシャルは、負の
//! コストを含みうる元のグラフに対して1回だけベルマン-フォード法を行うことで
//! 求める。増加のたびにポテンシャルを最短距離だけ更新すると、還元コストは
//! 常に非負に保たれることが知られており、2回目以降の増加はダイクストラ法
//! だけで済む。

use std::{cmp, collections};

use super::min_cost_flow_graph::{self, MinCostFlowGraph};

impl<T: min_cost_flow_graph::FlowValue> MinCostFlowGraph<T> {
    /// successive shortest path 法により、頂点 `s` から頂点 `t` への
    /// 最大流量における最小コストを求める。
    ///
    /// 呼び出し後、内部の残余容量は最小費用最大流を実現した状態に更新されて
    /// いる。各辺の流量は [`get_edge`](Self::get_edge) で参照できる。
    ///
    /// # Args
    /// - `s`: 始点。`0..vertex_count()` の範囲でなければならない。
    /// - `t`: 終点。`0..vertex_count()` の範囲でなければならず、`s` と異なって
    ///   いなければならない。
    ///
    /// # Returns
    /// `(T, T)`: `(s から t への最大流量, その流量を実現する最小コスト)`。
    ///
    /// # Panics
    /// - `s == t` の場合にパニックする。
    /// - `s`/`t` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Constraints
    /// - `s` から、残余容量が正の辺のみを辿って到達できる範囲に、コストが
    ///   負の閉路があってはならない。存在すると、コストがいくらでも小さく
    ///   できてしまい、最小コストが定義できない。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE + F E log V)
    ///   - V は頂点数、E は辺数、F は最大流量である。ポテンシャルの初期化に
    ///     ベルマン-フォード法で O(VE)、以降の各増加にダイクストラ法で
    ///     O(E log V) かかる。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::min_cost_flow_graph::MinCostFlowGraph;
    ///
    /// // 0 -> 1 -> 2 の経路 (容量2, コスト3) と、0 -> 2 への直接の安い経路
    /// // (容量1, コスト1) を持つグラフ。
    /// let mut g = MinCostFlowGraph::<i64>::new(3);
    /// g.add_edge(0, 1, 2, 3);
    /// g.add_edge(1, 2, 2, 3);
    /// g.add_edge(0, 2, 1, 1);
    ///
    /// let (flow, cost) = g.min_cost_flow(0, 2);
    /// assert_eq!(3, flow);
    /// // 安い直接経路 (1単位, コスト1) をまず使い、残り2単位は
    /// // 0->1->2 (単位あたりコスト6) を使う。
    /// assert_eq!(1 * 1 + 2 * 6, cost);
    /// ```
    pub fn min_cost_flow(&mut self, s: usize, t: usize) -> (T, T) {
        debug_assert!(s != t, "s and t must be different vertices");

        let n = self.vertex_count();
        let mut potential = self.bellman_ford_potential(s);

        let mut flow = T::ZERO;
        let mut cost = T::ZERO;

        loop {
            let ShortestPath { dist, prev_edge } = self.dijkstra_with_potential(s, &potential);
            if dist[t].is_none() {
                // これ以上、t へ到達できる増加パスが存在しない。
                break;
            }

            // ポテンシャルを最短距離だけ更新する。これにより、次回の還元
            // コストも非負に保たれる。到達できなかった頂点のポテンシャルは
            // 以降のパス探索で使われることがないため、更新しなくてよい。
            for v in 0..n {
                if let Some(d) = dist[v] {
                    potential[v] = potential[v] + d;
                }
            }

            // s から t への経路をたどり、ボトルネックとなる残余容量を求める。
            let mut bottleneck = T::MAX;
            let mut v = t;
            while let Some((u, idx)) = prev_edge[v] {
                bottleneck = bottleneck.min(self.graph[u][idx].cap);
                v = u;
            }

            // 経路上の各辺に、ボトルネック分の流量を反映する。
            let mut v = t;
            while let Some((u, idx)) = prev_edge[v] {
                let edge_to = self.graph[u][idx].to;
                let edge_rev = self.graph[u][idx].rev;
                self.graph[u][idx].cap = self.graph[u][idx].cap - bottleneck;
                self.graph[edge_to][edge_rev].cap = self.graph[edge_to][edge_rev].cap + bottleneck;
                v = u;
            }

            flow = flow + bottleneck;
            // s のポテンシャルは常に0に保たれるため、更新後の t のポテンシャル
            // が、そのまま今回の経路の (元のコストでの) 総和に一致する。
            cost = cost + bottleneck * potential[t];
        }

        (flow, cost)
    }

    /// 負のコストを含みうる元のグラフに対して、始点 `s` からの最短距離を
    /// ベルマン-フォード法で求め、初期ポテンシャルとする。
    ///
    /// 呼び出し時点ではまだ流量が押し出されていないため、逆辺の残余容量は
    /// すべて0であり、実質的に元の辺のみを辿ることになる。
    ///
    /// # Args
    /// - `s`: 始点。
    ///
    /// # Returns
    /// `Vec<T>`: `result[v]` は `s` から `v` への最短距離。到達できない頂点は
    /// 以降のポテンシャルとして使われることがないため、`T::ZERO` を割り当てる。
    ///
    /// # Complexity
    /// - 時間計算量: O(VE)
    fn bellman_ford_potential(&self, s: usize) -> Vec<T> {
        let n = self.vertex_count();
        let mut dist = vec![None; n];
        dist[s] = Some(T::ZERO);

        // n-1 回の緩和で、負の閉路が存在しない限り最短距離が確定する。
        for _ in 0..n {
            let mut updated = false;
            for u in 0..n {
                let Some(du) = dist[u] else {
                    continue;
                };
                for edge in &self.graph[u] {
                    if edge.cap > T::ZERO {
                        let nd = du + edge.cost;
                        let is_better = match dist[edge.to] {
                            Some(cur) => nd < cur,
                            None => true,
                        };
                        if is_better {
                            dist[edge.to] = Some(nd);
                            updated = true;
                        }
                    }
                }
            }
            if !updated {
                break;
            }
        }

        dist.into_iter().map(|d| d.unwrap_or(T::ZERO)).collect()
    }

    /// 頂点ごとのポテンシャル `potential` を用いた還元コストのもとで、
    /// 始点 `s` からの最短距離をダイクストラ法で求める。
    ///
    /// # Args
    /// - `s`: 始点。
    /// - `potential`: 頂点ごとのポテンシャル。還元コスト
    ///   `cost(u, v) + potential[u] - potential[v]` が非負になっている必要が
    ///   ある。
    ///
    /// # Returns
    /// `ShortestPath<T>`: 還元コストでの最短距離と、経路復元に必要な情報。
    ///
    /// # Complexity
    /// - 時間計算量: O(E log V)
    fn dijkstra_with_potential(&self, s: usize, potential: &[T]) -> ShortestPath<T> {
        let n = self.vertex_count();
        let mut dist: Vec<Option<T>> = vec![None; n];
        let mut prev_edge = vec![None; n];
        dist[s] = Some(T::ZERO);

        let mut heap = collections::BinaryHeap::new();
        heap.push(cmp::Reverse((T::ZERO, s)));

        while let Some(cmp::Reverse((d, u))) = heap.pop() {
            if dist[u] != Some(d) {
                continue;
            }

            for (idx, edge) in self.graph[u].iter().enumerate() {
                if edge.cap > T::ZERO {
                    let reduced_cost = edge.cost + potential[u] - potential[edge.to];
                    let nd = d + reduced_cost;
                    let is_better = match dist[edge.to] {
                        Some(cur) => nd < cur,
                        None => true,
                    };
                    if is_better {
                        dist[edge.to] = Some(nd);
                        prev_edge[edge.to] = Some((u, idx));
                        heap.push(cmp::Reverse((nd, edge.to)));
                    }
                }
            }
        }

        ShortestPath { dist, prev_edge }
    }
}

/// [`dijkstra_with_potential`](MinCostFlowGraph::dijkstra_with_potential) の
/// 結果を保持する。
struct ShortestPath<T> {
    /// `dist[v]` は、還元コストのもとでの `s` から `v` への最短距離。
    /// 到達できない場合は `None`。
    dist: Vec<Option<T>>,
    /// `prev_edge[v]` は、最短路上で `v` の直前に使った
    /// `(頂点, その頂点の隣接リスト内での辺の添字)`。`s` 自身、または到達
    /// できない頂点では `None`。
    prev_edge: Vec<Option<(usize, usize)>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // min_cost_flow のテスト: 戻り値そのものと、呼び出し後に get_edge で
    // 観測できる各辺の流量を検証する。
    mod min_cost_flow {
        use super::*;

        /// Scenario: 安い直接経路を優先しつつ、残りは高い経路で補って
        /// 流し切る。
        /// - Given: `0->1->2` (容量2, 単位コスト3ずつ) と、`0->2` への
        ///   直接の安い経路 (容量1, 単位コスト1) がある。
        /// - When: 0 から 2 への最小費用流を求める。
        /// - Then: 最大流量は3になり、コストは 1*1 + 2*6 = 13 になる。
        #[test]
        fn prefers_cheaper_route_before_using_expensive_one() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(3);
            sut.add_edge(0, 1, 2, 3);
            sut.add_edge(1, 2, 2, 3);
            sut.add_edge(0, 2, 1, 1);
            // When
            let (flow, cost) = sut.min_cost_flow(0, 2);
            // Then
            assert_eq!(3, flow);
            assert_eq!(13, cost);
        }

        /// Scenario: 負のコストを持つ辺があっても正しく処理できる。
        /// - Given: `0->1` (容量1, コスト -5) と `1->2` (容量1, コスト 2) の
        ///   経路がある。
        /// - When: 0 から 2 への最小費用流を求める。
        /// - Then: 最大流量は1になり、コストは -3 (= -5 + 2) になる。
        #[test]
        fn handles_negative_cost_edge() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(3);
            sut.add_edge(0, 1, 1, -5);
            sut.add_edge(1, 2, 1, 2);
            // When
            let (flow, cost) = sut.min_cost_flow(0, 2);
            // Then
            assert_eq!(1, flow);
            assert_eq!(-3, cost);
        }

        /// Scenario: 終点に到達できないグラフでは、流量・コストともに0に
        /// なる (境界値)。
        /// - Given: 0 から 1 への辺を持たない、孤立した頂点を含むグラフが
        ///   ある。
        /// - When: 0 から 1 への最小費用流を求める。
        /// - Then: 流量・コストともに0になる。
        #[test]
        fn returns_zero_when_sink_is_unreachable() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            // When
            let (flow, cost) = sut.min_cost_flow(0, 1);
            // Then
            assert_eq!(0, flow);
            assert_eq!(0, cost);
        }

        /// Scenario: 複数の経路のうち、単位コストが最も低い経路から順に
        /// 使われる。
        /// - Given: `0->1` の並列な2辺 (容量1・単位コスト1, 容量1・単位
        ///   コスト5) がある。
        /// - When: 0 から 1 への最小費用流を求める。
        /// - Then: 流量は2 (両方の辺を使い切る) になり、コストは
        ///   1 + 5 = 6 になる。
        #[test]
        fn uses_cheaper_parallel_edge_first() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            sut.add_edge(0, 1, 1, 5);
            sut.add_edge(0, 1, 1, 1);
            // When
            let (flow, cost) = sut.min_cost_flow(0, 1);
            // Then
            assert_eq!(2, flow);
            assert_eq!(6, cost);
        }

        /// Scenario: 呼び出し後、各辺の流量を get_edge で参照できる。
        /// - Given: `0->1` (容量3, コスト1) の単純な辺がある。
        /// - When: 0 から 1 への最小費用流を求める。
        /// - Then: `get_edge` で流量が3 (容量いっぱい) と観測できる。
        #[test]
        fn exposes_flow_per_edge_via_get_edge() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            let e01 = sut.add_edge(0, 1, 3, 1);
            // When
            sut.min_cost_flow(0, 1);
            // Then
            assert_eq!((0, 1, 3, 1, 3), sut.get_edge(e01));
        }

        /// Scenario: s と t が同じ頂点であれば、パニックする (異常系)。
        /// - Given: 2頂点のグラフがある。
        /// - When: 同じ頂点を始点・終点として最小費用流を求めようとする。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "s and t must be different vertices")]
        fn panics_when_source_and_sink_are_the_same() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            // When, Then (panic)
            sut.min_cost_flow(0, 0);
        }
    }
}

//! Dinic 法による最大流の計算を提供するモジュールである。
//!
//! 各フェーズで、始点からの BFS 距離 (レベル) をもとに、レベルが単調増加する
//! 辺のみからなる「レベルグラフ」を構成し、そのグラフ上で飽和するまで
//! (増加パスが尽きるまで) フローを流す「ブロッキングフロー」を1回求める。
//! これを、始点から終点へ到達できなくなるまで繰り返す。ブロッキングフローの
//! 探索では、各頂点で「次に試す辺」を指すポインタ (`iter`) を使い回すことで、
//! 同じフェーズ内で無駄な辺の再訪問を避ける (この最適化により、1フェーズあたり
//! O(VE) で抑えられる)。
//!
//! 深さ優先探索は、頂点数が大きい場合の再帰によるスタックオーバーフローを
//! 避けるため、明示的なスタックを使って反復的に行う。

use std::collections;

use super::flow_graph::{self, FlowGraph};

impl<Cap: flow_graph::FlowCapacity> FlowGraph<Cap> {
    /// Dinic 法により、頂点 `s` から頂点 `t` への最大流量を求める。
    ///
    /// 呼び出し後、内部の残余容量は最大流を実現した状態に更新されている。
    /// 各辺の流量は [`get_edge`](Self::get_edge) で、最小カットは
    /// [`min_cut`](Self::min_cut) でそれぞれ参照できる。
    ///
    /// # Args
    /// - `s`: 始点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `t`: 終点であり、`0..vertex_count()` の範囲でなければならず、`s` と
    ///   異なっていなければならない。
    ///
    /// # Returns
    /// `Cap`: `s` から `t` への最大流量
    ///
    /// # Panics
    /// - `s == t` の場合にパニックする。
    /// - `s`/`t` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(V^2 E)
    ///   - V は頂点数、E は辺数である。単位容量に近いグラフではより高速に
    ///     動作する (例えば二部マッチングに相当する構成では O(E sqrt(V)))。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::flow_graph::FlowGraph;
    ///
    /// // 0 -> 1 -> 3, 0 -> 2 -> 3 の2つの経路 (各辺容量2) を持つグラフ。
    /// let mut g = FlowGraph::<i64>::new(4);
    /// g.add_edge(0, 1, 2);
    /// g.add_edge(0, 2, 2);
    /// g.add_edge(1, 3, 2);
    /// g.add_edge(2, 3, 2);
    ///
    /// assert_eq!(4, g.max_flow(0, 3));
    /// ```
    pub fn max_flow(&mut self, s: usize, t: usize) -> Cap {
        debug_assert!(s != t, "s and t must be different vertices");

        let n = self.vertex_count();
        let mut flow = Cap::ZERO;

        loop {
            let level = self.bfs_levels(s);
            // 終点に到達できなくなった時点で、これ以上増加パスは存在しない。
            if level[t].is_none() {
                break;
            }

            // 「現在辺」ポインタ。同じフェーズ (レベルグラフ) の間、頂点ごとに
            // 使い回すことで、探索済みで行き止まりと分かった辺を再訪問しない
            // ようにする。
            let mut iter = vec![0_usize; n];
            loop {
                let pushed = self.dfs_augmenting_path(s, t, &level, &mut iter);
                if pushed == Cap::ZERO {
                    break;
                }
                flow = flow + pushed;
            }
        }

        flow
    }

    /// 最大流を求めた後の残余グラフにおいて、頂点 `s` から到達可能な頂点の
    /// 集合を返す。これは `s` を含む側の最小カットに一致する。
    ///
    /// # Args
    /// - `s`: [`max_flow`](Self::max_flow) を呼んだ際に指定した始点と同じ
    ///   頂点を渡す。
    ///
    /// # Returns
    /// `Vec<bool>`: `result[v]` が `true` であれば、頂点 `v` は最小カットの
    /// `s` 側に属する。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::flow_graph::FlowGraph;
    ///
    /// let mut g = FlowGraph::<i64>::new(3);
    /// g.add_edge(0, 1, 3);
    /// g.add_edge(1, 2, 1);
    ///
    /// assert_eq!(1, g.max_flow(0, 2));
    /// let cut = g.min_cut(0);
    /// // 0->1 はまだ残余容量があるため 1 も s 側に含まれるが、1->2 が
    /// // 飽和しているため 2 には到達できない。
    /// assert!(cut[0]);
    /// assert!(cut[1]);
    /// assert!(!cut[2]);
    /// ```
    pub fn min_cut(&self, s: usize) -> Vec<bool> {
        let n = self.vertex_count();
        let mut visited = vec![false; n];
        visited[s] = true;

        // s を起点に、残余容量が正の辺のみを辿って幅優先探索を行う。訪問できた
        // 頂点が、最小カットの s 側に属する頂点である。
        let mut queue = collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            for edge in &self.graph[u] {
                // 残余容量が正の辺だけを辿り、まだ訪問していない頂点だけを
                // 新たに訪問済みにする。
                if edge.cap > Cap::ZERO && !visited[edge.to] {
                    visited[edge.to] = true;
                    queue.push_back(edge.to);
                }
            }
        }

        visited
    }

    /// 始点 `s` からの BFS により、残余容量が正の辺のみを辿ってレベル
    /// (最短距離) を求める。
    ///
    /// # Args
    /// - `s`: 始点
    ///
    /// # Returns
    /// `Vec<Option<usize>>`: `result[v]` は `s` から `v` への残余グラフ上の
    /// 最短距離であり、到達できない場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    fn bfs_levels(&self, s: usize) -> Vec<Option<usize>> {
        let n = self.vertex_count();
        let mut level = vec![None; n];
        level[s] = Some(0);

        // キューから頂点を1つずつ取り出し、そこから残余容量が正の辺を1本
        // 辿った先の頂点へレベルを伝播させていく (幅優先探索)。
        let mut queue = collections::VecDeque::new();
        queue.push_back(s);
        while let Some(u) = queue.pop_front() {
            for edge in &self.graph[u] {
                // 残余容量が正の辺のみを辿り、まだレベルが決まっていない頂点
                // だけを、u のレベルより1つ大きいレベルとして確定させる。
                if edge.cap > Cap::ZERO && level[edge.to].is_none() {
                    level[edge.to] = Some(level[u].unwrap() + 1);
                    queue.push_back(edge.to);
                }
            }
        }

        level
    }

    /// レベルグラフ上で、`s` から `t` への増加パスを1本探し、その最大流量
    /// (ボトルネック) だけ流す。
    ///
    /// 明示的なスタックで `s` からの経路を管理する。ある頂点で使える辺が
    /// 尽きたら (行き止まりなら) その頂点をスタックから外し、1つ前の頂点の
    /// 「現在辺」ポインタを進める。これにより、同じフェーズ内では、一度
    /// 行き止まりと分かった辺を二度と辿らない。
    ///
    /// # Args
    /// - `s`: 始点
    /// - `t`: 終点
    /// - `level`: [`bfs_levels`](Self::bfs_levels) が返したレベル
    /// - `iter`: 頂点ごとの「現在辺」ポインタであり、同じフェーズの間、
    ///   呼び出し元で使い回す。
    ///
    /// # Returns
    /// `Cap`: 見つけた増加パスに流した量であり、増加パスが見つからなかった
    /// 場合は `Cap::ZERO` である。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(V + E) (`iter` を同一フェーズ内で使い回すため)
    fn dfs_augmenting_path(
        &mut self,
        s: usize,
        t: usize,
        level: &[Option<usize>],
        iter: &mut [usize],
    ) -> Cap {
        // s から t への経路を、頂点ではなく「その頂点から次へ進んだ際に
        // 使った辺のインデックス」の列として記録する。ボトルネックの計算と
        // 流量の反映の両方で、辺そのものへ直接アクセスする必要があるためである。
        let mut path = Vec::new();
        let mut u = s;

        while u != t {
            let mut advanced = false;
            while iter[u] < self.graph[u].len() {
                let edge = &self.graph[u][iter[u]];
                let can_advance =
                    edge.cap > Cap::ZERO && level[edge.to] == level[u].map(|lv| lv + 1);
                if can_advance {
                    path.push((u, iter[u]));
                    u = edge.to;
                    advanced = true;
                    break;
                }
                iter[u] += 1;
            }

            if !advanced {
                // u から先へは進めない (行き止まり) ため、経路を1つ戻す。
                // s 自体が行き止まりであれば、このフェーズに増加パスは
                // もう存在しない。
                let Some(&(parent, _)) = path.last() else {
                    return Cap::ZERO;
                };
                path.pop();
                // 戻った先の頂点から見て、u へ向かう辺はもう使えないと
                // 分かったため、次に試す辺へ進める。
                iter[parent] += 1;
                u = parent;
            }
        }

        // 経路上の各辺の残余容量のうち最小のものが、流せる量 (ボトルネック)
        // になる。
        let bottleneck = path
            .iter()
            .map(|&(v, idx)| self.graph[v][idx].cap)
            .min()
            .unwrap_or(Cap::MAX);

        for (v, idx) in path {
            let edge_to = self.graph[v][idx].to;
            let edge_rev = self.graph[v][idx].rev;
            self.graph[v][idx].cap = self.graph[v][idx].cap - bottleneck;
            self.graph[edge_to][edge_rev].cap = self.graph[edge_to][edge_rev].cap + bottleneck;
        }

        bottleneck
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // max_flow のテスト: 戻り値そのものと、呼び出し後に get_edge で観測できる
    // 各辺の流量を検証する。
    mod max_flow {
        use super::*;

        /// Scenario: 単純な直列経路では、最も細い辺の容量が最大流になる。
        /// - Given: `0 -> 1 -> 2` (容量 3, 1) の経路がある。
        /// - When: 0 から 2 への最大流を求める。
        /// - Then: 最大流は 1 (最も細い辺の容量) になる。
        #[test]
        fn bottlenecked_by_thinnest_edge_in_series() {
            // Given
            let mut sut = FlowGraph::<i64>::new(3);
            sut.add_edge(0, 1, 3);
            sut.add_edge(1, 2, 1);
            // When
            let result = sut.max_flow(0, 2);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 並列な複数経路の容量は加算される。
        /// - Given: `0 -> 1 -> 3` と `0 -> 2 -> 3` (各辺容量2) の、独立した
        ///   2経路がある。
        /// - When: 0 から 3 への最大流を求める。
        /// - Then: 最大流は 4 (2経路の容量の和) になる。
        #[test]
        fn sums_capacities_of_independent_parallel_paths() {
            // Given
            let mut sut = FlowGraph::<i64>::new(4);
            sut.add_edge(0, 1, 2);
            sut.add_edge(0, 2, 2);
            sut.add_edge(1, 3, 2);
            sut.add_edge(2, 3, 2);
            // When
            let result = sut.max_flow(0, 3);
            // Then
            assert_eq!(4, result);
        }

        /// Scenario: 終点に到達できないグラフでは、最大流は0になる (境界値)。
        /// - Given: 0 から 1 への辺を持たない、孤立した頂点を含むグラフが
        ///   ある。
        /// - When: 0 から 1 への最大流を求める。
        /// - Then: 最大流は0になる。
        #[test]
        fn returns_zero_when_sink_is_unreachable() {
            // Given
            let mut sut = FlowGraph::<i64>::new(2);
            // When
            let result = sut.max_flow(0, 1);
            // Then
            assert_eq!(0, result);
        }

        /// Scenario: 中継点の容量が両側の辺より小さい場合、それがボトル
        /// ネックになる (逆辺を使わないと最大流に到達できない典型例)。
        /// - Given: `0 -> 1` (容量3), `1 -> 2` (容量1), `1 -> 3` (容量3),
        ///   `2 -> 3` (容量3) を持つグラフがある。
        /// - When: 0 から 3 への最大流を求める。
        /// - Then: 最大流は 3 (`0->1` と `1->3` の容量) になる。
        #[test]
        fn handles_diamond_shaped_graph_correctly() {
            // Given
            let mut sut = FlowGraph::<i64>::new(4);
            sut.add_edge(0, 1, 3);
            sut.add_edge(1, 2, 1);
            sut.add_edge(1, 3, 3);
            sut.add_edge(2, 3, 3);
            // When
            let result = sut.max_flow(0, 3);
            // Then
            assert_eq!(3, result);
        }

        /// Scenario: 呼び出し後、各辺の流量を get_edge で参照できる。
        /// - Given: `0 -> 1 -> 2` (容量5, 3) の経路がある。
        /// - When: 0 から 2 への最大流を求める。
        /// - Then: `0->1` の流量は3 (ボトルネック分のみ使われる) になり、
        ///   `1->2` の流量は3 (容量いっぱい) になる。
        #[test]
        fn exposes_flow_per_edge_via_get_edge() {
            // Given
            let mut sut = FlowGraph::<i64>::new(3);
            let e01 = sut.add_edge(0, 1, 5);
            let e12 = sut.add_edge(1, 2, 3);
            // When
            sut.max_flow(0, 2);
            // Then
            assert_eq!((0, 1, 5, 3), sut.get_edge(e01));
            assert_eq!((1, 2, 3, 3), sut.get_edge(e12));
        }

        /// Scenario: 同じ2頂点間の複数辺 (多重辺) の容量も合算して流せる。
        /// - Given: `0 -> 1` 間に容量2の辺が2本あるグラフがある。
        /// - When: 0 から 1 への最大流を求める。
        /// - Then: 最大流は 4 (2本の辺の容量の和) になる。
        #[test]
        fn sums_capacities_of_multi_edges() {
            // Given
            let mut sut = FlowGraph::<i64>::new(2);
            sut.add_edge(0, 1, 2);
            sut.add_edge(0, 1, 2);
            // When
            let result = sut.max_flow(0, 1);
            // Then
            assert_eq!(4, result);
        }

        /// Scenario: s と t が同じ頂点であれば、パニックする (異常系)。
        /// - Given: 2頂点のグラフがある。
        /// - When: 同じ頂点を始点・終点として最大流を求めようとする。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "s and t must be different vertices")]
        fn panics_when_source_and_sink_are_the_same() {
            // Given
            let mut sut = FlowGraph::<i64>::new(2);
            // When, Then (panic)
            sut.max_flow(0, 0);
        }
    }

    // min_cut のテスト: 戻り値そのものを検証する。
    mod min_cut {
        use super::*;

        /// Scenario: 最大流を求めた後、飽和した辺の先には到達できない。
        /// - Given: `0 -> 1 -> 2` (容量3, 1) の経路がある。
        /// - When: 最大流を求めた後、0 を含む側の最小カットを求める。
        /// - Then: 0 と 1 はカットの s 側に含まれ、2 は含まれない
        ///   (`1->2` が飽和しているため)。
        #[test]
        fn excludes_vertices_beyond_saturated_edge() {
            // Given
            let mut sut = FlowGraph::<i64>::new(3);
            sut.add_edge(0, 1, 3);
            sut.add_edge(1, 2, 1);
            // When
            sut.max_flow(0, 2);
            let result = sut.min_cut(0);
            // Then
            assert!(result[0]);
            assert!(result[1]);
            assert!(!result[2]);
        }
    }
}

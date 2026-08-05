//! 辺の重みが 0 か 1 のみである場合に特化した、単一始点最短路を提供する
//! モジュールである。
//!
//! Dijkstra 法と同じ結果を `O(V + E)` で求められる。優先度キューの代わりに
//! deque を用い、重み 0 の辺は先頭に、重み 1 の辺は末尾に積むことで、常に
//! コストの小さい順に頂点を取り出せることを利用する。

use super::graph;
use std::collections;

/// 0-1 BFS の結果を保持する。
pub struct ZeroOneBfs {
    /// `dist[v]` は、最も近い始点から `v` までの最短コスト。到達できない場合は
    /// `None`。
    dist: Vec<Option<usize>>,
    /// `prev[v]` は、最短路上で `v` の直前に訪れた頂点。
    prev: Vec<Option<usize>>,
}

impl ZeroOneBfs {
    /// 最も近い始点から頂点 `v` までの最短コストを返す。
    ///
    /// # Args
    /// - `v`: コストを求める頂点
    ///
    /// # Returns
    /// `Option<usize>`: `v` までの最短コストであり、いずれの始点からも到達
    /// できない場合は `None` である。
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
    /// `Option<Vec<usize>>`: 始点から `v` に至る頂点列であり、`v` にいずれの
    /// 始点からも到達できない場合は `None` である。
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

impl<T> graph::Graph<T> {
    /// 辺の重みが 0 か 1 のみであるとみなして、単一始点最短路を求める。
    ///
    /// `is_one` が辺ごとに「重み 1 であるかどうか」を返す。`false` を返した
    /// 辺は重み 0 として扱う。
    ///
    /// # Args
    /// - `starts`: 探索の始点となる頂点の列
    /// - `is_one`: 辺のペイロードを受け取り、その辺の重みが 1 であれば
    ///   `true`、0 であれば `false` を返す関数
    ///
    /// # Returns
    /// `ZeroOneBfs`: 各頂点までの最短コストと、経路復元に必要な情報
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
    /// // 同じ向きへの移動は重み 0、方向転換は重み 1、という盤面探索を想定する。
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, false); // 直進
    /// g.add_edge(1, 2, true); // 方向転換
    /// g.add_edge(0, 2, true); // 方向転換 (直接)
    ///
    /// let result = g.zero_one_bfs(&[0], |&is_turn| is_turn);
    /// assert_eq!(Some(1), result.distance(2));
    /// ```
    pub fn zero_one_bfs<F>(&self, starts: &[usize], is_one: F) -> ZeroOneBfs
    where
        F: Fn(&T) -> bool,
    {
        // dist[v] は、v までの最短コストのうちこれまでに見つかった最小値である
        // (まだ見つかっていなければ None)。prev[v] は、その最短路上で v の直前に
        // 訪れた頂点であり、後で path_to から経路を復元するために使う。
        let mut dist = vec![None; self.vertex_count()];
        let mut prev = vec![None; self.vertex_count()];
        let mut deque = collections::VecDeque::new();

        // すべての始点をコスト0としてキューに積んでおく。
        for &s in starts {
            if dist[s].is_none() {
                dist[s] = Some(0);
                deque.push_back(s);
            }
        }

        // 重みが0か1しか無いため、deque の先頭から取り出す限り、取り出す頂点の
        // コストは常に非減少 (Dijkstra 法の優先度キューと同じ役割を、深さの浅い
        // 挿入だけで代用できる)。
        while let Some(u) = deque.pop_front() {
            let du = dist[u].unwrap();
            // 頂点 u から出る各辺について、その辺を使うことで隣接頂点 v までの
            // コストがこれまでより小さくできるなら更新する (「緩和」と呼ばれる
            // 操作)。
            for (v, payload) in self.edges(u) {
                let weight = if is_one(payload) { 1 } else { 0 };
                let nd = du + weight;
                let is_better = match dist[v] {
                    Some(cur) => nd < cur,
                    None => true,
                };
                if is_better {
                    dist[v] = Some(nd);
                    prev[v] = Some(u);
                    // 重み 0 の辺は先頭へ、重み 1 の辺は末尾へ積むことで、
                    // deque 全体が常にコストの昇順に並んだ状態を保つ。
                    if weight == 0 {
                        deque.push_front(v);
                    } else {
                        deque.push_back(v);
                    }
                }
            }
        }

        ZeroOneBfs { dist, prev }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // zero_one_bfs のテスト: 戻り値 (ZeroOneBfs) が保持する最短コスト・経路を検証する。
    mod zero_one_bfs {
        use super::*;

        /// Scenario: 重み1の辺を経由するより、重み0の辺だけで迂回する方が短い場合、
        /// 迂回路が採用される。
        /// - Given: `0->2` の直接辺 (重み1) と `0->1->2` の経路 (重み0+0=0) を持つ
        ///   グラフがある。
        /// - When: 頂点 0 を始点に 0-1 BFS を行う。
        /// - Then: 頂点 2 への最短コストは 0 になる。
        #[test]
        fn chooses_zero_weight_detour_over_direct_edge() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, false);
            sut.add_edge(1, 2, false);
            sut.add_edge(0, 2, true);
            // When
            let result = sut.zero_one_bfs(&[0], |&is_one| is_one);
            // Then
            assert_eq!(Some(0), result.distance(2));
        }

        /// Scenario: 到達できない頂点への最短コストは `None` になる。
        /// - Given: 孤立した頂点を含むグラフがある。
        /// - When: 孤立していない頂点を始点に 0-1 BFS を行う。
        /// - Then: 孤立した頂点への最短コストは `None` になる。
        #[test]
        fn returns_none_for_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<bool>::new(2);
            // When
            let result = sut.zero_one_bfs(&[0], |&is_one| is_one);
            // Then
            assert_eq!(None, result.distance(1));
        }

        /// Scenario: すべての辺が重み1の場合、通常の BFS と同じ辺数が距離になる。
        /// - Given: `0->1->2` の単純パスで、すべての辺の重みが1であるグラフがある。
        /// - When: 頂点 0 を始点に 0-1 BFS を行う。
        /// - Then: 頂点 2 への最短コストは 2 になる。
        #[test]
        fn matches_hop_count_when_all_weights_are_one() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, true);
            sut.add_edge(1, 2, true);
            // When
            let result = sut.zero_one_bfs(&[0], |&is_one| is_one);
            // Then
            assert_eq!(Some(2), result.distance(2));
        }

        /// Scenario: 最短経路が経路として復元できる。
        /// - Given: `0->2` の直接辺 (重み1) と `0->1->2` の経路 (重み0) を持つ
        ///   グラフがある。
        /// - When: 頂点 0 を始点に 0-1 BFS を行い、頂点 2 への経路を求める。
        /// - Then: `[0, 1, 2]` が返る (直接辺は使われない)。
        #[test]
        fn returns_shortest_path_not_direct_edge() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, false);
            sut.add_edge(1, 2, false);
            sut.add_edge(0, 2, true);
            // When
            let result = sut.zero_one_bfs(&[0], |&is_one| is_one);
            // Then
            assert_eq!(Some(vec![0, 1, 2]), result.path_to(2));
        }
    }
}

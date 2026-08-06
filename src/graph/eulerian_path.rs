//! オイラー路・オイラー閉路の構築を提供するモジュールである。
//!
//! Hierholzer のアルゴリズムを用いる。有向グラフを対象とし、すべての辺を
//! ちょうど1回ずつ通る経路 (オイラー路)、またはそれが始点に戻る閉路
//! (オイラー閉路) を構築する。
//!
//! 頂点ごとに「次に使う辺のインデックス」を管理しながら、行き止まりに
//! 至った頂点を経路の末尾に積んでいく (帰りがけ順に積む) ことで、
//! 各辺をちょうど1回だけ使った経路を線形時間で構築する。

use super::graph;

impl<T> graph::Graph<T> {
    /// オイラー路・オイラー閉路を1つ構築する。
    ///
    /// 存在するかどうかは、以下のいずれかを満たすかで判定する。
    /// - オイラー閉路: 辺を持つすべての頂点で、入次数と出次数が等しい。
    /// - オイラー路 (閉路でない): ちょうど1頂点で `出次数 = 入次数 + 1`
    ///   (始点)、ちょうど1頂点で `入次数 = 出次数 + 1` (終点) であり、
    ///   それ以外の辺を持つ頂点はすべて入次数と出次数が等しい。
    ///
    /// いずれの場合も、辺を持つすべての頂点が (辺をたどることで) 互いに
    /// 到達可能でなければならない。
    ///
    /// # Returns
    /// `Option<(Vec<usize>, Vec<&T>)>`: オイラー路・オイラー閉路が存在する
    /// 場合、通る順に並んだ頂点列と、その間で使われた辺のペイロードを通った順に
    /// 並べた列であり、頂点列の長さは辺のペイロード列より常に1大きい
    /// (`vertices[i]` から `payloads[i]` を通って `vertices[i+1]` へ至る)。
    /// 多重辺があってもどの辺を使ったかを区別できるよう、ペイロードには
    /// 元の入力での辺番号などを持たせておくとよい。存在しない場合は `None`
    /// であり、辺を1本も持たないグラフに対しては `Some((vec![], vec![]))` を
    /// 返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    ///   - V は頂点数、E は辺数である。
    /// - 空間計算量: O(V + E)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 -> 1 -> 2 -> 0 はオイラー閉路をなす。
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(1, 2, ());
    /// g.add_edge(2, 0, ());
    ///
    /// let (vertices, payloads) = g.eulerian_path().unwrap();
    /// assert_eq!(4, vertices.len());
    /// assert_eq!(3, payloads.len());
    /// assert_eq!(vertices[0], vertices[3]);
    /// ```
    pub fn eulerian_path(&self) -> Option<(Vec<usize>, Vec<&T>)> {
        let n = self.vertex_count();
        let mut out_degree = vec![0_usize; n];
        let mut in_degree = vec![0_usize; n];
        let mut edge_count = 0_usize;
        for u in 0..n {
            for (v, _) in self.edges(u) {
                out_degree[u] += 1;
                in_degree[v] += 1;
                edge_count += 1;
            }
        }

        if edge_count == 0 {
            return Some((Vec::new(), Vec::new()));
        }

        // 始点・終点の候補を、次数の過不足から決める。
        let mut start = None;
        let mut extra_out = 0_usize;
        let mut extra_in = 0_usize;
        for v in 0..n {
            let diff = out_degree[v] as isize - in_degree[v] as isize;
            match diff {
                0 => {}
                1 => {
                    extra_out += 1;
                    start = Some(v);
                }
                -1 => {
                    extra_in += 1;
                }
                _ => return None,
            }
        }
        // 閉路の場合、始点は辺を持つ任意の頂点でよい。
        let start = match (extra_out, extra_in) {
            (0, 0) => (0..n).find(|&v| out_degree[v] > 0)?,
            (1, 1) => start?,
            _ => return None,
        };

        // 各頂点で「次にたどる辺」の位置を管理しながら、行き止まりに至った
        // 頂点を帰りがけ順に積む (Hierholzer のアルゴリズム)。頂点ごとに
        // イテレータを保持して `next()` で進めることで、`nth()` の再走査に
        // よる O(次数^2) の無駄を避け、全体で O(V + E) に抑える。
        // `arrival_edge` は `stack` と対にして、その頂点へ至った辺の
        // ペイロードを保持する (始点にはペイロードが無いため `None`)。
        let mut iters = (0..n).map(|u| self.edges(u)).collect::<Vec<_>>();
        let mut circuit_vertices = Vec::with_capacity(edge_count + 1);
        let mut circuit_payloads = Vec::with_capacity(edge_count);
        let mut stack = vec![start];
        let mut arrival_edge: Vec<Option<&T>> = vec![None];
        while let Some(&u) = stack.last() {
            if let Some((v, payload)) = iters[u].next() {
                stack.push(v);
                arrival_edge.push(Some(payload));
            } else {
                circuit_vertices.push(u);
                if let Some(payload) = arrival_edge.pop().unwrap() {
                    circuit_payloads.push(payload);
                }
                stack.pop();
            }
        }
        circuit_vertices.reverse();
        circuit_payloads.reverse();

        // 使われた辺の総数が全体の辺数と一致しなければ、辺を持つ頂点全体が
        // 連結ではなく、そもそもオイラー路が存在しない。
        if circuit_vertices.len() != edge_count + 1 {
            return None;
        }

        Some((circuit_vertices, circuit_payloads))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // eulerian_path のテスト: 戻り値そのものを検証する。
    mod eulerian_path {
        use super::*;

        /// Scenario: すべての頂点で入次数と出次数が等しい場合、始点に戻る
        /// オイラー閉路が得られる。
        /// - Given: `0->1->2->0` の閉路がある。
        /// - When: オイラー路を求める。
        /// - Then: 3本の辺すべてを1回ずつ通り、始点に戻る経路が得られる。
        #[test]
        fn returns_circuit_when_in_degree_equals_out_degree_everywhere() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 0, ());
            // When
            let (vertices, payloads) = sut.eulerian_path().unwrap();
            // Then
            assert_eq!(4, vertices.len());
            assert_eq!(3, payloads.len());
            assert_eq!(vertices[0], vertices[3]);
        }

        /// Scenario: ちょうど1頂点で出次数が入次数より1多く、別の1頂点で
        /// 入次数が出次数より1多い場合、前者を始点・後者を終点とする経路が
        /// 得られる。
        /// - Given: `0->1->2` の単純パスがある。
        /// - When: オイラー路を求める。
        /// - Then: `[0, 1, 2]` が得られる。
        #[test]
        fn returns_path_from_excess_out_degree_to_excess_in_degree_vertex() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            // When
            let (vertices, payloads) = sut.eulerian_path().unwrap();
            // Then
            assert_eq!(vec![0, 1, 2], vertices);
            assert_eq!(2, payloads.len());
        }

        /// Scenario: 辺を持たないグラフでは、空の経路が返る (境界値)。
        /// - Given: 3頂点、辺を持たないグラフがある。
        /// - When: オイラー路を求める。
        /// - Then: 空の経路が返る。
        #[test]
        fn returns_empty_path_for_graph_without_edges() {
            // Given
            let sut = graph::Graph::<()>::new(3);
            // When
            let result = sut.eulerian_path();
            // Then
            assert_eq!(Some((Vec::new(), Vec::new())), result);
        }

        /// Scenario: 次数の過不足を持つ頂点が2つを超える場合、オイラー路は
        /// 存在しない。
        /// - Given: 頂点 0 の出次数が入次数より2多いグラフがある。
        /// - When: オイラー路を求める。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_more_than_two_vertices_have_unbalanced_degree() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(0, 2, ());
            // When
            let result = sut.eulerian_path();
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 次数条件を満たしていても、辺を持つ頂点全体が連結でなければ
        /// オイラー路は存在しない。
        /// - Given: `0->1->0` の閉路と、それとは独立した `2->3->2` の閉路を
        ///   持つグラフがある。
        /// - When: オイラー路を求める。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_edges_are_disconnected() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 0, ());
            sut.add_edge(2, 3, ());
            sut.add_edge(3, 2, ());
            // When
            let result = sut.eulerian_path();
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 多重辺があっても、すべての辺をちょうど1回ずつ通る経路が
        /// 得られる。
        /// - Given: `0->1` の辺が2本あるグラフがある。
        /// - When: オイラー路を求める。
        /// - Then: 2本の辺をちょうど1回ずつ通る経路が得られる。
        #[test]
        fn handles_multi_edges() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 0, ());
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 0, ());
            // When
            let (vertices, payloads) = sut.eulerian_path().unwrap();
            // Then
            assert_eq!(5, vertices.len());
            assert_eq!(4, payloads.len());
        }

        /// Scenario: 返された辺のペイロード列は、実際に通った辺と対応しており、
        /// 元の入力での辺番号として使うことで多重辺を区別できる。
        /// - Given: 辺のペイロードとして元の入力での辺番号を持たせたグラフが
        ///   ある。
        /// - When: オイラー路を求める。
        /// - Then: 頂点列の各遷移が、返されたペイロード (辺番号) の辺と
        ///   実際に一致する。
        #[test]
        fn returns_payloads_matching_traversed_edges() {
            // Given
            let mut sut = graph::Graph::new(2);
            let mut edge_endpoints = Vec::new();
            for &(u, v) in &[(0, 1), (1, 0), (0, 1), (1, 0)] {
                sut.add_edge(u, v, edge_endpoints.len() as u32);
                edge_endpoints.push((u, v));
            }
            // When
            let (vertices, payloads) = sut.eulerian_path().unwrap();
            // Then
            for (i, &edge_id) in payloads.iter().enumerate() {
                assert_eq!(
                    edge_endpoints[*edge_id as usize],
                    (vertices[i], vertices[i + 1])
                );
            }
        }
    }
}

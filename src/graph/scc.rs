//! 強連結成分分解 (SCC) を提供するモジュールである。
//!
//! Kosaraju のアルゴリズムを用いる。まず元のグラフを深さ優先探索し、帰りがけ順
//! (postorder) を求める。次に、辺をすべて逆向きにした転置グラフに対して、
//! postorder の逆順 (完了時刻が遅い頂点から) に深さ優先探索を行う。このとき
//! 得られる各探索木が、1つの強連結成分をなす。

use super::graph;

/// SCC 分解の結果を保持する。
pub struct Scc {
    /// `component_id[v]` は、頂点 `v` が属する強連結成分の番号である。
    ///
    /// 番号は縮約グラフ (各強連結成分を1頂点に潰した DAG) のトポロジカル順序に
    /// なっている。すなわち、元のグラフに `u -> v` という辺があり、`u` と `v` が
    /// 異なる強連結成分に属するならば、`component_id(u) < component_id(v)` が
    /// 成り立つ。
    component_id: Vec<usize>,
    /// 強連結成分の個数。
    num_components: usize,
}

impl Scc {
    /// 強連結成分の個数を返す。
    ///
    /// # Returns
    /// `usize`: 強連結成分の個数。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn component_count(&self) -> usize {
        self.num_components
    }

    /// 頂点 `v` が属する強連結成分の番号を返す。
    ///
    /// # Args
    /// - `v`: 頂点番号。
    ///
    /// # Returns
    /// `usize`: `v` が属する強連結成分の番号。番号は縮約グラフのトポロジカル
    /// 順序になっている ([`Scc`] の説明を参照)。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn component_id(&self, v: usize) -> usize {
        self.component_id[v]
    }

    /// 頂点 `u` と `v` が同じ強連結成分に属するかどうかを返す。
    ///
    /// # Args
    /// - `u`: 判定したい頂点。
    /// - `v`: 判定したい頂点。
    ///
    /// # Returns
    /// `bool`: 同じ強連結成分に属する場合は `true`。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn same_component(&self, u: usize, v: usize) -> bool {
        self.component_id[u] == self.component_id[v]
    }

    /// 強連結成分ごとに頂点をまとめた一覧を返す。
    ///
    /// # Returns
    /// `Vec<Vec<usize>>`: `groups()[i]` は、番号 `i` の強連結成分に属する頂点の
    /// 一覧である。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    pub fn groups(&self) -> Vec<Vec<usize>> {
        let mut groups = vec![Vec::new(); self.num_components];
        for (v, &id) in self.component_id.iter().enumerate() {
            groups[id].push(v);
        }
        groups
    }
}

/// スタック上で処理待ちの頂点を表す。`Enter` は初訪問時、`Leave` はその頂点の
/// 子をすべて訪れ終えた後の処理を表す。
enum Frame {
    Enter(usize),
    Leave(usize),
}

impl<T> graph::Graph<T> {
    /// 強連結成分分解 (SCC) を行う。
    ///
    /// # Returns
    /// `Scc`: 各頂点が属する強連結成分の番号。
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
    /// // 0 <-> 1 <-> 2 は1つの強連結成分をなし、3 はそこから到達できるが
    /// // 戻れない別の成分をなす。
    /// let mut g = Graph::new(4);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(1, 2, ());
    /// g.add_edge(2, 0, ());
    /// g.add_edge(2, 3, ());
    ///
    /// let scc = g.scc();
    /// assert!(scc.same_component(0, 2));
    /// assert!(!scc.same_component(0, 3));
    /// assert_eq!(2, scc.component_count());
    /// ```
    pub fn scc(&self) -> Scc {
        let n = self.vertex_count();

        // 元のグラフを深さ優先探索し、帰りがけ順を求める。
        let mut visited = vec![false; n];
        let mut postorder = Vec::with_capacity(n);
        for start in 0..n {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            let mut stack = vec![Frame::Enter(start)];
            while let Some(frame) = stack.pop() {
                match frame {
                    Frame::Enter(u) => {
                        stack.push(Frame::Leave(u));
                        for (v, _) in self.edges(u) {
                            if !visited[v] {
                                visited[v] = true;
                                stack.push(Frame::Enter(v));
                            }
                        }
                    }
                    Frame::Leave(u) => postorder.push(u),
                }
            }
        }

        // 辺をすべて逆向きにした転置グラフを構築する。
        let mut transposed = graph::Graph::new(n);
        for u in 0..n {
            for (v, _) in self.edges(u) {
                transposed.add_edge(v, u, ());
            }
        }

        // 完了時刻が遅い頂点から順に、転置グラフを深さ優先探索する。
        // 1回の探索で訪れる頂点がちょうど1つの強連結成分をなす。
        const UNASSIGNED: usize = usize::MAX;
        let mut component_id = vec![UNASSIGNED; n];
        let mut num_components = 0;
        for &start in postorder.iter().rev() {
            if component_id[start] != UNASSIGNED {
                continue;
            }
            component_id[start] = num_components;
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                for (v, _) in transposed.edges(u) {
                    if component_id[v] == UNASSIGNED {
                        component_id[v] = num_components;
                        stack.push(v);
                    }
                }
            }
            num_components += 1;
        }

        Scc {
            component_id,
            num_components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // scc のテスト: 戻り値 (Scc) が保持する成分情報を検証する。
    mod scc {
        use super::*;

        /// Scenario: 相互に到達可能な頂点は、同じ強連結成分に属する。
        /// - Given: `0->1->2->0` の閉路をなすグラフがある。
        /// - When: SCC 分解を行う。
        /// - Then: 3頂点すべてが同じ成分に属し、成分数は1になる。
        #[test]
        fn groups_mutually_reachable_vertices_into_same_component() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 0, ());
            // When
            let result = sut.scc();
            // Then
            assert!(result.same_component(0, 1));
            assert!(result.same_component(1, 2));
            assert_eq!(1, result.component_count());
        }

        /// Scenario: 一方向にしか到達できない頂点は、別の強連結成分に属する。
        /// - Given: 閉路 `0->1->2->0` と、そこから片方向にのみ到達できる頂点 3
        ///   (`2->3`) を持つグラフがある。
        /// - When: SCC 分解を行う。
        /// - Then: 頂点 0 と頂点 3 は異なる成分に属し、成分数は2になる。
        #[test]
        fn separates_vertices_reachable_only_one_way() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 0, ());
            sut.add_edge(2, 3, ());
            // When
            let result = sut.scc();
            // Then
            assert!(!result.same_component(0, 3));
            assert_eq!(2, result.component_count());
        }

        /// Scenario: 辺を持たないグラフでは、各頂点が独立した成分をなす (境界値)。
        /// - Given: 3頂点、辺を持たないグラフがある。
        /// - When: SCC 分解を行う。
        /// - Then: 成分数は3になり、どの2頂点も異なる成分に属する。
        #[test]
        fn treats_each_vertex_as_own_component_without_edges() {
            // Given
            let sut = graph::Graph::<()>::new(3);
            // When
            let result = sut.scc();
            // Then
            assert_eq!(3, result.component_count());
            assert!(!result.same_component(0, 1));
        }

        /// Scenario: 縮約グラフの辺の向きに沿って、成分番号は昇順になる。
        /// - Given: 成分 `{0,1}` から成分 `{2,3}` へのみ到達可能なグラフがある。
        /// - When: SCC 分解を行う。
        /// - Then: 成分 `{0,1}` の番号は、成分 `{2,3}` の番号より小さい。
        #[test]
        fn assigns_increasing_ids_along_condensation_edges() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 0, ());
            sut.add_edge(2, 3, ());
            sut.add_edge(3, 2, ());
            sut.add_edge(1, 2, ());
            // When
            let result = sut.scc();
            // Then
            assert!(result.component_id(0) < result.component_id(2));
        }

        /// Scenario: 強連結成分ごとに頂点をまとめた一覧が得られる。
        /// - Given: `0->1->2->0` の閉路と、そこから到達できる孤立した頂点 3
        ///   (`2->3`) を持つグラフがある。
        /// - When: SCC 分解を行い、成分ごとの頂点一覧を求める。
        /// - Then: 一方の成分に `{0,1,2}` が、もう一方に `{3}` がまとまる。
        #[test]
        fn groups_vertices_by_component() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 0, ());
            sut.add_edge(2, 3, ());
            // When
            let scc = sut.scc();
            let mut result = scc.groups();
            for group in result.iter_mut() {
                group.sort_unstable();
            }
            // Then
            assert_eq!(2, result.len());
            assert!(result.contains(&vec![0, 1, 2]));
            assert!(result.contains(&vec![3]));
        }
    }
}

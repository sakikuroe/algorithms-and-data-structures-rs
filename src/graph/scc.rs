//! 強連結成分分解 (SCC) を提供するモジュールである。
//!
//! Tarjan のアルゴリズムを用いる。深さ優先探索を1回行うだけで、転置グラフを
//! 構築する必要がない。各頂点について、探索順序 (`ord`) と、その頂点の部分木
//! (逆辺を含む) から到達できる最も浅い探索順序 (`low`) を求める。`low[v]` が
//! `ord[v]` に一致した時点で、v を根とする強連結成分が確定する。

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
    /// `usize`: 強連結成分の個数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn component_count(&self) -> usize {
        self.num_components
    }

    /// 頂点 `v` が属する強連結成分の番号を返す。
    ///
    /// # Args
    /// - `v`: 頂点番号
    ///
    /// # Returns
    /// `usize`: `v` が属する強連結成分の番号であり、番号は縮約グラフの
    /// トポロジカル順序になっている ([`Scc`] の説明を参照)。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn component_id(&self, v: usize) -> usize {
        self.component_id[v]
    }

    /// 頂点 `u` と `v` が同じ強連結成分に属するかどうかを返す。
    ///
    /// # Args
    /// - `u`: 判定したい頂点
    /// - `v`: 判定したい頂点
    ///
    /// # Returns
    /// `bool`: 同じ強連結成分に属する場合は `true`
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

impl<T> graph::Graph<T> {
    /// 強連結成分分解 (SCC) を行う。
    ///
    /// # Returns
    /// `Scc`: 各頂点が属する強連結成分の番号
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

        const UNVISITED: usize = usize::MAX;
        // ord[v]: 深さ優先探索での v の発見順序。未訪問なら UNVISITED。
        let mut ord = vec![UNVISITED; n];
        // low[v]: v の部分木 (後退辺を含む) から到達できる、最も浅い発見順序。
        let mut low = vec![0_usize; n];
        // 現在の探索経路上にあり、まだどの強連結成分にも確定していない頂点を、
        // 発見順に保持するスタック。
        let mut on_stack = vec![false; n];
        let mut on_stack_list = Vec::with_capacity(n);

        const UNASSIGNED: usize = usize::MAX;
        let mut component_id = vec![UNASSIGNED; n];
        let mut num_components = 0;
        let mut now_ord = 0;

        // 「現在辺」ポインタ。探索経路上の各頂点について、次に調べる隣接辺の
        // 添字を持つ。
        let mut next_edge = vec![0_usize; n];
        let mut dfs_stack = Vec::new();

        for start in 0..n {
            if ord[start] != UNVISITED {
                continue;
            }

            dfs_stack.push(start);
            ord[start] = now_ord;
            low[start] = now_ord;
            now_ord += 1;
            on_stack_list.push(start);
            on_stack[start] = true;

            while let Some(&v) = dfs_stack.last() {
                if next_edge[v] < self.edges[v].len() {
                    let to = self.edges[v][next_edge[v]].0 as usize;
                    next_edge[v] += 1;

                    if ord[to] == UNVISITED {
                        // 未訪問の頂点へは、そのまま探索を進める。
                        dfs_stack.push(to);
                        ord[to] = now_ord;
                        low[to] = now_ord;
                        now_ord += 1;
                        on_stack_list.push(to);
                        on_stack[to] = true;
                    } else if on_stack[to] {
                        // 現在の探索経路上にある頂点への辺 (後退辺、または
                        // 横断辺) であれば、low を更新する。
                        low[v] = low[v].min(ord[to]);
                    }
                } else {
                    // v から出る辺をすべて調べ終えた。1つ前 (親) へ low を
                    // 伝播したうえで、v を経路スタックから外す。
                    dfs_stack.pop();
                    if let Some(&parent) = dfs_stack.last() {
                        low[parent] = low[parent].min(low[v]);
                    }

                    if low[v] == ord[v] {
                        // v が強連結成分の根である。保持スタックを v まで
                        // 遡り、その区間をまとめて1つの成分として確定する。
                        loop {
                            let u = on_stack_list.pop().unwrap();
                            on_stack[u] = false;
                            component_id[u] = num_components;
                            if u == v {
                                break;
                            }
                        }
                        num_components += 1;
                    }
                }
            }
        }

        // 探索の完了が早い成分ほど小さい番号が割り振られている
        // (縮約グラフのトポロジカル順序とは逆順) ため、昇順がトポロジカル
        // 順序になるよう番号を反転する。
        for id in &mut component_id {
            *id = num_components - 1 - *id;
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

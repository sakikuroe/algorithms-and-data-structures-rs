//! Kruskal 法による最小全域木 (森) の構築を提供するモジュールである。
//!
//! グラフに保持された辺は、向きを無視した無向辺として扱う。すなわち、
//! [`Graph::add_edge`](super::graph::Graph::add_edge) で一方向のみ追加した辺、
//! [`Graph::add_undirected_edge`](super::graph::Graph::add_undirected_edge) で
//! 双方向に追加した辺のいずれも候補として扱われる (後者は同一の辺が重複して
//! 候補に現れるが、Union-Find が片方を自動的に棄却するため結果に影響しない)。
//! グラフが連結でない場合は、連結成分ごとの最小全域木からなる「最小全域森」を
//! 返す。

use super::super::ds::union_find;
use super::graph;

/// 最小全域木 (森) の結果を保持する。
pub struct MinimumSpanningForest<'a, T> {
    /// 採用された辺の一覧。`(始点, 終点, ペイロードへの参照)` の組で、
    /// Kruskal 法が採用した順 (重みの昇順) に並ぶ。
    edges: Vec<(usize, usize, &'a T)>,
    /// 連結成分の個数。1 であれば単一の全域木、2 以上であれば森である。
    component_count: usize,
}

impl<'a, T> MinimumSpanningForest<'a, T> {
    /// 採用された辺を、Kruskal 法が採用した順 (重みの昇順) に列挙する。
    ///
    /// # Returns
    /// `impl Iterator<Item = (usize, usize, &T)>`: 採用された各辺の
    /// `(始点, 終点, ペイロード)` であり、合計重みが必要な場合は、この列を辿って
    /// 呼び出し側で加算すればよい。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (返すイテレータの消費全体では、採用された辺数に
    ///   比例する O(V))
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize, &'a T)> + '_ {
        self.edges.iter().copied()
    }

    /// 連結成分の個数を返す。
    ///
    /// # Returns
    /// `usize`: 連結成分の個数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn component_count(&self) -> usize {
        self.component_count
    }

    /// グラフ全体が単一の木として連結であるかどうかを返す。
    ///
    /// # Returns
    /// `bool`: 連結成分がちょうど1つであれば `true`
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn is_spanning_tree(&self) -> bool {
        self.component_count == 1
    }
}

impl<T: Copy + Ord> graph::Graph<T> {
    /// 辺のペイロード自体を重みとして扱い、Kruskal 法で最小全域木 (森) を求める。
    ///
    /// 重みの型 `T` を自分自身から取り出すだけでよい場合の簡易版であり、
    /// 重み以外の情報 (元の入力での辺番号など) も併せ持つペイロードから重みを
    /// 取り出したい場合は
    /// [`minimum_spanning_forest_by`](Self::minimum_spanning_forest_by) を使う。
    ///
    /// # Returns
    /// `MinimumSpanningForest<T>`: 採用された辺と連結成分数
    ///
    /// # Complexity
    /// - 時間計算量: O(E log E)
    ///   - E は辺数である。ソートが支配的であり、Union-Find 部分は
    ///     ならして O(E α(V)) で済む。
    /// - 空間計算量: O(V + E)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 三角形 0-1(1), 1-2(2), 0-2(3)。最小全域木は 0-1 と 1-2 を採用する。
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, 1_u32);
    /// g.add_undirected_edge(1, 2, 2_u32);
    /// g.add_undirected_edge(0, 2, 3_u32);
    ///
    /// let mst = g.minimum_spanning_forest();
    /// assert!(mst.is_spanning_tree());
    ///
    /// let total_weight = mst.edges().map(|(_, _, &w)| w).sum::<u32>();
    /// assert_eq!(3, total_weight);
    /// ```
    pub fn minimum_spanning_forest(&self) -> MinimumSpanningForest<'_, T> {
        self.minimum_spanning_forest_by(|&w| w)
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、Kruskal 法で
    /// 最小全域木 (森) を求める。
    ///
    /// ペイロード `T` が重み以外の情報 (元の入力での辺番号など) も併せ持つ場合に
    /// 使う。ペイロード自体が重みである場合は
    /// [`minimum_spanning_forest`](Self::minimum_spanning_forest) の方が簡潔に
    /// 書ける。
    ///
    /// # Args
    /// - `weight_of`: 辺のペイロードから、比較可能な重みを取り出す関数
    ///
    /// # Returns
    /// `MinimumSpanningForest<T>`: 採用された辺と連結成分数であり、返される辺の
    /// ペイロードは射影前の `T` そのものであり、元の辺番号などの情報を
    /// そのまま辿れる。
    ///
    /// # Complexity
    /// - 時間計算量: O(E log E)
    ///   - E は辺数である。
    /// - 空間計算量: O(V + E)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // ペイロードが (重み, 元の辺番号) の組であるケース。
    /// let mut g = Graph::new(2);
    /// g.add_undirected_edge(0, 1, (7_u32, 3_usize));
    ///
    /// let mst = g.minimum_spanning_forest_by(|&(w, _id)| w);
    /// let (_, _, &(_, id)) = mst.edges().next().unwrap();
    /// assert_eq!(3, id);
    /// ```
    pub fn minimum_spanning_forest_by<W, F>(&self, weight_of: F) -> MinimumSpanningForest<'_, T>
    where
        W: Ord,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();

        // グラフ中のすべての辺を、採用するかどうかまだ決まっていない候補として
        // 1つのリストに集める。add_edge・add_undirected_edge のどちらで登録
        // された辺も、ここでは区別せずすべて候補として扱う。
        let mut candidates = (0..n)
            .flat_map(|u| self.edges(u).map(move |(v, payload)| (u, v, payload)))
            .collect::<Vec<(usize, usize, &T)>>();
        // 集めた候補を、重みが軽い順に並べ替える。Kruskal 法は、この順に
        // 「軽い辺から貪欲に採用するかどうかを判定していく」ことで最小全域木を
        // 構成する。
        candidates.sort_by_key(|&(_, _, payload)| weight_of(payload));

        // Union-Find を使い、これまでに採用した辺だけで見たときの連結成分を
        // 管理する。同じ連結成分に属する頂点同士は、辺を通じてすでに (直接、
        // または他の頂点を経由して間接的に) つながっている状態を表す。
        let mut uf = union_find::UnionFind::new(n);
        let mut edges = Vec::new();
        // 軽い辺から順に見ていき、その両端点がまだ同じ連結成分でなければ採用する。
        for (u, v, payload) in candidates {
            if !uf.is_same(u, v) {
                // 両端点が別々の連結成分に属している場合、この辺を採用しても
                // 閉路はできず、木のまま2つの成分を1つにまとめられる。
                // そのため、この辺を採用し、両端点の連結成分を1つに統合する。
                uf.union(u, v);
                edges.push((u, v, payload));
            }
            // 両端点がすでに同じ連結成分であれば、採用すると閉路ができてしまう
            // ため、この辺は採用せず読み飛ばす。自己ループ (u == v) も、
            // 両端点が常に同じ連結成分であるため、ここで自然に棄却される。
        }

        // 最終的な連結成分の個数は、Union-Find 上で根になっている頂点の個数に
        // 等しい。これが1であれば全域木、2以上であれば連結成分ごとの木からなる
        // 森である。
        let component_count = (0..n).filter(|&v| uf.is_root(v)).count();

        MinimumSpanningForest {
            edges,
            component_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // minimum_spanning_forest のテスト: 戻り値 (MinimumSpanningForest) が保持
    // する採用辺・連結成分数を検証する。
    mod minimum_spanning_forest {
        use super::*;

        /// Scenario: 三角形では、最も重い辺を除いた2辺が採用される。
        /// - Given: `0-1`(1), `1-2`(2), `0-2`(3) の三角形がある。
        /// - When: 最小全域木を求める。
        /// - Then: 単一の全域木になり、合計重みは 3 (= 1 + 2) になる。
        #[test]
        fn excludes_heaviest_edge_in_triangle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, 1_u32);
            sut.add_undirected_edge(1, 2, 2_u32);
            sut.add_undirected_edge(0, 2, 3_u32);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert!(result.is_spanning_tree());
            assert_eq!(3, result.edges().map(|(_, _, &w)| w).sum::<u32>());
        }

        /// Scenario: 連結でないグラフでは、連結成分ごとの木からなる森になる。
        /// - Given: `{0,1}` と `{2,3}` の、互いに独立した2つの連結成分がある。
        /// - When: 最小全域木を求める。
        /// - Then: 連結成分数は2になり、単一の全域木ではなく、採用された辺は
        ///   2本 (各成分の辺) になる。
        #[test]
        fn forms_forest_for_disconnected_components() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, 1_u32);
            sut.add_undirected_edge(2, 3, 1_u32);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert_eq!(2, result.component_count());
            assert!(!result.is_spanning_tree());
            assert_eq!(2, result.edges().count());
        }

        /// Scenario: 同じ2頂点間に複数の辺 (多重辺) があると、最も軽い辺が
        /// 採用される。
        /// - Given: `0-1` 間に重み 5 と 1 の2本の辺がある。
        /// - When: 最小全域木を求める。
        /// - Then: 合計重みは軽い方の 1 になる。
        #[test]
        fn chooses_lighter_edge_among_parallel_edges() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_undirected_edge(0, 1, 5_u32);
            sut.add_undirected_edge(0, 1, 1_u32);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert_eq!(1, result.edges().count());
            assert_eq!(1, result.edges().map(|(_, _, &w)| w).sum::<u32>());
        }

        /// Scenario: 自己ループは、連結性に寄与しないため採用されない (境界値)。
        /// - Given: 頂点 0 に自己ループがあるグラフがある。
        /// - When: 最小全域木を求める。
        /// - Then: 採用された辺は0本のままである。
        #[test]
        fn excludes_self_loop() {
            // Given
            let mut sut = graph::Graph::new(1);
            sut.add_edge(0, 0, 1_u32);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert_eq!(0, result.edges().count());
            assert!(result.is_spanning_tree());
        }

        /// Scenario: 頂点を持たないグラフでは、連結成分数は0になる (境界値)。
        /// - Given: 頂点数 0 のグラフがある。
        /// - When: 最小全域木を求める。
        /// - Then: 連結成分数は0であり、採用された辺もない。
        #[test]
        fn returns_zero_components_for_graph_without_vertices() {
            // Given
            let sut = graph::Graph::<u32>::new(0);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert_eq!(0, result.component_count());
            assert_eq!(0, result.edges().count());
        }

        /// Scenario: 一方向のみ辺を追加した場合でも、無向辺として扱われる。
        /// - Given: `add_edge` で一方向 (`0 -> 1`) のみ辺を追加したグラフがある。
        /// - When: 最小全域木を求める。
        /// - Then: 単一の全域木として扱われる。
        #[test]
        fn treats_one_directional_edge_as_undirected() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_edge(0, 1, 1_u32);
            // When
            let result = sut.minimum_spanning_forest();
            // Then
            assert!(result.is_spanning_tree());
            assert_eq!(1, result.edges().count());
        }
    }

    // minimum_spanning_forest_by のテスト: 射影した重みで採用される辺が
    // 決まることを検証する。
    mod minimum_spanning_forest_by {
        use super::*;

        /// Scenario: ペイロードから射影した重みを使って辺の採用を決められる。
        /// - Given: ペイロードが `(重み, 元の辺番号)` の組であるグラフがある。
        /// - When: 重みだけを取り出す関数を渡して最小全域木を求める。
        /// - Then: 採用された辺のペイロードから、元の辺番号を復元できる。
        #[test]
        fn preserves_original_payload_for_selected_edges() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_undirected_edge(0, 1, (7_u32, 3_usize));
            // When
            let result = sut.minimum_spanning_forest_by(|&(w, _id)| w);
            // Then
            let (_, _, &(w, id)) = result.edges().next().unwrap();
            assert_eq!(7, w);
            assert_eq!(3, id);
        }
    }
}

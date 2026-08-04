//! ハッシュ可能な任意の頂点ラベルから `Graph<T>` を構築するビルダーを提供する
//! モジュールである。
//!
//! パズルの盤面、文字列、タプルなど、頂点が最初から整数で採番できない状態空間を
//! 探索する際に、手動の座標圧縮を書かずに済むようにする。

use super::graph;
use std::{collections, hash};

/// ハッシュ可能な任意の頂点ラベル `V` に頂点番号を自動採番しながら `Graph<T>` を
/// 構築するビルダーである。
pub struct GraphBuilder<V, T> {
    /// ラベルから、それに採番した頂点番号への対応。
    ids: collections::HashMap<V, usize>,
    /// 頂点番号 `i` に対応する元のラベル。
    labels: Vec<V>,
    /// 追加された辺の `(始点, 終点, ペイロード)` の列。
    edges: Vec<(usize, usize, T)>,
}

impl<V, T> GraphBuilder<V, T>
where
    V: Clone + Eq + hash::Hash,
{
    /// 頂点・辺を持たない空のビルダーを生成する。
    ///
    /// # Returns
    /// `Self`: 空のビルダー。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::builder::GraphBuilder;
    ///
    /// let builder = GraphBuilder::<&str, ()>::new();
    /// let (g, labels) = builder.build();
    /// assert_eq!(0, g.vertex_count());
    /// assert!(labels.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        GraphBuilder {
            ids: collections::HashMap::new(),
            labels: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// ラベル `label` に対応する頂点番号を返す。未登録のラベルであれば、
    /// 新たに頂点番号を採番して登録する。
    ///
    /// # Args
    /// - `label`: 頂点番号を求めたいラベル。
    ///
    /// # Returns
    /// `usize`: `label` に対応する頂点番号。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    fn id_of(&mut self, label: &V) -> usize {
        if let Some(&id) = self.ids.get(label) {
            id
        } else {
            let id = self.labels.len();
            self.labels.push(label.clone());
            self.ids.insert(label.clone(), id);
            id
        }
    }

    /// 有向辺 `u -> v` を、ペイロード `payload` とともに追加する。`u`/`v` が
    /// 未登録のラベルであれば、新たに頂点番号を採番する。
    ///
    /// # Args
    /// - `u`: 辺の始点のラベル。
    /// - `v`: 辺の終点のラベル。
    /// - `payload`: 辺に付与する任意の値。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::builder::GraphBuilder;
    ///
    /// let mut builder = GraphBuilder::new();
    /// builder.add_edge("a", "b", ());
    /// let (g, labels) = builder.build();
    /// assert_eq!(2, g.vertex_count());
    /// assert_eq!(vec!["a", "b"], labels);
    /// ```
    pub fn add_edge(&mut self, u: V, v: V, payload: T) {
        let src = self.id_of(&u);
        let dst = self.id_of(&v);
        self.edges.push((src, dst, payload));
    }

    /// これまでに登録した頂点・辺から `Graph<T>` を構築し、採番した頂点番号から
    /// 元のラベルを引ける対応表とあわせて返す。
    ///
    /// # Returns
    /// `(Graph<T>, Vec<V>)`: 構築されたグラフと、頂点番号 `i` に対応する元の
    /// ラベル `labels[i]` の対応表。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    ///   - V は登録された頂点数、E は登録された辺数である。
    /// - 空間計算量: O(V + E)
    #[must_use]
    pub fn build(self) -> (graph::Graph<T>, Vec<V>) {
        let mut g = graph::Graph::new(self.labels.len());
        for (src, dst, payload) in self.edges {
            g.add_edge(src, dst, payload);
        }
        (g, self.labels)
    }
}

impl<V, T> GraphBuilder<V, T>
where
    V: Clone + Eq + hash::Hash,
    T: Clone,
{
    /// 無向辺 `{u, v}` を追加する。内部的には `u -> v` と `v -> u` の2本の
    /// 有向辺として登録する。
    ///
    /// # Args
    /// - `u`: 辺の一方の端点のラベル。
    /// - `v`: 辺のもう一方の端点のラベル。
    /// - `payload`: 辺に付与する任意の値。両方向の辺に複製して持たせる。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    pub fn add_undirected_edge(&mut self, u: V, v: V, payload: T) {
        let src = self.id_of(&u);
        let dst = self.id_of(&v);
        self.edges.push((src, dst, payload.clone()));
        self.edges.push((dst, src, payload));
    }
}

impl<V, T> Default for GraphBuilder<V, T>
where
    V: Clone + Eq + hash::Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // add_edge/build のテスト: 構築されるグラフと対応表を検証する。
    mod add_edge {
        use super::*;

        /// Scenario: 初出のラベルには、登場順に頂点番号が採番される。
        /// - Given: 空のビルダーがある。
        /// - When: `"b" -> "a"`、続けて `"a" -> "c"` の順に辺を追加する。
        /// - Then: 対応表は登場順 `["b", "a", "c"]` になり、グラフは
        ///   3頂点・2辺を持つ。
        #[test]
        fn assigns_ids_in_order_of_first_appearance() {
            // Given
            let mut sut = GraphBuilder::new();
            // When
            sut.add_edge("b", "a", ());
            sut.add_edge("a", "c", ());
            // Then
            let (g, labels) = sut.build();
            assert_eq!(vec!["b", "a", "c"], labels);
            assert_eq!(3, g.vertex_count());
            assert_eq!(
                1,
                g.out_degree(labels.iter().position(|&l| l == "b").unwrap())
            );
        }

        /// Scenario: 既出のラベルには、同じ頂点番号が再利用される。
        /// - Given: 空のビルダーがある。
        /// - When: `"a" -> "b"` を追加した後、再び `"a"` を始点に別の辺を追加する。
        /// - Then: 頂点数は2のままであり、`"a"` の出次数が2になる。
        #[test]
        fn reuses_id_for_already_seen_label() {
            // Given
            let mut sut = GraphBuilder::new();
            // When
            sut.add_edge("a", "b", ());
            sut.add_edge("a", "b", ());
            // Then
            let (g, labels) = sut.build();
            assert_eq!(2, labels.len());
            assert_eq!(2, g.out_degree(0));
        }

        /// Scenario: 辺を1本も追加しなければ、空のグラフになる (境界値)。
        /// - Given: 空のビルダーがある。
        /// - When: 何も追加せずに構築する。
        /// - Then: グラフは頂点数0であり、対応表も空である。
        #[test]
        fn builds_empty_graph_when_no_edge_added() {
            // Given
            let sut = GraphBuilder::<&str, ()>::new();
            // When
            let (g, labels) = sut.build();
            // Then
            assert_eq!(0, g.vertex_count());
            assert!(labels.is_empty());
        }

        /// Scenario: ペイロードは辺とともに正しく記録される。
        /// - Given: 空のビルダーがある。
        /// - When: ペイロード `10` を持つ辺 `"a" -> "b"` を追加する。
        /// - Then: `"a"` から出る辺のペイロードが `10` である。
        #[test]
        fn records_payload() {
            // Given
            let mut sut = GraphBuilder::new();
            // When
            sut.add_edge("a", "b", 10_u32);
            // Then
            let (g, _labels) = sut.build();
            let result = g.edges(0).map(|(_, p)| *p).collect::<Vec<u32>>();
            assert_eq!(vec![10], result);
        }
    }

    // add_undirected_edge のテスト: 構築されるグラフを検証する。
    mod add_undirected_edge {
        use super::*;

        /// Scenario: 無向辺を追加すると、両端点の出次数がそれぞれ増える。
        /// - Given: 空のビルダーがある。
        /// - When: `{"a", "b"}` に無向辺を追加する。
        /// - Then: 構築されたグラフで、"a" と "b" の出次数がともに1になる。
        #[test]
        fn increases_out_degree_of_both_endpoints() {
            // Given
            let mut sut = GraphBuilder::new();
            // When
            sut.add_undirected_edge("a", "b", ());
            // Then
            let (g, _labels) = sut.build();
            assert_eq!(1, g.out_degree(0));
            assert_eq!(1, g.out_degree(1));
        }
    }
}

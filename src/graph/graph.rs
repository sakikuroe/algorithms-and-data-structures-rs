//! グラフを可変な隣接リストとして表現するデータ構造を提供するモジュールである。
//!
//! 各辺には任意のペイロード `T` を持たせられる。重みなしグラフには `T = ()` を、
//! 重み付きグラフには `T = u32` のような数値型を、重み以外の情報 (色・ラベル・
//! 元の入力での辺番号等) も併せ持たせたい場合は独自の構造体を `T` に割り当てる。
//! 頂点番号は内部では `u32` として保持し、メモリとキャッシュ効率を優先する一方、
//! 公開 API では `usize` を受け渡しし、他のデータ構造モジュールとの一貫性を保つ。

/// 有向グラフを、頂点ごとの隣接リストとして表現する。
pub struct Graph<T> {
    /// `edges[u]` は、頂点 `u` から出る `(行き先, ペイロード)` の並びである。
    pub(super) edges: Vec<Vec<(u32, T)>>,
}

impl<T> Graph<T> {
    /// `n` 頂点、辺を持たないグラフを生成する。
    ///
    /// # Args
    /// - `n`: 頂点数
    ///
    /// # Returns
    /// `Self`: 頂点数 `n`、辺を持たない `Graph<T>`
    ///
    /// # Complexity
    /// - 時間計算量: O(n)
    /// - 空間計算量: O(n)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let g = Graph::<()>::new(3);
    /// assert_eq!(3, g.vertex_count());
    /// ```
    #[must_use]
    pub fn new(n: usize) -> Self {
        // vec![Vec::new(); n] は Vec<T> の Clone (延いては T: Clone) を要求してしまう。
        // T に不要な制約を課さないよう、n 回 Vec::new() を呼び出す形で初期化する。
        Graph {
            edges: (0..n).map(|_| Vec::new()).collect::<Vec<Vec<(u32, T)>>>(),
        }
    }

    /// グラフの頂点数を返す。
    ///
    /// # Returns
    /// `usize`: 頂点数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let g = Graph::<()>::new(5);
    /// assert_eq!(5, g.vertex_count());
    /// ```
    pub fn vertex_count(&self) -> usize {
        self.edges.len()
    }

    /// 頂点 `u` の出次数 (`u` から出る辺の本数) を返す。
    ///
    /// # Args
    /// - `u`: 頂点番号であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `usize`: `u` から出る辺の本数
    ///
    /// # Panics
    /// - `u` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(0, 2, ());
    /// assert_eq!(2, g.out_degree(0));
    /// assert_eq!(0, g.out_degree(1));
    /// ```
    pub fn out_degree(&self, u: usize) -> usize {
        self.edges[u].len()
    }

    /// 頂点 `u` から出る辺を `(行き先, ペイロードへの参照)` の列として返す。
    ///
    /// # Args
    /// - `u`: 頂点番号であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `impl Iterator<Item = (usize, &T)>`: `u` から出る辺の `(行き先, ペイロード)` を、
    /// 追加した順に列挙するイテレータ
    ///
    /// # Panics
    /// - `u` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: `u` の出次数に比例する `O(deg(u))` (イテレータの消費全体では)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(2);
    /// g.add_edge(0, 1, 10_u32);
    /// let edges = g.edges(0).collect::<Vec<(usize, &u32)>>();
    /// assert_eq!(vec![(1, &10)], edges);
    /// ```
    pub fn edges(&self, u: usize) -> impl Iterator<Item = (usize, &T)> {
        self.edges[u]
            .iter()
            .map(|(dst, payload)| (*dst as usize, payload))
    }

    /// 有向辺 `src -> dst` を、ペイロード `payload` とともに追加する。
    ///
    /// # Args
    /// - `src`: 辺の始点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `dst`: 辺の終点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `payload`: 辺に付与する任意の値 (重み・色・ラベルなど)
    ///
    /// # Panics
    /// - `src` が `0..vertex_count()` の範囲外の場合にパニックする。
    /// - デバッグビルドに限り、`dst` が `0..vertex_count()` の範囲外の場合にもパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(2);
    /// g.add_edge(0, 1, 'a');
    /// assert_eq!(1, g.out_degree(0));
    /// ```
    pub fn add_edge(&mut self, src: usize, dst: usize, payload: T) {
        debug_assert!(dst < self.vertex_count());
        self.edges[src].push((dst as u32, payload));
    }
}

impl<T: Clone> Graph<T> {
    /// 無向辺 `{u, v}` を追加する。内部的には `u -> v` と `v -> u` の2本の有向辺
    /// として保持する。
    ///
    /// # Args
    /// - `u`: 辺の一方の端点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `v`: 辺のもう一方の端点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `payload`: 辺に付与する任意の値であり、両方向の辺に複製して持たせる。
    ///
    /// # Panics
    /// - `u`/`v` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(2);
    /// g.add_undirected_edge(0, 1, 'a');
    /// assert_eq!(1, g.out_degree(0));
    /// assert_eq!(1, g.out_degree(1));
    /// ```
    pub fn add_undirected_edge(&mut self, u: usize, v: usize, payload: T) {
        self.add_edge(u, v, payload.clone());
        self.add_edge(v, u, payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 5頂点、辺を持たないグラフ。
    fn create_empty_graph() -> Graph<()> {
        Graph::new(5)
    }

    // new のテスト: 生成直後の状態を検証する。
    mod new {
        use super::*;

        /// Scenario: 生成直後のグラフは、指定した頂点数を持ち、辺を持たない。
        /// - Given: 頂点数 5 を指定する。
        /// - When: `Graph::new(5)` を呼ぶ。
        /// - Then: `vertex_count()` が 5 を返し、すべての頂点の出次数が 0 である。
        #[test]
        fn creates_graph_with_given_vertex_count_and_no_edges() {
            // Given, When
            let sut = create_empty_graph();
            // Then
            assert_eq!(5, sut.vertex_count());
            for u in 0..5 {
                assert_eq!(0, sut.out_degree(u));
            }
        }

        /// Scenario: 頂点数 0 で生成すると、空のグラフになる (境界値)。
        /// - Given: 頂点数 0 を指定する。
        /// - When: `Graph::new(0)` を呼ぶ。
        /// - Then: `vertex_count()` が 0 を返す。
        #[test]
        fn creates_empty_graph_for_zero_vertices() {
            // Given, When
            let sut = Graph::<()>::new(0);
            // Then
            assert_eq!(0, sut.vertex_count());
        }
    }

    // add_edge のテスト: 呼び出し後の状態変化を検証する。
    mod add_edge {
        use super::*;

        /// Scenario: 有向辺を追加すると、始点の出次数のみが増える。
        /// - Given: 3頂点、辺を持たないグラフがある。
        /// - When: `0 -> 1` に辺を追加する。
        /// - Then: 頂点 0 の出次数が 1 になり、頂点 1 の出次数は 0 のままである。
        #[test]
        fn increases_out_degree_of_source_only() {
            // Given
            let mut sut = create_empty_graph();
            // When
            sut.add_edge(0, 1, ());
            // Then
            assert_eq!(1, sut.out_degree(0));
            assert_eq!(0, sut.out_degree(1));
        }

        /// Scenario: 追加した辺の行き先とペイロードが記録される。
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `0 -> 1` にペイロード `42` を持つ辺を追加する。
        /// - Then: `edges(0)` が `(1, &42)` を1件だけ返す。
        #[test]
        fn records_destination_and_payload() {
            // Given
            let mut sut = Graph::new(2);
            // When
            sut.add_edge(0, 1, 42_u32);
            // Then
            let result = sut.edges(0).collect::<Vec<(usize, &u32)>>();
            assert_eq!(vec![(1, &42)], result);
        }

        /// Scenario: 自己ループを追加できる (境界値)。
        /// - Given: 1頂点、辺を持たないグラフがある。
        /// - When: `0 -> 0` に辺を追加する。
        /// - Then: 頂点 0 の出次数が 1 になる。
        #[test]
        fn allows_self_loop() {
            // Given
            let mut sut = Graph::new(1);
            // When
            sut.add_edge(0, 0, ());
            // Then
            assert_eq!(1, sut.out_degree(0));
        }

        /// Scenario: 同じ2頂点間に複数の辺を追加できる (多重辺)。
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `0 -> 1` に辺を2本追加する。
        /// - Then: 頂点 0 の出次数が 2 になる。
        #[test]
        fn allows_multi_edge() {
            // Given
            let mut sut = Graph::new(2);
            // When
            sut.add_edge(0, 1, ());
            sut.add_edge(0, 1, ());
            // Then
            assert_eq!(2, sut.out_degree(0));
        }
    }

    // add_undirected_edge のテスト: 呼び出し後の状態変化を検証する。
    mod add_undirected_edge {
        use super::*;

        /// Scenario: 無向辺を追加すると、両端点の出次数がそれぞれ増える。
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `{0, 1}` に無向辺を追加する。
        /// - Then: 頂点 0, 1 の出次数がともに 1 になる。
        #[test]
        fn increases_out_degree_of_both_endpoints() {
            // Given
            let mut sut = Graph::new(2);
            // When
            sut.add_undirected_edge(0, 1, ());
            // Then
            assert_eq!(1, sut.out_degree(0));
            assert_eq!(1, sut.out_degree(1));
        }

        /// Scenario: 両方向の辺に同じペイロードが複製される。
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `{0, 1}` にペイロード `7` を持つ無向辺を追加する。
        /// - Then: `0 -> 1` と `1 -> 0` の両方に `7` が記録される。
        #[test]
        fn duplicates_payload_to_both_directions() {
            // Given
            let mut sut = Graph::new(2);
            // When
            sut.add_undirected_edge(0, 1, 7_u32);
            // Then
            assert_eq!(vec![(1, &7)], sut.edges(0).collect::<Vec<(usize, &u32)>>());
            assert_eq!(vec![(0, &7)], sut.edges(1).collect::<Vec<(usize, &u32)>>());
        }
    }

    // out_degree のテスト: 戻り値そのものを検証する。
    mod out_degree {
        use super::*;

        /// Scenario: 辺を持たない頂点の出次数は 0 である (境界値)。
        /// - Given: 頂点を1つ持ち、辺を持たないグラフがある。
        /// - When: その頂点の出次数を求める。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_for_vertex_without_edges() {
            // Given
            let sut = Graph::<()>::new(1);
            // When
            let result = sut.out_degree(0);
            // Then
            assert_eq!(0, result);
        }
    }
}

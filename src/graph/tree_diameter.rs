//! 木の直径 (最も離れた2頂点間の距離) を求める機能を提供するモジュールである。
//!
//! 木上の最遠点を求める古典的な性質、すなわち「任意の頂点から最も遠い頂点の
//! 1つは、直径の端点になっている」ことを利用し、2回の探索 (BFS または
//! Dijkstra 法) で直径を求める。1回目の探索で任意の頂点 (頂点0) から最も
//! 遠い頂点 `u` を求め、2回目の探索で `u` から最も遠い頂点までの距離が、
//! そのまま木の直径になる。
//!
//! [`hld`](super::hld) モジュールが提供する `Hld::distance` は根からの深さに
//! 基づく辺数のみを扱うのに対し、このモジュールは辺の重みを考慮した距離
//! (または辺数そのもの) を独立に扱う。木構造の検証やパス分解を必要としない
//! 単発の問い合わせであるため、`Hld` には依存しない。

use super::graph;

impl<T> graph::Graph<T> {
    /// 辺の重みを無視し、辺数を距離として木の直径を求める。
    ///
    /// # Returns
    /// `Result<usize, NotATreeError>`: 木であれば、最も離れた2頂点間の距離
    /// (辺数)。木でなければ、その理由を表すエラー。頂点数が0の場合は
    /// `Ok(0)` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。木であることの検証、および2回の BFS がいずれも
    ///     O(V) で収まる。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1-2-3 の単純パス。直径は両端点間の距離 3。
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    /// g.add_undirected_edge(2, 3, ());
    ///
    /// assert_eq!(Ok(3), g.try_tree_diameter_unweighted());
    /// ```
    pub fn try_tree_diameter_unweighted(&self) -> Result<usize, graph::NotATreeError> {
        let n = self.vertex_count();
        if n == 0 {
            return Ok(0);
        }
        graph::validate_tree(self, 0)?;

        let first = self.bfs(&[0]);
        let farthest = (0..n)
            .max_by_key(|&v| first.distance(v))
            .expect("n > 0 here");

        let second = self.bfs(&[farthest]);
        let diameter = (0..n)
            .filter_map(|v| second.distance(v))
            .max()
            .expect("farthest itself is always reachable from itself");

        Ok(diameter)
    }

    /// 辺の重みを無視し、辺数を距離として木の直径と、直径を実現する経路を
    /// 求める。
    ///
    /// # Returns
    /// `Result<(usize, Vec<usize>), NotATreeError>`: 木であれば、直径
    /// (辺数) と、その直径を実現する2頂点間の経路 (端点を含む頂点列)。
    /// 木でなければ、その理由を表すエラー。頂点数が0の場合は
    /// `Ok((0, Vec::new()))` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1-2-3 の単純パス。直径は両端点間の距離 3。
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    /// g.add_undirected_edge(2, 3, ());
    ///
    /// let (diameter, path) = g.try_tree_diameter_path_unweighted().unwrap();
    /// assert_eq!(3, diameter);
    /// assert_eq!(4, path.len());
    /// ```
    pub fn try_tree_diameter_path_unweighted(
        &self,
    ) -> Result<(usize, Vec<usize>), graph::NotATreeError> {
        let n = self.vertex_count();
        if n == 0 {
            return Ok((0, Vec::new()));
        }
        graph::validate_tree(self, 0)?;

        let first = self.bfs(&[0]);
        let one_end = (0..n)
            .max_by_key(|&v| first.distance(v))
            .expect("n > 0 here");

        let second = self.bfs(&[one_end]);
        let other_end = (0..n)
            .max_by_key(|&v| second.distance(v))
            .expect("n > 0 here");
        let diameter = second
            .distance(other_end)
            .expect("farthest itself is always reachable from itself");
        let path = second
            .path_to(other_end)
            .expect("other_end is always reachable from one_end");

        Ok((diameter, path))
    }
}

impl<T> graph::Graph<T> {
    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、重み付きの
    /// 木の直径を求める。
    ///
    /// ペイロード `T` が重み以外の情報も併せ持つ場合に使う。ペイロード自体が
    /// 重みである場合は [`try_tree_diameter`](Self::try_tree_diameter) の方が
    /// 簡潔に書ける。
    ///
    /// # Args
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す関数。
    ///
    /// # Returns
    /// `Result<W, NotATreeError>`: 木であれば、最も離れた2頂点間の距離
    /// (最短路の重みの和)。木でなければ、その理由を表すエラー。頂点数が0の
    /// 場合は `Ok(W::default())` を返す。
    ///
    /// # Constraints
    /// - `weight_of` はすべての辺に対して非負の値を返さなければならない。
    /// - `W::default()` が加法の単位元 (0 に相当する値) でなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。木であることの検証は O(V)、2回の Dijkstra 法が
    ///     それぞれ O(V log V) で収まる (木であるため辺数は V-1)。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // ペイロードが (重み, 元の辺番号) の組であるケース。
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, (3_u32, 0_usize));
    /// g.add_undirected_edge(1, 2, (4_u32, 1_usize));
    ///
    /// assert_eq!(Ok(7), g.try_tree_diameter_by(|&(w, _id)| w));
    /// ```
    pub fn try_tree_diameter_by<W, F>(&self, weight_of: F) -> Result<W, graph::NotATreeError>
    where
        W: Copy + Ord + std::ops::Add<Output = W> + Default,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();
        if n == 0 {
            return Ok(W::default());
        }
        graph::validate_tree(self, 0)?;

        let first = self.dijkstra_by(&[(0, W::default())], &weight_of);
        let farthest = (0..n)
            .max_by_key(|&v| first.distance(v))
            .expect("n > 0 here");

        let second = self.dijkstra_by(&[(farthest, W::default())], &weight_of);
        let diameter = (0..n)
            .filter_map(|v| second.distance(v))
            .max()
            .expect("farthest itself is always reachable from itself");

        Ok(diameter)
    }

    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、重み付き
    /// の木の直径と、直径を実現する経路を求める。
    ///
    /// # Args
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す
    ///   関数。
    ///
    /// # Returns
    /// `Result<(W, Vec<usize>), NotATreeError>`: 木であれば、直径 (最短路
    /// の重みの和) と、その直径を実現する2頂点間の経路 (端点を含む頂点列)。
    /// 木でなければ、その理由を表すエラー。頂点数が0の場合は
    /// `Ok((W::default(), Vec::new()))` を返す。
    ///
    /// # Constraints
    /// - `weight_of` はすべての辺に対して非負の値を返さなければならない。
    /// - `W::default()` が加法の単位元 (0 に相当する値) でなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, (3_u32, 0_usize));
    /// g.add_undirected_edge(1, 2, (4_u32, 1_usize));
    ///
    /// let (diameter, path) = g.try_tree_diameter_path_by(|&(w, _id)| w).unwrap();
    /// assert_eq!(7, diameter);
    /// assert_eq!(vec![2, 1, 0], path);
    /// ```
    pub fn try_tree_diameter_path_by<W, F>(
        &self,
        weight_of: F,
    ) -> Result<(W, Vec<usize>), graph::NotATreeError>
    where
        W: Copy + Ord + std::ops::Add<Output = W> + Default,
        F: Fn(&T) -> W,
    {
        let n = self.vertex_count();
        if n == 0 {
            return Ok((W::default(), Vec::new()));
        }
        graph::validate_tree(self, 0)?;

        let first = self.dijkstra_by(&[(0, W::default())], &weight_of);
        let one_end = (0..n)
            .max_by_key(|&v| first.distance(v))
            .expect("n > 0 here");

        let second = self.dijkstra_by(&[(one_end, W::default())], &weight_of);
        let other_end = (0..n)
            .max_by_key(|&v| second.distance(v))
            .expect("n > 0 here");
        let diameter = second
            .distance(other_end)
            .expect("farthest itself is always reachable from itself");
        let path = second
            .path_to(other_end)
            .expect("other_end is always reachable from one_end");

        Ok((diameter, path))
    }
}

impl<T: Copy + Ord + std::ops::Add<Output = T> + Default> graph::Graph<T> {
    /// 辺のペイロード自体を重みとして扱い、木の直径を求める。
    ///
    /// 重みの型 `T` を自分自身から取り出すだけでよい場合の簡易版であり、
    /// 重み以外の情報も併せ持つペイロードから重みを取り出したい場合は
    /// [`try_tree_diameter_by`](Self::try_tree_diameter_by) を使う。
    ///
    /// # Returns
    /// `Result<T, NotATreeError>`: 木であれば、最も離れた2頂点間の距離。
    /// 木でなければ、その理由を表すエラー。頂点数が0の場合は
    /// `Ok(T::default())` を返す。
    ///
    /// # Constraints
    /// - すべての辺の重みが非負でなければならない。
    /// - `T::default()` が加法の単位元 (0 に相当する値) でなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, 3_u32);
    /// g.add_undirected_edge(1, 2, 4_u32);
    ///
    /// assert_eq!(Ok(7), g.try_tree_diameter());
    /// ```
    pub fn try_tree_diameter(&self) -> Result<T, graph::NotATreeError> {
        self.try_tree_diameter_by(|&w| w)
    }

    /// 辺のペイロード自体を重みとして扱い、木の直径と、直径を実現する経路
    /// を求める。
    ///
    /// # Returns
    /// `Result<(T, Vec<usize>), NotATreeError>`: 木であれば、直径と、その
    /// 直径を実現する2頂点間の経路 (端点を含む頂点列)。木でなければ、
    /// その理由を表すエラー。頂点数が0の場合は `Ok((T::default(), Vec::new()))`
    /// を返す。
    ///
    /// # Constraints
    /// - すべての辺の重みが非負でなければならない。
    /// - `T::default()` が加法の単位元 (0 に相当する値) でなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, 3_u32);
    /// g.add_undirected_edge(1, 2, 4_u32);
    ///
    /// let (diameter, path) = g.try_tree_diameter_path().unwrap();
    /// assert_eq!(7, diameter);
    /// assert_eq!(vec![2, 1, 0], path);
    /// ```
    pub fn try_tree_diameter_path(&self) -> Result<(T, Vec<usize>), graph::NotATreeError> {
        self.try_tree_diameter_path_by(|&w| w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // try_tree_diameter_unweighted のテスト: 戻り値そのものを検証する。
    mod try_tree_diameter_unweighted {
        use super::*;

        /// Scenario: 単純パスの直径は、両端点間の辺数になる。
        /// - Given: 0-1-2-3 の単純パス (4頂点) がある。
        /// - When: 木の直径 (辺数) を求める。
        /// - Then: 3 が返る。
        #[test]
        fn returns_hop_count_between_endpoints_for_path() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Ok(3), result);
        }

        /// Scenario: 星グラフの直径は、2枚の葉を経由した距離 (2) になる。
        /// - Given: 頂点0を中心とする星グラフ (5頂点) がある。
        /// - When: 木の直径を求める。
        /// - Then: 2 が返る (葉 -> 中心 -> 別の葉)。
        #[test]
        fn returns_two_for_star_graph() {
            // Given
            let mut sut = graph::Graph::new(5);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(0, 2, ());
            sut.add_undirected_edge(0, 3, ());
            sut.add_undirected_edge(0, 4, ());
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Ok(2), result);
        }

        /// Scenario: 頂点が1つだけの木の直径は0になる (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 木の直径を求める。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_for_single_vertex() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Ok(0), result);
        }

        /// Scenario: 頂点数が0のグラフでも0を返す (境界値)。
        /// - Given: 頂点数0のグラフがある。
        /// - When: 木の直径を求める。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_for_empty_graph() {
            // Given
            let sut = graph::Graph::<()>::new(0);
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Ok(0), result);
        }

        /// Scenario: 非連結なグラフに対しては、`Disconnected` エラーが返る。
        /// - Given: 孤立した頂点を含む、非連結なグラフがある。
        /// - When: 木の直径を求める。
        /// - Then: `Err(NotATreeError::Disconnected)` が返る。
        #[test]
        fn returns_disconnected_error_for_disconnected_graph() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            // 頂点2は孤立している。
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Err(graph::NotATreeError::Disconnected), result);
        }

        /// Scenario: 閉路を含むグラフに対しては、`HasCycle` エラーが返る。
        /// - Given: 0-1-2-0 の閉路を含むグラフがある。
        /// - When: 木の直径を求める。
        /// - Then: `Err(NotATreeError::HasCycle)` が返る。
        #[test]
        fn returns_has_cycle_error_for_graph_with_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            // When
            let result = sut.try_tree_diameter_unweighted();
            // Then
            assert_eq!(Err(graph::NotATreeError::HasCycle), result);
        }
    }

    // try_tree_diameter_path_unweighted のテスト: 戻り値そのもの (直径と経路)
    // を検証する。
    mod try_tree_diameter_path_unweighted {
        use super::*;

        /// Scenario: 直径だけでなく、それを実現する経路も得られる。
        /// - Given: 0-1-2-3 の単純パス (4頂点) がある。
        /// - When: 木の直径と経路を求める。
        /// - Then: 直径は3になり、経路は両端点である頂点0と頂点3を含む
        ///   4頂点からなる。
        #[test]
        fn returns_path_reaching_both_endpoints() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            // When
            let (diameter, path) = sut.try_tree_diameter_path_unweighted().unwrap();
            // Then
            assert_eq!(3, diameter);
            assert_eq!(4, path.len());
            assert!(path.contains(&0));
            assert!(path.contains(&3));
        }

        /// Scenario: 頂点が1つだけの木では、経路も頂点1つだけになる
        /// (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 木の直径と経路を求める。
        /// - Then: 直径は0になり、経路は頂点0のみからなる。
        #[test]
        fn returns_single_vertex_path_for_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let (diameter, path) = sut.try_tree_diameter_path_unweighted().unwrap();
            // Then
            assert_eq!(0, diameter);
            assert_eq!(vec![0], path);
        }

        /// Scenario: 頂点数が0のグラフでは、経路も空になる (境界値)。
        /// - Given: 頂点数0のグラフがある。
        /// - When: 木の直径と経路を求める。
        /// - Then: 直径は0になり、経路は空になる。
        #[test]
        fn returns_empty_path_for_empty_graph() {
            // Given
            let sut = graph::Graph::<()>::new(0);
            // When
            let (diameter, path) = sut.try_tree_diameter_path_unweighted().unwrap();
            // Then
            assert_eq!(0, diameter);
            assert!(path.is_empty());
        }
    }

    // try_tree_diameter のテスト: 戻り値そのものを検証する。
    mod try_tree_diameter {
        use super::*;

        /// Scenario: 単純パスの直径は、両端点間の辺重みの総和になる。
        /// - Given: 0-1 (重み3), 1-2 (重み4) の単純パスがある。
        /// - When: 木の直径を求める。
        /// - Then: 7 が返る。
        #[test]
        fn returns_sum_of_weights_between_endpoints_for_path() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, 3_u32);
            sut.add_undirected_edge(1, 2, 4_u32);
            // When
            let result = sut.try_tree_diameter();
            // Then
            assert_eq!(Ok(7), result);
        }

        /// Scenario: 直径は、必ずしも最も重い1本の枝だけでなく、複数の枝を
        /// 経由する経路が最長になることもある。
        /// - Given: 頂点0を中心に、重み1の枝1本と、重み10・重み20からなる
        ///   長さ2のパスを持つ木がある。
        /// - When: 木の直径を求める。
        /// - Then: 31 が返る (重み1の葉から、重み10・20のパスの末端までの
        ///   経路 1+10+20 が最長になる)。
        #[test]
        fn finds_longest_path_across_branches() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, 1_u32);
            sut.add_undirected_edge(0, 2, 10_u32);
            sut.add_undirected_edge(2, 3, 20_u32);
            // When
            let result = sut.try_tree_diameter();
            // Then
            assert_eq!(Ok(31), result);
        }

        /// Scenario: 頂点が1つだけの木の直径は0になる (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 木の直径を求める。
        /// - Then: 0 が返る。
        #[test]
        fn returns_zero_for_single_vertex() {
            // Given
            let sut = graph::Graph::<u32>::new(1);
            // When
            let result = sut.try_tree_diameter();
            // Then
            assert_eq!(Ok(0), result);
        }
    }

    // try_tree_diameter_by のテスト: 戻り値そのものを検証する。
    mod try_tree_diameter_by {
        use super::*;

        /// Scenario: ペイロードから射影した重みを使って直径を求められる。
        /// - Given: ペイロードが `(重み, 元の辺番号)` の組である木がある。
        /// - When: 重みだけを取り出す関数を渡して木の直径を求める。
        /// - Then: 射影した重みに基づく直径が得られる。
        #[test]
        fn computes_diameter_using_projected_weight() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, (3_u32, 0_usize));
            sut.add_undirected_edge(1, 2, (4_u32, 1_usize));
            // When
            let result = sut.try_tree_diameter_by(|&(w, _id)| w);
            // Then
            assert_eq!(Ok(7), result);
        }
    }

    // try_tree_diameter_path_by のテスト: 戻り値そのもの (直径と経路) を
    // 検証する。
    mod try_tree_diameter_path_by {
        use super::*;

        /// Scenario: ペイロードから射影した重みを使って、直径とその経路
        /// を求められる。
        /// - Given: ペイロードが `(重み, 元の辺番号)` の組である、
        ///   0-1 (重み3), 1-2 (重み4) の単純パスがある。
        /// - When: 重みだけを取り出す関数を渡して、木の直径と経路を求める。
        /// - Then: 直径は7になり、経路は両端点である頂点0と頂点2を含む
        ///   3頂点からなる。
        #[test]
        fn computes_diameter_and_path_using_projected_weight() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, (3_u32, 0_usize));
            sut.add_undirected_edge(1, 2, (4_u32, 1_usize));
            // When
            let (diameter, path) = sut.try_tree_diameter_path_by(|&(w, _id)| w).unwrap();
            // Then
            assert_eq!(7, diameter);
            assert_eq!(3, path.len());
            assert!(path.contains(&0));
            assert!(path.contains(&2));
        }
    }

    // try_tree_diameter_path のテスト: 戻り値そのもの (直径と経路) を検証する。
    mod try_tree_diameter_path {
        use super::*;

        /// Scenario: 直径だけでなく、それを実現する経路も得られる。
        /// - Given: 0-1 (重み3), 1-2 (重み4) の単純パスがある。
        /// - When: 木の直径と経路を求める。
        /// - Then: 直径は7になり、経路は両端点である頂点0と頂点2を含む
        ///   3頂点からなる。
        #[test]
        fn returns_path_reaching_both_endpoints() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, 3_u32);
            sut.add_undirected_edge(1, 2, 4_u32);
            // When
            let (diameter, path) = sut.try_tree_diameter_path().unwrap();
            // Then
            assert_eq!(7, diameter);
            assert_eq!(3, path.len());
            assert!(path.contains(&0));
            assert!(path.contains(&2));
        }

        /// Scenario: 頂点が1つだけの木では、経路も頂点1つだけになる
        /// (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 木の直径と経路を求める。
        /// - Then: 直径は0になり、経路は頂点0のみからなる。
        #[test]
        fn returns_single_vertex_path_for_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<u32>::new(1);
            // When
            let (diameter, path) = sut.try_tree_diameter_path().unwrap();
            // Then
            assert_eq!(0, diameter);
            assert_eq!(vec![0], path);
        }
    }
}

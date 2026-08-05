//! 深さ優先探索 (DFS) を提供するモジュールである。
//!
//! 辺のペイロード `T` には一切関与しないため、重みなし・重み付きいずれの
//! グラフに対しても、グラフの走査順序のみを扱う目的でそのまま使える。

use super::graph;

/// スタック上で処理待ちの頂点を表す。`Enter` は初訪問時、`Leave` はその頂点の
/// 子をすべて訪れ終えた後の処理を表す。
enum Frame {
    Enter(usize),
    Leave(usize),
}

/// DFS の行きがけ順・帰りがけ順を保持する。
pub struct Dfs {
    /// 行きがけ順 (各頂点に最初に訪れた順) に並んだ頂点列。
    preorder: Vec<usize>,
    /// 帰りがけ順 (各頂点の子をすべて訪れ終えて後戻りした順) に並んだ頂点列。
    postorder: Vec<usize>,
}

impl Dfs {
    /// 行きがけ順 (各頂点に最初に訪れた順) の頂点列を返す。
    ///
    /// # Returns
    /// `&[usize]`: 行きがけ順の頂点列
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn preorder(&self) -> &[usize] {
        &self.preorder
    }

    /// 帰りがけ順 (各頂点の子をすべて訪れ終えて後戻りした順) の頂点列を返す。
    ///
    /// # Returns
    /// `&[usize]`: 帰りがけ順の頂点列
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn postorder(&self) -> &[usize] {
        &self.postorder
    }
}

impl<T> graph::Graph<T> {
    /// 単一始点からの深さ優先探索を行い、行きがけ順・帰りがけ順を記録する。
    ///
    /// 明示的なスタックを用いた非再帰の実装であり、頂点数が大きいグラフに
    /// 対しても、再帰呼び出しによるスタックオーバーフローを起こさない。
    ///
    /// # Args
    /// - `start`: 探索の始点となる頂点であり、`0..vertex_count()` の範囲でなければ
    ///   ならない。
    ///
    /// # Returns
    /// `Dfs`: `start` から到達可能な頂点の行きがけ順・帰りがけ順であり、到達できない
    /// 頂点は含まれない。
    ///
    /// # Panics
    /// - `start` が `0..vertex_count()` の範囲外の場合にパニックする。
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
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    /// g.add_edge(1, 2, ());
    ///
    /// let dfs = g.dfs(0);
    /// assert_eq!(&[0, 1, 2], dfs.preorder());
    /// assert_eq!(&[2, 1, 0], dfs.postorder());
    /// ```
    pub fn dfs(&self, start: usize) -> Dfs {
        let mut visited = vec![false; self.vertex_count()];
        let mut preorder = Vec::new();
        let mut postorder = Vec::new();
        let mut stack = vec![Frame::Enter(start)];
        visited[start] = true;

        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Enter(u) => {
                    preorder.push(u);
                    // u の子をすべて訪れ終えた後に postorder へ積むため、
                    // 先に Leave(u) を積んでおく。
                    stack.push(Frame::Leave(u));
                    for (v, _) in self.edges(u) {
                        if !visited[v] {
                            visited[v] = true;
                            stack.push(Frame::Enter(v));
                        }
                    }
                }
                Frame::Leave(u) => {
                    postorder.push(u);
                }
            }
        }

        Dfs {
            preorder,
            postorder,
        }
    }
}

/// グラフ全体 (森) を対象とした深さ優先探索の結果を保持する。
///
/// 頂点 `0..vertex_count()` を昇順に走査し、未訪問の頂点が見つかるたびに
/// そこを根とする新たな木として深さ優先探索を開始する。連結でないグラフでも
/// すべての頂点を訪問できる。
pub struct DfsForest {
    /// 行きがけ順 (各頂点に最初に訪れた順) に並んだ頂点列。
    preorder: Vec<usize>,
    /// 帰りがけ順 (各頂点の子をすべて訪れ終えて後戻りした順) に並んだ頂点列。
    postorder: Vec<usize>,
    /// `discover_time[v]` は、頂点 `v` を最初に訪れた時刻 (1 から始まり、頂点の
    /// 発見・完了のたびに 1 ずつ増えるグローバルなカウンター)。
    discover_time: Vec<usize>,
    /// `finish_time[v]` は、頂点 `v` の子をすべて訪れ終えた時刻。
    finish_time: Vec<usize>,
    /// `depth[v]` は、`v` を含む木における根からの深さ (辺数)。
    depth: Vec<usize>,
    /// オイラーツアー: 頂点に最初に訪れたとき、および子から親へ戻るたびに、
    /// そのときの頂点を記録した列。
    euler_tour: Vec<usize>,
    /// `first_occurrence[v]` は、`v` が `euler_tour` の中で最初に現れる位置。
    first_occurrence: Vec<usize>,
}

impl DfsForest {
    /// 行きがけ順 (各頂点に最初に訪れた順) の頂点列を返す。
    ///
    /// # Returns
    /// `&[usize]`: 行きがけ順の頂点列
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn preorder(&self) -> &[usize] {
        &self.preorder
    }

    /// 帰りがけ順 (各頂点の子をすべて訪れ終えて後戻りした順) の頂点列を返す。
    ///
    /// # Returns
    /// `&[usize]`: 帰りがけ順の頂点列
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn postorder(&self) -> &[usize] {
        &self.postorder
    }

    /// 頂点 `v` を最初に訪れた時刻を返す。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `usize`: `v` の発見時刻であり、`1..=2 * vertex_count()` の範囲に収まる。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn discover_time(&self, v: usize) -> usize {
        self.discover_time[v]
    }

    /// 頂点 `v` の子をすべて訪れ終えた時刻を返す。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `usize`: `v` の完了時刻であり、`1..=2 * vertex_count()` の範囲に収まる。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn finish_time(&self, v: usize) -> usize {
        self.finish_time[v]
    }

    /// 頂点 `v` を含む木における、根からの深さ (辺数) を返す。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `usize`: `v` の深さ
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn depth(&self, v: usize) -> usize {
        self.depth[v]
    }

    /// オイラーツアー (頂点に最初に訪れたとき、および子から親へ戻るたびに記録した
    /// 頂点列) を返す。
    ///
    /// # Returns
    /// `&[usize]`: オイラーツアー
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn euler_tour(&self) -> &[usize] {
        &self.euler_tour
    }

    /// 頂点 `v` が `euler_tour()` の中で最初に現れる位置を返す。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `usize`: `euler_tour()` における `v` の最初の出現位置
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn first_occurrence(&self, v: usize) -> usize {
        self.first_occurrence[v]
    }
}

impl<T> graph::Graph<T> {
    /// グラフ全体を対象とした深さ優先探索を行う。
    ///
    /// 頂点 `0..vertex_count()` を昇順に走査し、未訪問の頂点が見つかるたびに
    /// そこを根とする新たな木として深さ優先探索を開始する。明示的なスタックを
    /// 用いた非再帰の実装であり、頂点数が大きいグラフに対しても、再帰呼び出し
    /// によるスタックオーバーフローを起こさない。
    ///
    /// # Returns
    /// `DfsForest`: 発見・完了時刻、深さ、オイラーツアーを含む探索結果
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
    /// let mut g = Graph::new(3);
    /// g.add_edge(0, 1, ());
    ///
    /// let forest = g.dfs_forest();
    /// assert_eq!(&[0, 1, 2], forest.preorder());
    /// assert_eq!(0, forest.depth(0));
    /// assert_eq!(1, forest.depth(1));
    /// assert_eq!(0, forest.depth(2));
    /// ```
    pub fn dfs_forest(&self) -> DfsForest {
        let n = self.vertex_count();
        let mut visited = vec![false; n];
        let mut preorder = Vec::new();
        let mut postorder = Vec::new();
        let mut discover_time = vec![0; n];
        let mut finish_time = vec![0; n];
        let mut depth = vec![0; n];
        let mut euler_tour = Vec::new();
        let mut first_occurrence = vec![0; n];
        let mut time = 0_usize;

        for root in 0..n {
            if visited[root] {
                continue;
            }
            visited[root] = true;
            time += 1;
            discover_time[root] = time;
            preorder.push(root);
            first_occurrence[root] = euler_tour.len();
            euler_tour.push(root);

            // (頂点, 隣接頂点のスナップショット, 次に見るインデックス) を1フレームとする
            // 非再帰の DFS を行う。子から親へ戻るたびに、親を再びオイラーツアーへ
            // 記録するため、単純な Enter/Leave の2状態だけでは表現できず、
            // スタック自体から「現在戻ってきた先の頂点」を参照できる形にしている。
            let neighbors = self.edges(root).map(|(v, _)| v).collect::<Vec<usize>>();
            let mut stack = vec![(root, neighbors, 0_usize)];

            while let Some(&mut (u, ref neighbors, ref mut idx)) = stack.last_mut() {
                if *idx < neighbors.len() {
                    let v = neighbors[*idx];
                    *idx += 1;

                    if !visited[v] {
                        visited[v] = true;
                        depth[v] = depth[u] + 1;
                        time += 1;
                        discover_time[v] = time;
                        preorder.push(v);
                        first_occurrence[v] = euler_tour.len();
                        euler_tour.push(v);
                        let next_neighbors =
                            self.edges(v).map(|(w, _)| w).collect::<Vec<usize>>();
                        stack.push((v, next_neighbors, 0));
                    }
                } else {
                    time += 1;
                    finish_time[u] = time;
                    postorder.push(u);
                    stack.pop();
                    if let Some(&(parent, _, _)) = stack.last() {
                        euler_tour.push(parent);
                    }
                }
            }
        }

        DfsForest {
            preorder,
            postorder,
            discover_time,
            finish_time,
            depth,
            euler_tour,
            first_occurrence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // dfs のテスト: 戻り値 (Dfs) が保持する行きがけ順・帰りがけ順を検証する。
    mod dfs {
        use super::*;

        /// Scenario: 単純パスでは、行きがけ順が始点からの到達順と一致する。
        /// - Given: `0 -> 1 -> 2` の単純パスからなる有向グラフがある。
        /// - When: 頂点 0 を始点に DFS を行う。
        /// - Then: 行きがけ順は `[0, 1, 2]` になる。
        #[test]
        fn returns_visiting_order_as_preorder_for_simple_path() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            // When
            let result = sut.dfs(0);
            // Then
            assert_eq!(&[0, 1, 2], result.preorder());
        }

        /// Scenario: 単純パスでは、帰りがけ順は行きがけ順の逆になる。
        /// - Given: `0 -> 1 -> 2` の単純パスからなる有向グラフがある。
        /// - When: 頂点 0 を始点に DFS を行う。
        /// - Then: 帰りがけ順は `[2, 1, 0]` になる。
        #[test]
        fn returns_reversed_order_as_postorder_for_simple_path() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            // When
            let result = sut.dfs(0);
            // Then
            assert_eq!(&[2, 1, 0], result.postorder());
        }

        /// Scenario: 始点から到達できない頂点は、行きがけ順・帰りがけ順のいずれにも
        /// 含まれない。
        /// - Given: 頂点 0 と、それとは独立した孤立頂点 1 からなるグラフがある。
        /// - When: 頂点 0 を始点に DFS を行う。
        /// - Then: 行きがけ順・帰りがけ順のいずれも `[0]` のみになる。
        #[test]
        fn excludes_unreachable_vertex() {
            // Given
            let sut = graph::Graph::<()>::new(2);
            // When
            let result = sut.dfs(0);
            // Then
            assert_eq!(&[0], result.preorder());
            assert_eq!(&[0], result.postorder());
        }

        /// Scenario: 閉路があっても、各頂点は1度だけ訪問される。
        /// - Given: `0 -> 1 -> 2 -> 0` の閉路からなる有向グラフがある。
        /// - When: 頂点 0 を始点に DFS を行う。
        /// - Then: 行きがけ順・帰りがけ順のいずれも、3頂点をちょうど1回ずつ含む。
        #[test]
        fn visits_each_vertex_exactly_once_even_with_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            sut.add_edge(1, 2, ());
            sut.add_edge(2, 0, ());
            // When
            let result = sut.dfs(0);
            // Then
            let mut preorder = result.preorder().to_vec();
            preorder.sort_unstable();
            assert_eq!(vec![0, 1, 2], preorder);
            let mut postorder = result.postorder().to_vec();
            postorder.sort_unstable();
            assert_eq!(vec![0, 1, 2], postorder);
        }
    }

    // dfs_forest のテスト: 戻り値 (DfsForest) が保持する発見・完了時刻、深さ、
    // オイラーツアーを検証する。
    mod dfs_forest {
        use super::*;

        /// Scenario: AOJ ALDS1_11_B (Depth First Search) の公式サンプルと同じ
        /// グラフを与えると、発見時刻・完了時刻が問題の期待出力と一致する。
        /// - Given: 6頂点からなる有向グラフ (自己ループを含む) がある。
        /// - When: グラフ全体を対象に深さ優先探索を行う。
        /// - Then: 各頂点の発見時刻・完了時刻が、問題の期待出力と一致する。
        #[test]
        fn matches_aoj_alds1_11_b_official_sample() {
            // Given
            let mut sut = graph::Graph::new(6);
            sut.add_edge(0, 1, ());
            sut.add_edge(0, 3, ());
            sut.add_edge(1, 4, ());
            sut.add_edge(2, 4, ());
            sut.add_edge(2, 5, ());
            sut.add_edge(4, 3, ());
            sut.add_edge(5, 5, ());
            // When
            let result = sut.dfs_forest();
            // Then
            let expected_discover = [1, 2, 9, 4, 3, 10];
            let expected_finish = [8, 7, 12, 5, 6, 11];
            for v in 0..6 {
                assert_eq!(expected_discover[v], result.discover_time(v));
                assert_eq!(expected_finish[v], result.finish_time(v));
            }
        }

        /// Scenario: 連結でないグラフでも、未訪問の頂点があれば昇順に次の木として
        /// 探索を開始し、すべての頂点を訪問する。
        /// - Given: `0 -> 1` の辺のみを持つグラフと、独立した孤立頂点 2 がある。
        /// - When: グラフ全体を対象に深さ優先探索を行う。
        /// - Then: 行きがけ順は `[0, 1, 2]` になり、頂点 2 は頂点 0 とは別の木の根
        ///   (深さ0) として訪問される。
        #[test]
        fn visits_every_component_in_ascending_order_of_unvisited_vertex() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_edge(0, 1, ());
            // When
            let result = sut.dfs_forest();
            // Then
            assert_eq!(&[0, 1, 2], result.preorder());
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.depth(1));
            assert_eq!(0, result.depth(2));
        }

        /// Scenario: オイラーツアーは、頂点への最初の訪問と、子から親へ戻るたびの
        /// 頂点を記録した列になり、`first_occurrence` はその中での最初の出現位置に
        /// なる。
        /// - Given: 頂点 0 を根とし、`0 -> 1 -> 3` および `0 -> 2` の辺を持つ木が
        ///   ある。
        /// - When: グラフ全体を対象に深さ優先探索を行う。
        /// - Then: オイラーツアーは `[0, 1, 3, 1, 0, 2, 0]` になり、`first_occurrence`
        ///   はそれぞれの頂点が最初に現れる位置と一致する。
        #[test]
        fn records_euler_tour_with_backtracking_to_parent() {
            // Given
            let mut sut = graph::Graph::new(4);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(0, 2, ());
            sut.add_undirected_edge(1, 3, ());
            // When
            let result = sut.dfs_forest();
            // Then
            assert_eq!(&[0, 1, 3, 1, 0, 2, 0], result.euler_tour());
            assert_eq!(0, result.first_occurrence(0));
            assert_eq!(1, result.first_occurrence(1));
            assert_eq!(5, result.first_occurrence(2));
            assert_eq!(2, result.first_occurrence(3));
        }
    }
}

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
}

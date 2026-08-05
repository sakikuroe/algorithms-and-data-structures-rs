//! 重心分解 (Centroid Decomposition) を提供するモジュールである。
//!
//! 木の重心 (取り除くと、残る各連結成分の頂点数がすべて元の頂点数の半分以下に
//! なる頂点) を繰り返し取り除いていくことで、木を「重心木」という新たな
//! 木構造へ分解する。ある頂点を取り除いて重心が確定するたびに、残った
//! 各連結成分に対して同じ手続きを再帰的に適用し、重心同士を親子関係で
//! つなぐ。この重心木の深さは O(log V) に抑えられるため、「2頂点間のパスに
//! 関する問い合わせを、両者の LCA にあたる重心ごとに分割統治する」アルゴリズム
//! (2頂点間距離の集計など) の土台として使われる。

use std::collections;

use super::graph::{self, NotATreeError};

/// 重心分解の結果を保持する。
pub struct CentroidDecomposition {
    /// `parent[v]` は、頂点 `v` を重心とする部分問題の元となった、1つ上の
    /// 階層の重心。最上位 (最初に取り除かれた重心) では `None`。
    parent: Vec<Option<usize>>,
    /// `level[v]` は、重心分解における頂点 `v` の階層。最初に取り除かれた
    /// 重心が0であり、以降取り除かれるたびに1ずつ増える。
    level: Vec<usize>,
}

impl CentroidDecomposition {
    /// 頂点 `v` を重心とする部分問題の、1つ上の階層の重心を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `Option<usize>`: 1つ上の階層の重心。`v` が最上位の重心である場合は
    /// `None`。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn parent(&self, v: usize) -> Option<usize> {
        self.parent[v]
    }

    /// 重心分解における頂点 `v` の階層を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: `v` の階層。最初に取り除かれた重心が0であり、以降取り除かれる
    /// たびに1ずつ増える。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn level(&self, v: usize) -> usize {
        self.level[v]
    }
}

/// スタック上で処理待ちの頂点を表す。`Enter` は初訪問時、`Leave` はその頂点の
/// 子をすべて訪れ終えた後の処理を表す。いずれも、木を辿る際に元の頂点へ
/// 戻らないための親頂点を併せて持つ。
enum Frame {
    Enter(usize, Option<usize>),
    Leave(usize, Option<usize>),
}

impl<T> graph::Graph<T> {
    /// グラフが木であることを検証したうえで、木を重心分解する。
    ///
    /// # Returns
    /// `Result<CentroidDecomposition, NotATreeError>`: 木であれば、各頂点が
    /// 重心分解においてどの階層に属し、どの重心から取り除かれたかを保持
    /// する。木でなければ、その理由を表すエラー。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。木であることの検証は O(V) で収まる。重心を
    ///     1つ取り除くたびに、残る連結成分の頂点数は半分以下になるため、
    ///     ある頂点が部分問題に含まれる回数は O(log V) 回に抑えられる。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 頂点0を中心とする星グラフ。中心を取り除けば、残りは1頂点ずつの
    /// // 連結成分に分かれるため、中心が唯一の重心になる。
    /// let mut g = Graph::new(5);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    /// g.add_undirected_edge(0, 3, ());
    /// g.add_undirected_edge(0, 4, ());
    ///
    /// let cd = g.try_centroid_decomposition().unwrap();
    /// assert_eq!(0, cd.level(0));
    /// assert_eq!(None, cd.parent(0));
    /// assert_eq!(Some(0), cd.parent(1));
    /// ```
    pub fn try_centroid_decomposition(&self) -> Result<CentroidDecomposition, NotATreeError> {
        graph::validate_tree(self, 0)?;

        let n = self.vertex_count();
        let mut removed = vec![false; n];
        // 部分木サイズの一時置き場。呼び出しのたびに、書き込んだ分だけ0へ
        // 戻すことで、n に比例した初期化コストを避ける。
        let mut subtree_size = vec![0_usize; n];
        let mut parent = vec![None; n];
        let mut level = vec![0; n];

        // 未処理の部分問題を、(その中の適当な1頂点, 親となる重心, 階層) の
        // 組として管理する。
        let mut queue = collections::VecDeque::new();
        queue.push_back((0_usize, None, 0_usize));
        while let Some((start, centroid_parent, lvl)) = queue.pop_front() {
            let c = self.find_centroid(start, &removed, &mut subtree_size);
            removed[c] = true;
            parent[c] = centroid_parent;
            level[c] = lvl;

            for (v, _) in self.edges(c) {
                if !removed[v] {
                    queue.push_back((v, Some(c), lvl + 1));
                }
            }
        }

        Ok(CentroidDecomposition { parent, level })
    }

    /// `removed` で取り除かれた頂点を無視した残りの連結成分のうち、`start` を
    /// 含む成分の重心を求める。
    ///
    /// # Args
    /// - `start`: 対象の連結成分に含まれる、任意の1頂点。
    /// - `removed`: すでに重心として取り除かれた頂点の集合。
    /// - `subtree_size`: 部分木サイズの一時置き場。呼び出し前後で全要素が0の
    ///   状態を保つ (呼び出し中に書き込んだ分は、戻り値を返す前に0へ戻す)。
    ///
    /// # Returns
    /// `usize`: 求めた重心。
    ///
    /// # Complexity
    /// - 時間計算量: O(連結成分のサイズ)
    fn find_centroid(&self, start: usize, removed: &[bool], subtree_size: &mut [usize]) -> usize {
        // 部分木サイズを、start を根とみなした帰りがけ順の反復 DFS で求める。
        let mut visited_order = Vec::new();
        let mut stack = vec![Frame::Enter(start, None)];
        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Enter(u, p) => {
                    visited_order.push(u);
                    stack.push(Frame::Leave(u, p));
                    for (v, _) in self.edges(u) {
                        if !removed[v] && Some(v) != p {
                            stack.push(Frame::Enter(v, Some(u)));
                        }
                    }
                }
                Frame::Leave(u, p) => {
                    subtree_size[u] += 1;
                    if let Some(p) = p {
                        subtree_size[p] += subtree_size[u];
                    }
                }
            }
        }

        let total = subtree_size[start];

        // start から、部分木サイズが全体の半分を超える子がある方向へ進み続ける。
        // そのような子は、木の構造上高々1つしか存在しない。これ以上進めなく
        // なった頂点が重心である。
        let mut current = start;
        let mut current_parent = None;
        while let Some((next, _)) = self
            .edges(current)
            .find(|&(v, _)| !removed[v] && Some(v) != current_parent && 2 * subtree_size[v] > total)
        {
            current_parent = Some(current);
            current = next;
        }

        // 次回の呼び出しに備え、このラウンドで書き込んだ分だけ0へ戻す。
        for v in visited_order {
            subtree_size[v] = 0;
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // try_centroid_decomposition のテスト: 戻り値 (CentroidDecomposition) が
    // 保持する重心・階層関係を検証する。
    mod try_centroid_decomposition {
        use super::*;

        /// Scenario: 星グラフでは、中心のみが唯一の重心になる。
        /// - Given: 頂点0を中心とし、頂点1,2,3,4を葉とする星グラフがある。
        /// - When: 重心分解を行う。
        /// - Then: 頂点0が最上位の重心 (階層0、親なし) になり、残る葉は
        ///   すべて頂点0を親とする階層1の重心になる (中心を取り除いた時点で
        ///   葉はそれぞれ孤立するため)。
        #[test]
        fn treats_center_as_sole_centroid_for_star_graph() {
            // Given
            let mut sut = graph::Graph::new(5);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(0, 2, ());
            sut.add_undirected_edge(0, 3, ());
            sut.add_undirected_edge(0, 4, ());
            // When
            let result = sut.try_centroid_decomposition().unwrap();
            // Then
            assert_eq!(0, result.level(0));
            assert_eq!(None, result.parent(0));
            for leaf in 1..5 {
                assert_eq!(1, result.level(leaf));
                assert_eq!(Some(0), result.parent(leaf));
            }
        }

        /// Scenario: 頂点数が奇数の単純パスでは、ちょうど中央の頂点が
        /// 唯一の重心になる。
        /// - Given: 0-1-2-3-4 の単純パス (5頂点) がある。
        /// - When: 重心分解を行う。
        /// - Then: 中央の頂点2が最上位の重心になる (どちらの端から見ても
        ///   両側の頂点数が2以下になるのは頂点2のみであるため)。
        #[test]
        fn picks_middle_vertex_as_centroid_for_odd_length_path() {
            // Given
            let mut sut = graph::Graph::new(5);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 3, ());
            sut.add_undirected_edge(3, 4, ());
            // When
            let result = sut.try_centroid_decomposition().unwrap();
            // Then
            assert_eq!(0, result.level(2));
            assert_eq!(None, result.parent(2));
        }

        /// Scenario: すべての頂点が、いずれかの階層でちょうど1回ずつ
        /// 重心として取り除かれる。
        /// - Given: 0-1-2-3-4-5-6 の単純パス (7頂点) がある。
        /// - When: 重心分解を行う。
        /// - Then: 全頂点の階層が確定しており (デフォルト値である0のまま
        ///   放置された頂点がない)、階層は高々 log2(7) 程度に収まる。
        #[test]
        fn assigns_level_to_every_vertex_within_logarithmic_depth() {
            // Given
            let mut sut = graph::Graph::new(7);
            for i in 0..6 {
                sut.add_undirected_edge(i, i + 1, ());
            }
            // When
            let result = sut.try_centroid_decomposition().unwrap();
            // Then
            let max_level = (0..7).map(|v| result.level(v)).max().unwrap();
            assert!(
                max_level <= 3,
                "max level {max_level} exceeds log2(7) bound"
            );
        }

        /// Scenario: 頂点が1つだけの木では、その頂点自身が唯一の重心になる
        /// (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 重心分解を行う。
        /// - Then: 頂点0が階層0・親なしの重心になる。
        #[test]
        fn handles_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let result = sut.try_centroid_decomposition().unwrap();
            // Then
            assert_eq!(0, result.level(0));
            assert_eq!(None, result.parent(0));
        }

        /// Scenario: 2頂点の木では、いずれか一方が重心になる (境界値)。
        /// - Given: 0-1 の1辺のみからなる木がある。
        /// - When: 重心分解を行う。
        /// - Then: 一方が階層0の重心になり、もう一方はそれを親とする
        ///   階層1の重心になる。
        #[test]
        fn handles_two_vertex_tree() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_undirected_edge(0, 1, ());
            // When
            let result = sut.try_centroid_decomposition().unwrap();
            // Then
            let levels = [result.level(0), result.level(1)];
            assert!(levels.contains(&0));
            assert!(levels.contains(&1));
            let root = if result.level(0) == 0 { 0 } else { 1 };
            let leaf = 1 - root;
            assert_eq!(None, result.parent(root));
            assert_eq!(Some(root), result.parent(leaf));
        }

        /// Scenario: 非連結なグラフを渡すと、`Disconnected` が返る。
        /// - Given: 孤立した頂点を含む、非連結なグラフがある。
        /// - When: 重心分解を行う。
        /// - Then: `Err(NotATreeError::Disconnected)` が返る。
        #[test]
        fn returns_disconnected_error_for_disconnected_graph() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            // 頂点2は孤立している。
            // When
            let result = sut.try_centroid_decomposition();
            // Then
            assert!(matches!(result, Err(NotATreeError::Disconnected)));
        }

        /// Scenario: 閉路を含むグラフを渡すと、`HasCycle` が返る。
        /// - Given: 0-1-2-0 の閉路を含むグラフがある。
        /// - When: 重心分解を行う。
        /// - Then: `Err(NotATreeError::HasCycle)` が返る。
        #[test]
        fn returns_has_cycle_error_for_graph_with_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            // When
            let result = sut.try_centroid_decomposition();
            // Then
            assert!(matches!(result, Err(NotATreeError::HasCycle)));
        }
    }
}

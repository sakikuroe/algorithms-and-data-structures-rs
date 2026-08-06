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
    /// - `v`: 対象の頂点
    ///
    /// # Returns
    /// `Option<usize>`: 1つ上の階層の重心であり、`v` が最上位の重心である
    /// 場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn parent(&self, v: usize) -> Option<usize> {
        self.parent[v]
    }

    /// 重心分解における頂点 `v` の階層を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点
    ///
    /// # Returns
    /// `usize`: `v` の階層であり、最初に取り除かれた重心が0、以降取り除かれる
    /// たびに1ずつ増える。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn level(&self, v: usize) -> usize {
        self.level[v]
    }

    /// 重心ごとに、その時点でまだ取り除かれていない部分問題を、辺数を
    /// 距離として巡り、コールバック `f` へ渡す。
    ///
    /// 重心分解を使うアルゴリズムの多くは、各重心について「すでに重心として
    /// 取り除かれた頂点 (祖先) を避けて部分問題内を探索し、重心を含む部分問題
    /// 全体の集計から、各枝だけの集計を差し引く」という手順を必要とする。この
    /// メソッドは、その祖先の判定・探索・集計対象を効率よく使い回すための
    /// リセット処理をすべて内部で肩代わりし、利用者には「重心・部分問題全体の
    /// 距離の列・枝ごとの距離の列」だけを渡す。
    ///
    /// # Args
    /// - `g`: 重心分解の元になったグラフ
    /// - `f`: 各重心について1回ずつ呼び出されるコールバックであり、引数は
    ///   `(重心, 部分問題全体の頂点の距離の列 (重心自身の距離0を含む),
    ///   枝ごとの頂点の距離の列)` である。距離の順序は規定しない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。重心を1つ取り除くたびに、残る連結成分の頂点数は
    ///     半分以下になるため、ある頂点が部分問題に含まれる回数は
    ///     O(log V) 回に抑えられる。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1-2-3-4 の単純パス。全頂点対の距離の総和を求める。
    /// let mut g = Graph::new(5);
    /// for i in 0..4 {
    ///     g.add_undirected_edge(i, i + 1, ());
    /// }
    /// let cd = g.try_centroid_decomposition().unwrap();
    ///
    /// let mut total = 0_i64;
    /// cd.for_each_component(&g, |_centroid, whole, branches| {
    ///     let whole_sum = whole.iter().map(|&d| d as i64).sum::<i64>();
    ///     total += (whole.len() as i64 - 1) * whole_sum;
    ///     for branch in branches {
    ///         let branch_sum = branch.iter().map(|&d| d as i64).sum::<i64>();
    ///         total -= (branch.len() as i64 - 1) * branch_sum;
    ///     }
    /// });
    /// assert_eq!(20, total); // 0-1,0-2,...,3-4 の距離の総和
    /// ```
    pub fn for_each_component<T>(
        &self,
        g: &graph::Graph<T>,
        mut f: impl FnMut(usize, &[usize], &[Vec<usize>]),
    ) {
        let n = g.vertex_count();
        let mut blocked = vec![false; n];
        let mut dist: Vec<usize> = vec![usize::MAX; n];
        let mut branch_of: Vec<usize> = vec![usize::MAX; n];

        for c in 0..n {
            let mut ancestors = Vec::new();
            let mut cur = self.parent(c);
            while let Some(p) = cur {
                blocked[p] = true;
                ancestors.push(p);
                cur = self.parent(p);
            }

            let mut component = vec![c];
            dist[c] = 0;
            let mut queue = collections::VecDeque::new();
            queue.push_back(c);
            while let Some(u) = queue.pop_front() {
                for (v, _) in g.edges(u) {
                    if !blocked[v] && dist[v] == usize::MAX {
                        dist[v] = dist[u] + 1;
                        branch_of[v] = if u == c { v } else { branch_of[u] };
                        component.push(v);
                        queue.push_back(v);
                    }
                }
            }

            let whole = component.iter().map(|&v| dist[v]).collect::<Vec<usize>>();

            let mut branch_index: collections::HashMap<usize, usize> = collections::HashMap::new();
            let mut branches: Vec<Vec<usize>> = Vec::new();
            for &v in &component {
                if v != c {
                    let idx = *branch_index
                        .entry(branch_of[v])
                        .or_insert_with(|| {
                            branches.push(Vec::new());
                            branches.len() - 1
                        });
                    branches[idx].push(dist[v]);
                }
            }

            f(c, &whole, &branches);

            for v in component {
                dist[v] = usize::MAX;
                branch_of[v] = usize::MAX;
            }
            for p in ancestors {
                blocked[p] = false;
            }
        }
    }

    /// 辺のペイロードから重みを射影する関数 `weight_of` を与えて、重心ごとに
    /// その時点でまだ取り除かれていない部分問題を巡り、コールバック `f` へ
    /// 渡す。
    ///
    /// 挙動は [`for_each_component`](Self::for_each_component) と同様であり、
    /// 辺数の代わりに `weight_of` が返す重みの総和を距離として扱う。
    ///
    /// # Args
    /// - `g`: 重心分解の元になったグラフ
    /// - `weight_of`: 辺のペイロードから、加算・比較可能な重みを取り出す関数
    /// - `f`: 各重心について1回ずつ呼び出されるコールバックであり、引数は
    ///   `(重心, 部分問題全体の頂点の距離の列 (重心自身の距離0を含む),
    ///   枝ごとの頂点の距離の列)` である。距離の順序は規定しない。
    ///
    /// # Constraints
    /// - `weight_of` はすべての辺に対して非負の値を返さなければならない。
    /// - `W::default()` が加法の単位元 (0 に相当する値) でなければならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(V log V)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0-1 (重み3), 1-2 (重み1), 1-3 (重み2), 3-4 (重み1) の木。
    /// // 距離が4以下となる頂点対の個数を求める。
    /// let mut g = Graph::new(5);
    /// g.add_undirected_edge(0, 1, 3_u32);
    /// g.add_undirected_edge(1, 2, 1_u32);
    /// g.add_undirected_edge(1, 3, 2_u32);
    /// g.add_undirected_edge(3, 4, 1_u32);
    /// let cd = g.try_centroid_decomposition().unwrap();
    ///
    /// let k = 4_u32;
    /// let mut count = 0_i64;
    /// let count_pairs = |depths: &[u32]| -> i64 {
    ///     let mut sorted = depths.to_vec();
    ///     sorted.sort_unstable();
    ///     let (mut lo, mut hi) = (0, sorted.len().saturating_sub(1));
    ///     let mut c = 0_i64;
    ///     while lo < hi {
    ///         if sorted[lo] + sorted[hi] <= k {
    ///             c += (hi - lo) as i64;
    ///             lo += 1;
    ///         } else {
    ///             hi -= 1;
    ///         }
    ///     }
    ///     c
    /// };
    /// cd.for_each_component_by(&g, |&w| w, |_centroid, whole, branches| {
    ///     count += count_pairs(whole);
    ///     for branch in branches {
    ///         count -= count_pairs(branch);
    ///     }
    /// });
    /// assert_eq!(8, count);
    /// ```
    pub fn for_each_component_by<T, W, WF>(
        &self,
        g: &graph::Graph<T>,
        weight_of: WF,
        mut f: impl FnMut(usize, &[W], &[Vec<W>]),
    ) where
        W: Copy + Ord + std::ops::Add<Output = W> + Default,
        WF: Fn(&T) -> W,
    {
        let n = g.vertex_count();
        let mut blocked = vec![false; n];
        let mut dist: Vec<Option<W>> = vec![None; n];
        let mut branch_of: Vec<usize> = vec![usize::MAX; n];

        for c in 0..n {
            let mut ancestors = Vec::new();
            let mut cur = self.parent(c);
            while let Some(p) = cur {
                blocked[p] = true;
                ancestors.push(p);
                cur = self.parent(p);
            }

            let mut component = vec![c];
            dist[c] = Some(W::default());
            let mut queue = collections::VecDeque::new();
            queue.push_back(c);
            while let Some(u) = queue.pop_front() {
                for (v, payload) in g.edges(u) {
                    if !blocked[v] && dist[v].is_none() {
                        dist[v] = Some(dist[u].expect("u was already visited") + weight_of(payload));
                        branch_of[v] = if u == c { v } else { branch_of[u] };
                        component.push(v);
                        queue.push_back(v);
                    }
                }
            }

            let whole = component
                .iter()
                .map(|&v| dist[v].expect("every vertex in component has a distance"))
                .collect::<Vec<W>>();

            let mut branch_index: collections::HashMap<usize, usize> = collections::HashMap::new();
            let mut branches: Vec<Vec<W>> = Vec::new();
            for &v in &component {
                if v != c {
                    let idx = *branch_index
                        .entry(branch_of[v])
                        .or_insert_with(|| {
                            branches.push(Vec::new());
                            branches.len() - 1
                        });
                    branches[idx].push(dist[v].expect("every vertex in component has a distance"));
                }
            }

            f(c, &whole, &branches);

            for v in component {
                dist[v] = None;
                branch_of[v] = usize::MAX;
            }
            for p in ancestors {
                blocked[p] = false;
            }
        }
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
    /// したものであり、木でなければその理由を表すエラーである。
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
    /// - `start`: 対象の連結成分に含まれる、任意の1頂点
    /// - `removed`: すでに重心として取り除かれた頂点の集合
    /// - `subtree_size`: 部分木サイズの一時置き場であり、呼び出し前後で全要素
    ///   が0の状態を保つ (呼び出し中に書き込んだ分は、戻り値を返す前に0へ
    ///   戻す)。
    ///
    /// # Returns
    /// `usize`: 求めた重心
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

    // for_each_component のテスト: コールバックへ渡される内容 (依存先との
    // 相互作用) を検証する。
    mod for_each_component {
        use super::*;

        /// Scenario: すべての頂点が、いずれかの重心の部分問題全体に
        /// ちょうど1回含まれる (自分自身を重心とする回)。
        /// - Given: 0-1-2-3-4 の単純パス (5頂点) がある。
        /// - When: 重心分解を行い、`for_each_component` を呼ぶ。
        /// - Then: 各頂点 v について、v を重心とする回の部分問題全体に
        ///   v 自身 (距離0) が含まれる。
        #[test]
        fn includes_centroid_itself_at_distance_zero_in_its_own_component() {
            // Given
            let mut g = graph::Graph::new(5);
            for i in 0..4 {
                g.add_undirected_edge(i, i + 1, ());
            }
            let sut = g.try_centroid_decomposition().unwrap();
            // When
            let mut seen_self_at_zero = [false; 5];
            sut.for_each_component(&g, |centroid, whole, _branches| {
                seen_self_at_zero[centroid] = whole.contains(&0);
            });
            // Then
            assert!(seen_self_at_zero.iter().all(|&seen| seen));
        }

        /// Scenario: 頂点対の距離の総和を、枝ごとの二重計上を差し引く
        /// 標準的な手順で正しく求められる。
        /// - Given: 0-1-2-3-4 の単純パス (5頂点、辺の重みはすべて1) が
        ///   ある。
        /// - When: 重心分解の各重心について、部分問題全体の距離の和から
        ///   各枝だけの距離の和を差し引いて集計する。
        /// - Then: 全頂点対の距離の総和 (20) になる。
        #[test]
        fn computes_sum_of_all_pairwise_distances_via_branch_subtraction() {
            // Given
            let mut g = graph::Graph::new(5);
            for i in 0..4 {
                g.add_undirected_edge(i, i + 1, ());
            }
            let sut = g.try_centroid_decomposition().unwrap();
            // When
            let mut total = 0_i64;
            sut.for_each_component(&g, |_centroid, whole, branches| {
                let whole_sum = whole.iter().map(|&d| d as i64).sum::<i64>();
                total += (whole.len() as i64 - 1) * whole_sum;
                for branch in branches {
                    let branch_sum = branch.iter().map(|&d| d as i64).sum::<i64>();
                    total -= (branch.len() as i64 - 1) * branch_sum;
                }
            });
            // Then
            assert_eq!(20, total);
        }

        /// Scenario: 頂点が1つだけの木では、唯一の重心の部分問題は
        /// その頂点自身のみからなり、枝は存在しない (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: `for_each_component` を呼ぶ。
        /// - Then: 部分問題全体は距離0の1頂点のみからなり、枝は空になる。
        #[test]
        fn handles_single_vertex_tree_with_no_branches() {
            // Given
            let g = graph::Graph::<()>::new(1);
            let sut = g.try_centroid_decomposition().unwrap();
            // When
            let mut whole_snapshot = Vec::new();
            let mut branch_count = 0;
            sut.for_each_component(&g, |_centroid, whole, branches| {
                whole_snapshot = whole.to_vec();
                branch_count = branches.len();
            });
            // Then
            assert_eq!(vec![0], whole_snapshot);
            assert_eq!(0, branch_count);
        }
    }

    // for_each_component_by のテスト: コールバックへ渡される内容を検証する。
    mod for_each_component_by {
        use super::*;

        /// Scenario: 辺の重みを反映した距離が渡され、距離がK以下となる
        /// 頂点対の個数を、枝ごとの二重計上を差し引く標準的な手順で
        /// 正しく求められる。
        /// - Given: 0-1 (重み3), 1-2 (重み1), 1-3 (重み2), 3-4 (重み1) の
        ///   木がある。
        /// - When: 重心分解の各重心について、部分問題全体で距離の和が
        ///   K=4以下となるペア数から、各枝だけでのペア数を差し引いて
        ///   集計する。
        /// - Then: 距離が4以下となる頂点対の個数 (8) になる。
        #[test]
        fn computes_pairs_within_distance_threshold_via_branch_subtraction() {
            // Given
            let mut g = graph::Graph::new(5);
            g.add_undirected_edge(0, 1, 3_u32);
            g.add_undirected_edge(1, 2, 1_u32);
            g.add_undirected_edge(1, 3, 2_u32);
            g.add_undirected_edge(3, 4, 1_u32);
            let sut = g.try_centroid_decomposition().unwrap();
            let k = 4_u32;
            let count_pairs = |depths: &[u32]| -> i64 {
                let mut sorted = depths.to_vec();
                sorted.sort_unstable();
                let (mut lo, mut hi) = (0, sorted.len().saturating_sub(1));
                let mut c = 0_i64;
                while lo < hi {
                    if sorted[lo] + sorted[hi] <= k {
                        c += (hi - lo) as i64;
                        lo += 1;
                    } else {
                        hi -= 1;
                    }
                }
                c
            };
            // When
            let mut count = 0_i64;
            sut.for_each_component_by(&g, |&w| w, |_centroid, whole, branches| {
                count += count_pairs(whole);
                for branch in branches {
                    count -= count_pairs(branch);
                }
            });
            // Then
            assert_eq!(8, count);
        }
    }
}

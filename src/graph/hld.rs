//! 重軽分解 (Heavy-Light Decomposition, HLD) を提供するモジュールである。
//!
//! 木を「重い辺」(各頂点から、部分木サイズが最大の子への辺) からなる連結な
//! パスの集まりに分解し、各頂点に、パスに沿って連続した番号 (`id`) を割り
//! 振る。この番号付けにより、任意の2頂点間のパスを O(log V) 本の連続した
//! 区間に分解できるため、区間に対応するデータ構造 (セグメント木など) と
//! 組み合わせることで、木上のパスクエリを高速に処理できる。
//!
//! chain の先頭 (`head`) を辿ることで、LCA・k個上の祖先・2頂点間のパス上の
//! ジャンプも O(log V) で求まるため、木に関する各種クエリをこの1つの構造に
//! 集約している。ダブリングによる別実装 (二分累乗法) と比べて、前処理が
//! O(V log V) ではなく O(V) で済み、キャッシュ効率の良い連続したメモリ配置
//! (`Vec<Option<usize>>` の段ごとの二重配列ではなく、単純な `Vec<usize>` の
//! 組) になるという利点がある。
//!
//! 区間分解 ([`vertex_path_ranges`](Hld::vertex_path_ranges) /
//! [`edge_path_ranges`](Hld::edge_path_ranges)) は、区間同士および区間内部の
//! 結合順序を区別しない。そのため、可換なモノイド (総和・最小値・最大値・xor
//! など) によるパスクエリを想定しており、非可換な演算 (行列積など、経路の
//! 向きによって結果が変わる演算) には対応しない。

use super::graph::{self, NotATreeError};

/// スタック上で処理待ちの頂点を表す。`Enter` は初訪問時、`Leave` はその頂点の
/// 子をすべて訪れ終えた後の処理を表す。いずれも、木を辿る際に元の頂点へ
/// 戻らないための親頂点を併せて持つ。
enum Frame {
    Enter(usize, Option<usize>),
    Leave(usize, Option<usize>),
}

/// HLD の結果を保持する。
pub struct Hld {
    /// 構築時に指定した根。
    root: usize,
    /// `parent[v]` は、頂点 `v` の親。根では `None`。
    parent: Vec<Option<usize>>,
    /// `depth[v]` は、根から頂点 `v` までの距離 (辺数)。
    depth: Vec<usize>,
    /// `head[v]` は、頂点 `v` が属する連結パスのうち、最も浅い頂点。
    head: Vec<usize>,
    /// `id[v]` は、パスに沿って割り振られた頂点 `v` の番号 (`0..頂点数`)。
    /// 同じパスに属する頂点の間では、深さの昇順と一致する。
    id: Vec<usize>,
    /// `order[i]` は、番号が `i` である頂点。`id` の逆写像であり、
    /// [`kth_ancestor`](Self::kth_ancestor) で使う。
    order: Vec<usize>,
    /// `subtree_size[v]` は、頂点 `v` を根とする部分木のサイズ。
    subtree_size: Vec<usize>,
}

impl Hld {
    /// 構築時に指定した根を返す。
    ///
    /// # Returns
    /// `usize`: 根とした頂点。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn root(&self) -> usize {
        self.root
    }

    /// 根から頂点 `v` までの深さ (辺数) を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: 根から `v` までの深さ。根自身の深さは0。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn depth(&self, v: usize) -> usize {
        self.depth[v]
    }

    /// 頂点 `u` と `v` の間の距離 (最短路の辺数) を返す。
    ///
    /// 2頂点間の距離はどちらの頂点を根と見なしても変わらないため、`root`
    /// を引数に取らない。
    ///
    /// # Args
    /// - `u`: 1つ目の頂点。
    /// - `v`: 2つ目の頂点。
    ///
    /// # Returns
    /// `usize`: `u` と `v` の間の距離。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn distance(&self, u: usize, v: usize) -> usize {
        let w = self.lca_from_root(u, v);
        self.depth[u] + self.depth[v] - 2 * self.depth[w]
    }

    /// 頂点 `v` を根とする部分木のサイズを、`root` を根としたときの値で返す。
    ///
    /// # Args
    /// - `root`: 部分木サイズを数える際に根と見なす頂点。構築時に指定した
    ///   根と同じ頂点を渡せば、通常の (付け替えを行わない) 部分木サイズが
    ///   得られる。
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: `root` を根と見なしたときの、`v` を根とする部分木に含まれる
    /// 頂点数。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// // 0 の子が 1, 2、1 の子が 3 である木。
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    /// g.add_undirected_edge(1, 3, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// assert_eq!(2, hld.subtree_size(0, 1)); // 通常の部分木サイズ ({1, 3})
    /// // 頂点3を根と見なすと、頂点1の部分木は「3側を除いた残り全部」になる。
    /// assert_eq!(3, hld.subtree_size(3, 1));
    /// ```
    pub fn subtree_size(&self, root: usize, v: usize) -> usize {
        if root == v {
            return self.parent.len();
        }

        let (lo, hi) = self.subtree_range(v);
        let root_id = self.id[root];
        if root_id < lo || hi <= root_id {
            // root は (構築時の根における) v の部分木の外にある。
            // 付け替えても v の部分木は変化しない。
            self.subtree_size[v]
        } else {
            // root は v の部分木の中にある。v から root へ向かう最初の子
            // (root 側の枝) を除いた残り全体が、付け替え後の v の部分木になる。
            let steps = self.depth[root] - self.depth[v] - 1;
            let child_toward_root = self
                .kth_ancestor(root, steps)
                .expect("root must be a strict descendant of v here");
            self.parent.len() - self.subtree_size[child_toward_root]
        }
    }

    /// 頂点 `v` に割り振られた番号を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: `0..頂点数` の範囲で割り振られた `v` の番号。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn vertex_id(&self, v: usize) -> usize {
        self.id[v]
    }

    /// 頂点 `v` を根とする部分木 (構築時の根における部分木) に対応する、
    /// 番号の半開区間を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `(usize, usize)`: `v` を根とする部分木に属する頂点の番号が、
    /// ちょうど収まる半開区間 `[start, end)`。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn subtree_range(&self, v: usize) -> (usize, usize) {
        (self.id[v], self.id[v] + self.subtree_size[v])
    }

    /// 頂点 `u` と `v` の最近共通祖先 (LCA) を、`root` を根としたときの値で
    /// 返す。
    ///
    /// # Args
    /// - `root`: LCA を求める際に根と見なす頂点。構築時に指定した根と同じ
    ///   頂点を渡せば、通常の (付け替えを行わない) LCA が得られる。
    /// - `u`: 1つ目の頂点。
    /// - `v`: 2つ目の頂点。
    ///
    /// # Returns
    /// `usize`: `root` を根と見なしたときの、`u` と `v` の最近共通祖先。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(4);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    /// g.add_undirected_edge(1, 3, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// assert_eq!(1, hld.lca(0, 1, 3));
    /// // 頂点3を根と見なすと、根から頂点2への経路は 3->1->0->2 となるため、
    /// // 頂点0が頂点2の祖先になり、頂点0と頂点2のLCAは頂点0になる。
    /// assert_eq!(0, hld.lca(3, 0, 2));
    /// ```
    pub fn lca(&self, root: usize, u: usize, v: usize) -> usize {
        // 構築時の根における3通りの LCA のうち、深さが最大のものが答えになる
        // (根の付け替えテクニック)。深さが等しい2つは必ず一致し、残る1つ
        // (または3つとも一致する場合はその値) が正しい答えになる。
        let candidates = [
            self.lca_from_root(u, v),
            self.lca_from_root(root, u),
            self.lca_from_root(root, v),
        ];
        *candidates
            .iter()
            .max_by_key(|&&w| self.depth[w])
            .expect("candidates is never empty")
    }

    /// 構築時の根における、頂点 `u` と `v` の最近共通祖先を返す。
    ///
    /// # Args
    /// - `u`/`v`: 対象の頂点。
    ///
    /// # Returns
    /// `usize`: 構築時の根における `u` と `v` の最近共通祖先。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    fn lca_from_root(&self, mut u: usize, mut v: usize) -> usize {
        // 異なるパスに属する間は、パスの根元がより深い側を、そのパスの
        // 親へ引き上げていく。同じパスに入れば、そのパス上を1歩も辿らずに
        // 残りの祖先関係が確定する。
        while self.head[u] != self.head[v] {
            if self.depth[self.head[u]] > self.depth[self.head[v]] {
                u = self.parent[self.head[u]].unwrap();
            } else {
                v = self.parent[self.head[v]].unwrap();
            }
        }
        if self.depth[u] <= self.depth[v] { u } else { v }
    }

    /// 頂点 `v` から、構築時の根の方向へ `k` 個進んだ祖先を返す。
    ///
    /// # Args
    /// - `v`: 起点となる頂点。
    /// - `k`: 根の方向へ進む歩数。
    ///
    /// # Returns
    /// `Option<usize>`: `k` 個上の祖先。根を超えてしまう場合は `None`。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// assert_eq!(Some(0), hld.kth_ancestor(2, 2));
    /// assert_eq!(None, hld.kth_ancestor(2, 3));
    /// ```
    pub fn kth_ancestor(&self, v: usize, mut k: usize) -> Option<usize> {
        let mut v = v;
        loop {
            // v からその頂点が属するパスの先頭までの歩数。
            let steps_to_head = self.depth[v] - self.depth[self.head[v]];
            if k <= steps_to_head {
                // 同じパスの中で完結するため、番号を k だけ戻すだけでよい。
                return Some(self.order[self.id[v] - k]);
            }
            // パスの先頭を超えて、さらにもう1歩 (親へ) 進む必要がある。
            k -= steps_to_head + 1;
            v = self.parent[self.head[v]]?;
        }
    }

    /// 頂点 `u` から `v` へ向かうパス上で、`u` から `k` 歩進んだ頂点を返す。
    ///
    /// # Args
    /// - `u`: パスの起点。
    /// - `v`: パスの終点。
    /// - `k`: `u` から `v` へ向かう方向に進む歩数。
    ///
    /// # Returns
    /// `Option<usize>`: `k` 歩進んだ頂点。`k` が `u` から `v` までの距離を
    /// 超える場合は `None`。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(0, 2, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// assert_eq!(Some(0), hld.jump(1, 2, 1));
    /// assert_eq!(Some(2), hld.jump(1, 2, 2));
    /// ```
    pub fn jump(&self, u: usize, v: usize, k: usize) -> Option<usize> {
        let w = self.lca_from_root(u, v);
        let dist_to_lca = self.depth[u] - self.depth[w];
        let dist_from_lca = self.depth[v] - self.depth[w];

        if k <= dist_to_lca {
            self.kth_ancestor(u, k)
        } else if k <= dist_to_lca + dist_from_lca {
            self.kth_ancestor(v, dist_to_lca + dist_from_lca - k)
        } else {
            None
        }
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての頂点の番号を、
    /// 半開区間の列として返す。
    ///
    /// 頂点に値を持たせたパスクエリ (可換なモノイドに限る) で使う。
    ///
    /// # Args
    /// - `u`: パスの一方の端点。
    /// - `v`: パスのもう一方の端点。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パス上の頂点の番号をちょうど覆う、半開区間
    /// `[start, end)` の列。区間の個数は O(log V) 本になる。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// let total_length: usize = hld
    ///     .vertex_path_ranges(0, 2)
    ///     .iter()
    ///     .map(|&(l, r)| r - l)
    ///     .sum();
    /// assert_eq!(3, total_length);
    /// ```
    pub fn vertex_path_ranges(&self, u: usize, v: usize) -> Vec<(usize, usize)> {
        self.path_ranges(u, v, false)
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての辺に対応する番号を、
    /// 半開区間の列として返す。
    ///
    /// 辺に値を持たせたパスクエリ (可換なモノイドに限る) で使う。各辺は、
    /// その子側の頂点の番号で表す。
    ///
    /// # Args
    /// - `u`: パスの一方の端点。
    /// - `v`: パスのもう一方の端点。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パス上の辺に対応する番号をちょうど覆う、
    /// 半開区間 `[start, end)` の列。区間の個数は O(log V) 本になる。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn edge_path_ranges(&self, u: usize, v: usize) -> Vec<(usize, usize)> {
        self.path_ranges(u, v, true)
    }

    /// [`vertex_path_ranges`](Self::vertex_path_ranges) と
    /// [`edge_path_ranges`](Self::edge_path_ranges) の共通部分を担う。
    ///
    /// # Args
    /// - `u`/`v`: パスの両端点。
    /// - `exclude_lca`: `true` の場合、最後にまとめる同一パス上の区間から
    ///   LCA 自身の番号を除く (辺クエリ用)。
    ///
    /// # Returns
    /// `Vec<(usize, usize)>`: パスを覆う半開区間の列。
    fn path_ranges(&self, mut u: usize, mut v: usize, exclude_lca: bool) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();

        // 異なるパスに属する間は、パスの根元がより深い側の区間を切り出し、
        // その親へ引き上げる。
        while self.head[u] != self.head[v] {
            if self.depth[self.head[u]] < self.depth[self.head[v]] {
                std::mem::swap(&mut u, &mut v);
            }
            ranges.push((self.id[self.head[u]], self.id[u] + 1));
            u = self.parent[self.head[u]].unwrap();
        }

        // 同じパスに入った時点で、浅い方が LCA になる。
        let (lca, deeper) = if self.id[u] <= self.id[v] {
            (u, v)
        } else {
            (v, u)
        };
        let start = if exclude_lca {
            self.id[lca] + 1
        } else {
            self.id[lca]
        };
        if start <= self.id[deeper] {
            ranges.push((start, self.id[deeper] + 1));
        }

        ranges
    }
}

impl<T> graph::Graph<T> {
    /// グラフが `root` を根とする木であることを検証したうえで、重軽分解
    /// (HLD) の前処理を行う。
    ///
    /// # Args
    /// - `root`: 根とする頂点。`0..vertex_count()` の範囲でなければならない。
    ///
    /// # Returns
    /// `Result<Hld, NotATreeError>`: 木であれば、各頂点の深さ・所属パス・
    /// 番号などの HLD の結果一式。木でなければ、その理由を表すエラー。
    ///
    /// # Panics
    /// - `root` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。木であることの検証も同じ O(V) に収まる。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::graph::Graph;
    ///
    /// let mut g = Graph::new(3);
    /// g.add_undirected_edge(0, 1, ());
    /// g.add_undirected_edge(1, 2, ());
    ///
    /// let hld = g.try_hld(0).unwrap();
    /// assert_eq!(0, hld.lca(0, 0, 2));
    /// ```
    pub fn try_hld(&self, root: usize) -> Result<Hld, NotATreeError> {
        graph::validate_tree(self, root)?;

        let n = self.vertex_count();

        // 第1段階: 深さ・親・部分木サイズを求める。木であることは検証済み
        // のため、ここでは反復深さ優先探索でそのまま辿るだけでよい。
        let mut parent: Vec<Option<usize>> = vec![None; n];
        let mut depth = vec![0; n];
        let mut subtree_size = vec![1; n];

        let mut stack = vec![Frame::Enter(root, None)];
        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Enter(u, p) => {
                    stack.push(Frame::Leave(u, p));
                    parent[u] = p;
                    if let Some(p) = p {
                        depth[u] = depth[p] + 1;
                    }
                    for (v, _) in self.edges(u) {
                        if Some(v) != p {
                            stack.push(Frame::Enter(v, Some(u)));
                        }
                    }
                }
                Frame::Leave(u, p) => {
                    if let Some(p) = p {
                        subtree_size[p] += subtree_size[u];
                    }
                }
            }
        }

        // 第2段階: 各頂点について、部分木サイズが最大の子 (重い子) を求める。
        let mut heavy_child: Vec<Option<usize>> = vec![None; n];
        for (u, heavy_child) in heavy_child.iter_mut().enumerate() {
            for (v, _) in self.edges(u) {
                if parent[v] == Some(u) {
                    let is_heavier = match *heavy_child {
                        Some(current) => subtree_size[v] > subtree_size[current],
                        None => true,
                    };
                    if is_heavier {
                        *heavy_child = Some(v);
                    }
                }
            }
        }

        // 第3段階: 重い子を優先して辿ることで、パスに沿って連続した番号を
        // 割り振る。スタックが後入れ先出しであることを利用し、軽い子を先に、
        // 重い子を最後に積むことで、重い子を次に処理させ、同じパスの番号を
        // 連続させる。
        let mut id = vec![0; n];
        let mut order = vec![0; n];
        let mut head = vec![0; n];
        let mut counter = 0;
        let mut stack = vec![(root, root)];
        while let Some((u, chain_head)) = stack.pop() {
            id[u] = counter;
            order[counter] = u;
            head[u] = chain_head;
            counter += 1;

            for (v, _) in self.edges(u) {
                if parent[v] == Some(u) && Some(v) != heavy_child[u] {
                    // 軽い子は、そこを頭とする新しいパスを始める。
                    stack.push((v, v));
                }
            }
            if let Some(v) = heavy_child[u] {
                stack.push((v, chain_head));
            }
        }

        Ok(Hld {
            root,
            parent,
            depth,
            head,
            id,
            order,
            subtree_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Background: 0 を根とし、0-1, 0-2, 1-3, 1-4, 3-5 の辺を持つ木。
    ///
    /// ```text
    ///         0
    ///        / \
    ///       1   2
    ///      / \
    ///     3   4
    ///     |
    ///     5
    /// ```
    /// 頂点1の部分木サイズが4 (1,3,4,5) と最大であるため、0からの重い辺は
    /// 0->1 になる。同様に1からの重い辺は3の部分木サイズが2 (3,5) で
    /// 4より大きいため 1->3 になる。
    fn create_tree() -> graph::Graph<()> {
        let mut g = graph::Graph::new(6);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(0, 2, ());
        g.add_undirected_edge(1, 3, ());
        g.add_undirected_edge(1, 4, ());
        g.add_undirected_edge(3, 5, ());
        g
    }

    /// Background: 0-1-2 の単純パス。
    fn create_path() -> graph::Graph<()> {
        let mut g = graph::Graph::new(3);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(1, 2, ());
        g
    }

    // try_hld のテスト: 戻り値そのもの (Ok/Err) を検証する。
    mod try_hld {
        use super::*;

        /// Scenario: 連結な木を渡すと、`Ok` が返る。
        /// - Given: 上記の木がある。
        /// - When: 頂点0を根に HLD の前処理を行う。
        /// - Then: `Ok` が返る。
        #[test]
        fn returns_ok_for_valid_tree() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0);
            // Then
            assert!(result.is_ok());
        }

        /// Scenario: 非連結なグラフを渡すと、`Disconnected` が返る。
        /// - Given: 孤立した頂点を含む、非連結なグラフがある。
        /// - When: 頂点0を根に HLD の前処理を行う。
        /// - Then: `Err(NotATreeError::Disconnected)` が返る。
        #[test]
        fn returns_disconnected_error_for_disconnected_graph() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            // 頂点2は孤立している。
            // When
            let result = sut.try_hld(0);
            // Then
            assert!(matches!(result, Err(NotATreeError::Disconnected)));
        }

        /// Scenario: 閉路を含むグラフを渡すと、`HasCycle` が返る。
        /// - Given: 0-1-2-0 の閉路を含むグラフがある。
        /// - When: 頂点0を根に HLD の前処理を行う。
        /// - Then: `Err(NotATreeError::HasCycle)` が返る。
        #[test]
        fn returns_has_cycle_error_for_graph_with_cycle() {
            // Given
            let mut sut = graph::Graph::new(3);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(1, 2, ());
            sut.add_undirected_edge(2, 0, ());
            // When
            let result = sut.try_hld(0);
            // Then
            assert!(matches!(result, Err(NotATreeError::HasCycle)));
        }

        /// Scenario: 多重辺があると、`HasCycle` が返る (境界値)。
        /// - Given: 0-1 間に2本の辺を持つグラフがある。
        /// - When: 頂点0を根に HLD の前処理を行う。
        /// - Then: `Err(NotATreeError::HasCycle)` が返る。
        #[test]
        fn returns_has_cycle_error_for_multi_edge() {
            // Given
            let mut sut = graph::Graph::new(2);
            sut.add_undirected_edge(0, 1, ());
            sut.add_undirected_edge(0, 1, ());
            // When
            let result = sut.try_hld(0);
            // Then
            assert!(matches!(result, Err(NotATreeError::HasCycle)));
        }

        /// Scenario: 頂点が1つだけの木でも正しく前処理できる (境界値)。
        /// - Given: 頂点数1、辺を持たない木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: `Ok` が返り、深さは0、部分木サイズは1になる。
        #[test]
        fn handles_single_vertex_tree() {
            // Given
            let sut = graph::Graph::<()>::new(1);
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.subtree_size(0, 0));
        }
    }

    // Hld (try_hld が返す Hld) のテスト: 各アクセサの戻り値を検証する。
    mod hld {
        use super::*;

        /// Scenario: 各頂点の深さが、根からの辺数と一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点0を根に前処理する。
        /// - Then: 各頂点の深さが期待通りになる。
        #[test]
        fn returns_depth_as_hop_count_from_root() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(0, result.depth(0));
            assert_eq!(1, result.depth(1));
            assert_eq!(3, result.depth(5));
        }

        /// Scenario: 通常の部分木サイズ (根の付け替えなし) は、実際にその
        /// 頂点の子孫の個数 (自身を含む) と一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点0を根に前処理し、構築時と同じ根を渡して部分木サイズ
        ///   を求める。
        /// - Then: 頂点1の部分木サイズは4 (1,3,4,5)、頂点3の部分木サイズは
        ///   2 (3,5) になる。
        #[test]
        fn returns_subtree_size_matching_descendant_count() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(4, result.subtree_size(0, 1));
            assert_eq!(2, result.subtree_size(0, 3));
            assert_eq!(1, result.subtree_size(0, 4));
        }

        /// Scenario: 根を付け替えると、部分木サイズが正しく変化する。
        /// - Given: 上記の木がある。
        /// - When: 頂点5を根と見なして、頂点1の部分木サイズを求める。
        /// - Then: 頂点1の部分木は「頂点3側を除いた残り全部」になり、
        ///   サイズは4 (全6頂点から、3を根とする部分木2 (3,5) を除いた分)
        ///   になる。
        #[test]
        fn returns_rerooted_subtree_size() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(4, result.subtree_size(5, 1));
        }

        /// Scenario: 付け替え先の根が、対象の部分木の外にある場合、部分木
        /// サイズは変化しない。
        /// - Given: 上記の木がある。
        /// - When: 頂点2を根と見なして、頂点1の部分木サイズを求める
        ///   (頂点2は頂点1の部分木の外にある)。
        /// - Then: 通常の部分木サイズ (4) のままになる。
        #[test]
        fn keeps_subtree_size_unchanged_when_new_root_is_outside() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(4, result.subtree_size(2, 1));
        }

        /// Scenario: 部分木に対応する区間の幅が、部分木サイズと一致する。
        /// - Given: 上記の木がある。
        /// - When: 頂点1の部分木の区間を求める。
        /// - Then: 区間の幅が、頂点1の部分木サイズ (4) と一致する。
        #[test]
        fn returns_subtree_range_with_width_matching_subtree_size() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            let (start, end) = result.subtree_range(1);
            // Then
            assert_eq!(4, end - start);
        }

        /// Scenario: 共通の祖先を持つ頂点同士の LCA は、その共通祖先になる。
        /// - Given: 上記の木がある。
        /// - When: 構築時と同じ根を渡して、頂点4と頂点5 (ともに頂点1の
        ///   子孫) の LCA を求める。
        /// - Then: 頂点1が返る。
        #[test]
        fn returns_common_ancestor_as_lca() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(1, result.lca(0, 4, 5));
        }

        /// Scenario: 異なる部分木に属する頂点同士の LCA は、根になる。
        /// - Given: 上記の木がある。
        /// - When: 頂点5 (頂点1の部分木) と頂点2 (根の直接の子) の LCA を
        ///   求める。
        /// - Then: 根 (頂点0) が返る。
        #[test]
        fn returns_root_for_vertices_in_different_subtrees() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(0, result.lca(0, 5, 2));
        }

        /// Scenario: 根を付け替えると、LCA が正しく変化する。
        /// - Given: 上記の木がある。
        /// - When: 頂点3を根と見なして、頂点0と頂点2のLCAを求める。
        /// - Then: 頂点0が返る (3を根とする木では、根から頂点2への経路が
        ///   3->1->0->2 となり、頂点0が頂点2の祖先になるため)。
        #[test]
        fn returns_rerooted_lca() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(0, result.lca(3, 0, 2));
        }

        /// Scenario: 距離は、両頂点の深さと LCA の深さから正しく求まる。
        /// - Given: 上記の木がある。
        /// - When: 頂点5と頂点4の距離を求める。
        /// - Then: 3 (5->3->1->4) になる。
        #[test]
        fn computes_distance_via_lca_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(3, result.distance(5, 4));
        }

        /// Scenario: k 個上の祖先が、根を超えない範囲で正しく求まる。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から2個上の祖先を求める。
        /// - Then: 頂点1が返る (5->3->1)。
        #[test]
        fn returns_kth_ancestor_within_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(Some(1), result.kth_ancestor(5, 2));
        }

        /// Scenario: 深さを超える歩数を指定すると `None` になる (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から4個上の祖先を求める (深さは3しかない)。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_when_k_exceeds_depth() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(None, result.kth_ancestor(5, 4));
        }

        /// Scenario: 0個上の祖先は、自分自身になる (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から0個上の祖先を求める。
        /// - Then: 頂点5自身が返る。
        #[test]
        fn returns_itself_for_zero_steps() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(Some(5), result.kth_ancestor(5, 0));
        }

        /// Scenario: 単純パス上で、始点から終点へ向かうジャンプが正しく
        /// 求まる。
        /// - Given: 0-1-2 の単純パスがある。
        /// - When: 頂点0から頂点2へ向かって1歩・2歩進んだ頂点を求める。
        /// - Then: それぞれ頂点1、頂点2が返る。
        #[test]
        fn jump_follows_path_from_start_to_end() {
            // Given
            let sut = create_path();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(Some(1), result.jump(0, 2, 1));
            assert_eq!(Some(2), result.jump(0, 2, 2));
        }

        /// Scenario: LCA を経由するジャンプも正しく求まる。
        /// - Given: 上記の木がある (頂点4から頂点5へのパスは 4-1-3-5)。
        /// - When: 頂点4から頂点5へ向かって2歩進んだ頂点を求める。
        /// - Then: 頂点3が返る (4->1->3)。
        #[test]
        fn jump_crosses_lca_correctly() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(Some(3), result.jump(4, 5, 2));
        }

        /// Scenario: パスの長さを超える歩数を指定すると `None` になる
        /// (境界値)。
        /// - Given: 0-1-2 の単純パスがある。
        /// - When: 頂点0から頂点2へ向かって3歩進んだ頂点を求める
        ///   (パスの長さは2しかない)。
        /// - Then: `None` が返る。
        #[test]
        fn jump_returns_none_when_k_exceeds_path_length() {
            // Given
            let sut = create_path();
            // When
            let result = sut.try_hld(0).unwrap();
            // Then
            assert_eq!(None, result.jump(0, 2, 3));
        }

        /// Scenario: 頂点パスの区間分解は、パス上のすべての頂点をちょうど
        /// 1回ずつ覆う。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から頂点4へのパスの区間分解を求める
        ///   (パスは 5-3-1-4 の4頂点からなる)。
        /// - Then: 区間の幅の合計が4になり、実際に5,3,1,4の番号のみを覆う。
        #[test]
        fn vertex_path_ranges_cover_every_vertex_on_path_exactly_once() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            let ranges = result.vertex_path_ranges(5, 4);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(4, total_width);

            let mut covered = ranges
                .iter()
                .flat_map(|&(l, r)| l..r)
                .collect::<Vec<usize>>();
            covered.sort_unstable();
            let mut expected = [5, 3, 1, 4]
                .iter()
                .map(|&v| result.vertex_id(v))
                .collect::<Vec<usize>>();
            expected.sort_unstable();
            assert_eq!(expected, covered);
        }

        /// Scenario: 辺パスの区間分解は、頂点パスの区間分解よりちょうど1
        /// 少ない個数の番号を覆う (LCA 自身の番号を含まないため)。
        /// - Given: 上記の木がある。
        /// - When: 頂点5から頂点4への辺の区間分解を求める
        ///   (パス上の辺は 5-3, 3-1, 1-4 の3本)。
        /// - Then: 区間の幅の合計が3になる。
        #[test]
        fn edge_path_ranges_exclude_lca_itself() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            let ranges = result.edge_path_ranges(5, 4);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(3, total_width);
        }

        /// Scenario: 一方が他方の祖先である場合、辺パスの区間分解は
        /// ちょうどその間の辺の本数を覆う (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点1 (祖先) から頂点5 (子孫) への辺の区間分解を求める
        ///   (パス上の辺は 1-3, 3-5 の2本)。
        /// - Then: 区間の幅の合計が2になる。
        #[test]
        fn edge_path_ranges_handle_ancestor_descendant_pair() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            let ranges = result.edge_path_ranges(1, 5);
            // Then
            let total_width = ranges.iter().map(|&(l, r)| r - l).sum::<usize>();
            assert_eq!(2, total_width);
        }

        /// Scenario: 同じ頂点同士のパスでは、頂点パスはその頂点自身のみを
        /// 覆い、辺パスは何も覆わない (境界値)。
        /// - Given: 上記の木がある。
        /// - When: 頂点3自身へのパスの区間分解を求める。
        /// - Then: 頂点パスの幅は1になり、辺パスの幅は0になる。
        #[test]
        fn handles_same_vertex_pair() {
            // Given
            let sut = create_tree();
            // When
            let result = sut.try_hld(0).unwrap();
            let vertex_width = result
                .vertex_path_ranges(3, 3)
                .iter()
                .map(|&(l, r)| r - l)
                .sum::<usize>();
            let edge_width = result
                .edge_path_ranges(3, 3)
                .iter()
                .map(|&(l, r)| r - l)
                .sum::<usize>();
            // Then
            assert_eq!(1, vertex_width);
            assert_eq!(0, edge_width);
        }
    }
}

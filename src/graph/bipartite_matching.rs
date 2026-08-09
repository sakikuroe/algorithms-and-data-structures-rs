//! Hopcroft-Karp 法による二部グラフの最大マッチングを提供するモジュールである。
//!
//! 左側・右側の頂点数と、両者をまたぐ辺のみを明示的に扱う専用の
//! `BipartiteGraph` を用いる。[`Graph<T>`](super::graph::Graph) は任意の
//! ペイロードを持つ一般の有向グラフを表すのに対し、二部マッチングでは
//! 「左側頂点の集合」「右側頂点の集合」という構造そのものがアルゴリズムの
//! 前提になるため、この区別を型で表現できる専用構造を用意する。
//!
//! アルゴリズムは、BFS で増加パスの最短長を求めるフェーズと、その長さの
//! パスをすべて (頂点素に) 見つけて反映する DFS フェーズを、増加パスが
//! 尽きるまで交互に繰り返す。1回の DFS フェーズで最短長のパスをまとめて
//! 反映することにより、フェーズの回数が O(sqrt(V)) 回に抑えられる。

use std::collections;

/// 二部グラフを、左側頂点から右側頂点への隣接リストとして表現する。
pub struct BipartiteGraph {
    /// 右側の頂点数。
    right_size: usize,
    /// `adjacency[u]` は、左側の頂点 `u` から辺が張られている右側の頂点の
    /// 一覧である。
    adjacency: Vec<Vec<usize>>,
}

impl BipartiteGraph {
    /// 左側 `left_size` 頂点、右側 `right_size` 頂点、辺を持たない二部グラフを
    /// 生成する。
    ///
    /// # Args
    /// - `left_size`: 左側の頂点数
    /// - `right_size`: 右側の頂点数
    ///
    /// # Returns
    /// `Self`: 指定した頂点数を持ち、辺を持たない `BipartiteGraph`
    ///
    /// # Complexity
    /// - 時間計算量: O(left_size)
    /// - 空間計算量: O(left_size)
    #[must_use]
    pub fn new(left_size: usize, right_size: usize) -> Self {
        BipartiteGraph {
            right_size,
            adjacency: vec![Vec::new(); left_size],
        }
    }

    /// 左側の頂点数を返す。
    ///
    /// # Returns
    /// `usize`: 左側の頂点数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn left_size(&self) -> usize {
        self.adjacency.len()
    }

    /// 右側の頂点数を返す。
    ///
    /// # Returns
    /// `usize`: 右側の頂点数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn right_size(&self) -> usize {
        self.right_size
    }

    /// 左側の頂点 `u` と右側の頂点 `v` を結ぶ辺を追加する。
    ///
    /// # Args
    /// - `u`: 左側の頂点番号であり、`0..left_size()` の範囲でなければならない。
    /// - `v`: 右側の頂点番号であり、`0..right_size()` の範囲でなければならない。
    ///
    /// # Panics
    /// - `u` が `0..left_size()` の範囲外の場合にパニックする。
    /// - デバッグビルドに限り、`v` が `0..right_size()` の範囲外の場合にも
    ///   パニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::bipartite_matching::BipartiteGraph;
    ///
    /// let mut g = BipartiteGraph::new(2, 2);
    /// g.add_edge(0, 1);
    /// let matching = g.hopcroft_karp();
    /// assert_eq!(1, matching.size());
    /// ```
    pub fn add_edge(&mut self, u: usize, v: usize) {
        debug_assert!(v < self.right_size);
        self.adjacency[u].push(v);
    }
}

/// Hopcroft-Karp 法の結果を保持する。
pub struct BipartiteMatching {
    /// `match_left[u]` は、左側の頂点 `u` にマッチした右側の頂点。マッチして
    /// いない場合は `None`。
    match_left: Vec<Option<usize>>,
    /// `match_right[v]` は、右側の頂点 `v` にマッチした左側の頂点。マッチして
    /// いない場合は `None`。
    match_right: Vec<Option<usize>>,
}

impl BipartiteMatching {
    /// マッチングに含まれる辺の本数を返す。
    ///
    /// # Returns
    /// `usize`: マッチした頂点対の個数
    ///
    /// # Complexity
    /// - 時間計算量: O(左側の頂点数)
    pub fn size(&self) -> usize {
        self.match_left.iter().filter(|m| m.is_some()).count()
    }

    /// 左側の頂点 `u` にマッチした右側の頂点を返す。
    ///
    /// # Args
    /// - `u`: 左側の頂点番号
    ///
    /// # Returns
    /// `Option<usize>`: マッチした右側の頂点であり、マッチしていない場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn match_of_left(&self, u: usize) -> Option<usize> {
        self.match_left[u]
    }

    /// 右側の頂点 `v` にマッチした左側の頂点を返す。
    ///
    /// # Args
    /// - `v`: 右側の頂点番号
    ///
    /// # Returns
    /// `Option<usize>`: マッチした左側の頂点であり、マッチしていない場合は
    /// `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn match_of_right(&self, v: usize) -> Option<usize> {
        self.match_right[v]
    }

    /// マッチした頂点対を `(左側の頂点, 右側の頂点)` の組として列挙する。
    ///
    /// # Returns
    /// `impl Iterator<Item = (usize, usize)>`: マッチした頂点対の列であり、
    /// 左側の頂点番号の昇順に並ぶ。
    ///
    /// # Complexity
    /// - 時間計算量: O(1) (返すイテレータの消費全体では、マッチした頂点対の
    ///   個数に比例する O(左側の頂点数))
    pub fn pairs(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.match_left
            .iter()
            .enumerate()
            .filter_map(|(u, m)| m.map(|v| (u, v)))
    }
}

impl BipartiteGraph {
    /// Hopcroft-Karp 法により、二部グラフの最大マッチングを求める。
    ///
    /// # Returns
    /// `BipartiteMatching`: 求めた最大マッチング
    ///
    /// # Complexity
    /// - 時間計算量: O(E sqrt(V))
    ///   - V は頂点数 (左側 + 右側)、E は辺数である。増加パスの探索は
    ///     O(sqrt(V)) 回のフェーズで完了し、各フェーズは O(E) で行える。
    /// - 空間計算量: O(V)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::bipartite_matching::BipartiteGraph;
    ///
    /// // 左0は右0のみ、左1は右0・右1の両方と辺を持つ。
    /// let mut g = BipartiteGraph::new(2, 2);
    /// g.add_edge(0, 0);
    /// g.add_edge(1, 0);
    /// g.add_edge(1, 1);
    ///
    /// let matching = g.hopcroft_karp();
    /// // 左0を右0に、左1を右1に回すことで、両方をマッチさせられる。
    /// assert_eq!(2, matching.size());
    /// ```
    pub fn hopcroft_karp(&self) -> BipartiteMatching {
        let mut match_left: Vec<Option<usize>> = vec![None; self.left_size()];
        let mut match_right: Vec<Option<usize>> = vec![None; self.right_size()];

        loop {
            let Some(level) = self.bfs_layers(&match_left, &match_right) else {
                // 増加パスがもう存在しない。
                break;
            };

            // 「現在辺」ポインタ。同じフェーズの間、頂点ごとに使い回すことで、
            // 探索済みで行き止まりと分かった辺を再訪問しない。
            let mut iter = vec![0_usize; self.left_size()];
            // 未マッチの左側頂点それぞれを始点として、直前の BFS で求めた層に
            // 沿う増加パスを探す。見つかったパスはその場でマッチングへ反映され、
            // 次の頂点を始点とする探索は反映後の状態から始まる。
            for u in 0..self.left_size() {
                if match_left[u].is_none() {
                    self.try_augment(u, &level, &mut iter, &mut match_left, &mut match_right);
                }
            }
        }

        BipartiteMatching {
            match_left,
            match_right,
        }
    }

    /// 未マッチの左側頂点すべてを始点とする BFS により、増加パスの最短長
    /// (層) を求める。
    ///
    /// # Args
    /// - `match_left`/`match_right`: 現在のマッチング
    ///
    /// # Returns
    /// `Option<Vec<Option<usize>>>`: 未マッチの右側頂点へ到達できた場合、
    /// 各左側頂点の層 (`None` はこのフェーズで到達しなかったことを表す) を
    /// `Some` で返す。1つも到達できなかった場合は `None`
    /// (増加パスが尽きたことを表す)。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    fn bfs_layers(
        &self,
        match_left: &[Option<usize>],
        match_right: &[Option<usize>],
    ) -> Option<Vec<Option<usize>>> {
        let mut level = vec![None; self.left_size()];
        let mut queue = collections::VecDeque::new();
        // 未マッチの左側頂点をすべて層0としてキューに積み、そこから同時に
        // BFS を始める。
        for u in 0..self.left_size() {
            if match_left[u].is_none() {
                level[u] = Some(0);
                queue.push_back(u);
            }
        }

        // 未マッチの右側頂点に到達できた層。これより深い層を探索しても、
        // より短い増加パスは見つからないため探索を打ち切る。
        let mut target_level = None;

        while let Some(u) = queue.pop_front() {
            // u の層がすでに target_level に達しているなら、その先を辿っても
            // 最短の増加パスにはならないため、u からの探索は打ち切る。
            if target_level.is_some_and(|tl| level[u].unwrap() >= tl) {
                continue;
            }
            // u から辺で結ばれた右側頂点 v を1つずつ調べる。
            for &v in &self.adjacency[u] {
                match match_right[v] {
                    None => {
                        // v は未マッチであり、u から v への増加パスが見つかった。
                        // ここでは最短の層を記録するだけにとどめる (経路そのもの
                        // は、後で try_augment が層をもとに改めて辿るため、
                        // v をキューに積む必要はない)。
                        target_level.get_or_insert(level[u].unwrap() + 1);
                    }
                    Some(u2) => {
                        // v はすでにマッチ済みであり、その相手 u2 へ1つ余分に
                        // 進んでから探索を続ける。u2 を初めて訪れた場合のみ層を
                        // 記録してキューに積む。
                        if level[u2].is_none() {
                            level[u2] = Some(level[u].unwrap() + 1);
                            queue.push_back(u2);
                        }
                    }
                }
            }
        }

        target_level.map(|_| level)
    }

    /// 層 `level` に沿って、左側の頂点 `u0` を始点とする増加パスを探し、
    /// 見つかった場合はマッチングへ反映する。
    ///
    /// 明示的なスタックで `u0` からの経路を管理する。ある頂点で使える辺が
    /// 尽きたら (行き止まりなら) その頂点をスタックから外す。`iter` は
    /// 同じフェーズの間使い回されるため、一度行き止まりと分かった頂点を
    /// 二度と訪問しない。
    ///
    /// # Args
    /// - `u0`: 増加パスの始点となる、未マッチの左側頂点
    /// - `level`: [`bfs_layers`](Self::bfs_layers) が返した層
    /// - `iter`: 頂点ごとの「現在辺」ポインタであり、同じフェーズの間、
    ///   呼び出し元で使い回す。
    /// - `match_left`/`match_right`: 現在のマッチングであり、増加パスが
    ///   見つかった場合、その場で更新する。
    ///
    /// # Returns
    /// `bool`: 増加パスが見つかり、マッチングを更新できた場合は `true`
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(E) (`iter` を同一フェーズ内で使い回すため)
    fn try_augment(
        &self,
        u0: usize,
        level: &[Option<usize>],
        iter: &mut [usize],
        match_left: &mut [Option<usize>],
        match_right: &mut [Option<usize>],
    ) -> bool {
        // u0 から続く、まだ行き止まりと分からずに辿れている左側頂点の列。
        let mut stack = vec![u0];

        while let Some(&u) = stack.last() {
            // この頂点 u からスタックへ新たに1段積めた場合に true にする。
            let mut advanced = false;
            // u に残っている辺を、前回このフェーズで調べ終えた続きから
            // 1本ずつ調べる (iter[u] より前は、すでに行き止まりと分かって
            // いるため調べ直さない)。
            while iter[u] < self.adjacency[u].len() {
                let v = self.adjacency[u][iter[u]];
                match match_right[v] {
                    None => {
                        // v は未マッチであり、増加パスが見つかった。
                        // スタック上の各左側頂点を、その1つ手前で使った
                        // 右側頂点へ順にマッチさせ直していく。
                        let mut v = v;
                        while let Some(u) = stack.pop() {
                            let previous_match = match_left[u];
                            match_left[u] = Some(v);
                            match_right[v] = Some(u);
                            match previous_match {
                                Some(prev_v) => v = prev_v,
                                None => break,
                            }
                        }
                        return true;
                    }
                    Some(u2) => {
                        // v はすでにマッチ済みであり、その相手が u2 である。
                        // u2 の層が u のちょうど1つ先である場合に限り、
                        // bfs_layers が見つけた最短の増加パスに沿っていると
                        // 判断してスタックへ積む。層が合わない辺を辿っても
                        // 最短のパスにはならないため無視する。
                        if level[u2] == level[u].map(|lv| lv + 1) {
                            stack.push(u2);
                            advanced = true;
                            break;
                        }
                    }
                }
                iter[u] += 1;
            }

            if !advanced {
                // u からはこれ以上増加パスを伸ばせない (行き止まり) ため、
                // スタックから外す。1つ手前の頂点から見て、u へ向かう辺は
                // もう使えないと分かったため、次に試す辺へ進めておく。
                // これを怠ると、同じ行き止まりの辺を無限に辿り直してしまう。
                stack.pop();
                if let Some(&parent) = stack.last() {
                    iter[parent] += 1;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // hopcroft_karp のテスト: 戻り値 (BipartiteMatching) が保持するマッチング
    // を検証する。
    mod hopcroft_karp {
        use super::*;

        /// Scenario: 完全に独立した辺は、そのままマッチングに採用される。
        /// - Given: `左0-右0` と `左1-右1` の、互いに独立した2辺がある。
        /// - When: 最大マッチングを求める。
        /// - Then: マッチングのサイズは2になり、両方のペアが採用される。
        #[test]
        fn matches_independent_edges() {
            // Given
            let mut sut = BipartiteGraph::new(2, 2);
            sut.add_edge(0, 0);
            sut.add_edge(1, 1);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(2, result.size());
            assert_eq!(Some(0), result.match_of_left(0));
            assert_eq!(Some(1), result.match_of_left(1));
        }

        /// Scenario: 競合する辺があっても、増加パスにより最大サイズの
        /// マッチングが得られる。
        /// - Given: 左0が右0のみと、左1が右0・右1の両方と辺を持つ。
        /// - When: 最大マッチングを求める。
        /// - Then: マッチングのサイズは2になる (左1を右1に回すことで、
        ///   左0と右0のペアも成立させられる)。
        #[test]
        fn finds_maximum_matching_via_augmenting_path() {
            // Given
            let mut sut = BipartiteGraph::new(2, 2);
            sut.add_edge(0, 0);
            sut.add_edge(1, 0);
            sut.add_edge(1, 1);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(2, result.size());
        }

        /// Scenario: 2フェーズ目の増加パス探索で、行き止まりの子を持つ
        /// 頂点を経由しても無限ループに陥らず、正しく完了する。
        /// - Given: 1フェーズ目の後に左2が未マッチのまま残り、左2から
        ///   見て最初に辿る辺の先 (左0) が行き止まりになる構成のグラフが
        ///   ある (左0は右0のみにしか辺を持たないため、必ず行き止まりに
        ///   なる)。
        /// - When: 最大マッチングを求める。
        /// - Then: 全頂点がマッチし、マッチングのサイズは3になる。
        #[test]
        fn does_not_loop_forever_when_augmenting_path_revisits_dead_end() {
            // Given
            let mut sut = BipartiteGraph::new(3, 3);
            sut.add_edge(0, 0);
            sut.add_edge(1, 1);
            sut.add_edge(1, 2);
            sut.add_edge(2, 0);
            sut.add_edge(2, 1);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(3, result.size());
        }

        /// Scenario: 左右の頂点数が異なり、全員をマッチさせられない場合、
        /// マッチングのサイズは小さい方の頂点数を超えない。
        /// - Given: 左3頂点すべてが右0とのみ辺を持つ (右は1頂点のみ)。
        /// - When: 最大マッチングを求める。
        /// - Then: マッチングのサイズは1になる。
        #[test]
        fn bounds_matching_size_by_smaller_side() {
            // Given
            let mut sut = BipartiteGraph::new(3, 1);
            sut.add_edge(0, 0);
            sut.add_edge(1, 0);
            sut.add_edge(2, 0);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(1, result.size());
        }

        /// Scenario: 辺を持たないグラフでは、マッチングのサイズは0になる
        /// (境界値)。
        /// - Given: 左右とも頂点を持つが、辺を持たないグラフがある。
        /// - When: 最大マッチングを求める。
        /// - Then: マッチングのサイズは0になる。
        #[test]
        fn returns_empty_matching_without_edges() {
            // Given
            let sut = BipartiteGraph::new(2, 2);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(0, result.size());
        }

        /// Scenario: 呼び出し後、マッチした頂点対を pairs で列挙できる。
        /// - Given: `左0-右1` と `左1-右0` の2辺がある。
        /// - When: 最大マッチングを求める。
        /// - Then: `pairs` が `(0, 1)` と `(1, 0)` を返す。
        #[test]
        fn enumerates_matched_pairs() {
            // Given
            let mut sut = BipartiteGraph::new(2, 2);
            sut.add_edge(0, 1);
            sut.add_edge(1, 0);
            // When
            let result = sut.hopcroft_karp();
            // Then
            let pairs = result.pairs().collect::<Vec<(usize, usize)>>();
            assert_eq!(vec![(0, 1), (1, 0)], pairs);
        }

        /// Scenario: 右側の対応する頂点からも、マッチした相手を参照できる。
        /// - Given: `左0-右0` の1辺がある。
        /// - When: 最大マッチングを求める。
        /// - Then: `match_of_right(0)` が `Some(0)` を返す。
        #[test]
        fn exposes_match_from_right_side_too() {
            // Given
            let mut sut = BipartiteGraph::new(1, 1);
            sut.add_edge(0, 0);
            // When
            let result = sut.hopcroft_karp();
            // Then
            assert_eq!(Some(0), result.match_of_right(0));
        }
    }
}

//! Project Selection Problem (燃やす埋める問題) を最小カットに帰着させて解くための
//! モジュールである。
//!
//! 各頂点に「選ぶ/選ばない」の二値を割り当てる問題のうち、次のような損得・制約の
//! 組み合わせで表現できるものを、[`FlowGraph`] 上の最小カットとして解く。
//!
//! - 頂点ごとの損得 ([`add_weight`](ProjectSelection::add_weight))
//! - 「u を選ぶなら v も選ばなければならない」という含意制約
//!   ([`add_constraint`](ProjectSelection::add_constraint))
//! - 複数の頂点をすべて選んだ場合にのみ得られる追加の報酬
//!   ([`add_and_bonus_when_selected`](ProjectSelection::add_and_bonus_when_selected))
//! - 複数の頂点をすべて選ばなかった場合にのみ得られる追加の報酬
//!   ([`add_and_bonus_when_unselected`](ProjectSelection::add_and_bonus_when_unselected))
//!
//! 「選ぶ」を、最大流を求めたあとの残余グラフで始点から到達できる側 (
//! [`min_cut`](super::max_flow) が返す側) と対応させる。頂点ごとの損得は、正なら
//! 始点からその頂点への辺、負ならその頂点から終点への辺として表現し、選ばれな
//! かった (または選ばれた) 場合にその分の損得を最小カットで打ち消す、という
//! 考え方に基づく。後者2つの「すべて選んだ/選ばなかった場合の報酬」は、報酬を
//! 表す補助頂点をもう1つ用意し、対象の頂点との間に含意制約を張ることで表現する。

use super::flow_graph::{self, FlowGraph};

/// Project Selection Problem を表現し、最小カットで解くための構造体である。
pub struct ProjectSelection<Cap> {
    flow: FlowGraph<Cap>,
    source: usize,
    sink: usize,
    /// これまでに `add_weight` で加えた正の損得、および `add_and_bonus_when_*`
    /// で加えた報酬の総和。選ばれなかった (または選ばれた) ことで打ち消される
    /// 分を、最終的に最小カットの値だけ差し引く際の基準値として使う。
    total_positive_weight: Cap,
}

impl<Cap: flow_graph::FlowCapacity> ProjectSelection<Cap> {
    /// `n` 頂点分の Project Selection Problem を用意する。
    ///
    /// 頂点 `0..n` が選択対象の頂点である。
    /// [`add_and_bonus_when_selected`](Self::add_and_bonus_when_selected) などで
    /// 補助頂点を使う場合は、その分もあらかじめ `n` に含めておく必要がある。
    ///
    /// # Args
    /// - `n`: 選択対象の頂点数であり、補助頂点として使う分も含める
    ///
    /// # Returns
    /// `Self`: 頂点数 `n`、損得・制約を持たない `ProjectSelection<Cap>`
    ///
    /// # Complexity
    /// - 時間計算量: O(n)
    #[must_use]
    pub fn new(n: usize) -> Self {
        ProjectSelection {
            flow: FlowGraph::new(n + 2),
            source: n,
            sink: n + 1,
            total_positive_weight: Cap::ZERO,
        }
    }

    /// 頂点 `v` を選んだ場合の損得 `weight` を加える。
    ///
    /// `weight` が正であれば選んだ場合にその分の得を、負であれば選んだ場合に
    /// その分の損を表す (選ばなければ、この呼び出しによる影響は 0 のままで
    /// ある)。同じ頂点に複数回呼び出した場合、損得は加算される。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..n` の範囲でなければならない (`n` は
    ///   [`new`](Self::new) に渡した値である)。
    /// - `weight`: `v` を選んだ場合の損得
    ///
    /// # Panics
    /// - `v` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    pub fn add_weight(&mut self, v: usize, weight: Cap) {
        if weight > Cap::ZERO {
            self.total_positive_weight = self.total_positive_weight + weight;
            self.flow.add_edge(self.source, v, weight);
        } else if weight < Cap::ZERO {
            self.flow.add_edge(v, self.sink, Cap::ZERO - weight);
        }
    }

    /// 頂点 `v` を必ず選ぶという制約を加える。
    ///
    /// あらかじめ値が確定している頂点を表すのに使う。[`add_weight`](Self::add_weight)
    /// と異なり、損得の合計には影響しない。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..n` の範囲でなければならない。
    ///
    /// # Panics
    /// - `v` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    pub fn force_selected(&mut self, v: usize) {
        self.flow.add_edge(self.source, v, Cap::MAX);
    }

    /// 頂点 `v` を必ず選ばないという制約を加える。
    ///
    /// [`force_selected`](Self::force_selected) と対になる制約であり、損得の
    /// 合計には影響しない。
    ///
    /// # Args
    /// - `v`: 頂点であり、`0..n` の範囲でなければならない。
    ///
    /// # Panics
    /// - `v` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    pub fn force_unselected(&mut self, v: usize) {
        self.flow.add_edge(v, self.sink, Cap::MAX);
    }

    /// 「頂点 `u` を選ぶなら頂点 `v` も選ばなければならない」という制約を加える。
    ///
    /// # Args
    /// - `u`: 選ぶ側の頂点であり、`0..n` の範囲でなければならない。
    /// - `v`: 連動して選ばれる側の頂点であり、`0..n` の範囲でなければならない。
    ///
    /// # Panics
    /// - `u`/`v` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    pub fn add_constraint(&mut self, u: usize, v: usize) {
        self.flow.add_edge(u, v, Cap::MAX);
    }

    /// `vertices` に含まれる頂点をすべて選んだ場合にのみ、報酬 `bonus` を得る
    /// という制約を加える。
    ///
    /// 内部では、報酬を表す補助頂点 `aux` を「選ぶと `bonus` を得る」頂点として
    /// 扱ったうえで、`aux` を選ぶなら `vertices` の各頂点も選ばなければならない
    /// という制約を加える。こうすることで、`aux` を選んでも損をしないのは
    /// `vertices` がすべて選ばれているときに限られ、最小カットを求める過程で
    /// 報酬を得るべきかどうかも同時に最適化される。
    ///
    /// # Args
    /// - `vertices`: すべて選ばれていないと報酬を得られない頂点の集合であり、
    ///   各要素は `0..n` の範囲でなければならない。
    /// - `bonus`: すべて選んだ場合に得る報酬
    /// - `aux`: この制約の表現に使う補助頂点であり、`0..n` の範囲でなければ
    ///   ならず、他の用途に使っていない頂点でなければならない。
    ///
    /// # Panics
    /// - `vertices` の要素や `aux` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(|vertices|)
    pub fn add_and_bonus_when_selected(&mut self, vertices: &[usize], bonus: Cap, aux: usize) {
        if bonus <= Cap::ZERO {
            return;
        }
        self.total_positive_weight = self.total_positive_weight + bonus;
        self.flow.add_edge(self.source, aux, bonus);
        for &v in vertices {
            self.flow.add_edge(aux, v, Cap::MAX);
        }
    }

    /// `vertices` に含まれる頂点をすべて選ばなかった場合にのみ、報酬 `bonus` を
    /// 得るという制約を加える。
    ///
    /// [`add_and_bonus_when_selected`](Self::add_and_bonus_when_selected) と
    /// 対になる変換であり、`vertices` のいずれかを選ぶなら補助頂点 `aux` も
    /// 選ばなければならない、という向きの制約を加えることで、`vertices` が
    /// すべて選ばれていないときに限って `aux` を選ばずに済み、報酬を得られる
    /// ようにする。
    ///
    /// # Args
    /// - `vertices`: すべて選ばれていると報酬を得られない頂点の集合であり、
    ///   各要素は `0..n` の範囲でなければならない。
    /// - `bonus`: すべて選ばなかった場合に得る報酬
    /// - `aux`: この制約の表現に使う補助頂点であり、`0..n` の範囲でなければ
    ///   ならず、他の用途に使っていない頂点でなければならない。
    ///
    /// # Panics
    /// - `vertices` の要素や `aux` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(|vertices|)
    pub fn add_and_bonus_when_unselected(&mut self, vertices: &[usize], bonus: Cap, aux: usize) {
        if bonus <= Cap::ZERO {
            return;
        }
        self.total_positive_weight = self.total_positive_weight + bonus;
        self.flow.add_edge(aux, self.sink, bonus);
        for &v in vertices {
            self.flow.add_edge(v, aux, Cap::MAX);
        }
    }

    /// これまでに加えた損得・制約のもとで達成できる、損得の合計の最大値と、
    /// それを実現する選択を求める。
    ///
    /// # Returns
    /// `(Cap, Vec<bool>)`: 損得の合計の最大値であり、`selected[v]` が `true`
    /// であれば頂点 `v` (`0..n`、`n` は [`new`](Self::new) に渡した値) が
    /// 選ばれていることを表す配列
    ///
    /// # Complexity
    /// - 時間計算量: [`max_flow`](FlowGraph::max_flow) の計算量に等しい
    pub fn solve(&mut self) -> (Cap, Vec<bool>) {
        let cut = self.flow.max_flow(self.source, self.sink);
        let reachable = self.flow.min_cut(self.source);
        let selected = reachable[..self.source].to_vec();
        (self.total_positive_weight - cut, selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // force_selected/force_unselected のテスト: 値があらかじめ確定している
    // 頂点を、損得に影響を与えずに固定できるかを検証する。
    mod force {
        use super::*;

        /// Scenario: 単体では損な頂点でも、force_selected で固定すると選ばれる。
        /// - Given: 頂点0 (損-5) を持つ ProjectSelection がある。
        /// - When: 頂点0を force_selected で固定して解く。
        /// - Then: 頂点0が選ばれ、合計値は -5 になる (固定自体は損得に影響しない)。
        #[test]
        fn selects_forced_vertex_even_at_a_loss() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(1);
            sut.add_weight(0, -5);
            // When
            sut.force_selected(0);
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(-5, total);
            assert!(selected[0]);
        }

        /// Scenario: 単体では得な頂点でも、force_unselected で固定すると選ばれない。
        /// - Given: 頂点0 (得+5) を持つ ProjectSelection がある。
        /// - When: 頂点0を force_unselected で固定して解く。
        /// - Then: 頂点0は選ばれず、合計値は 0 になる (固定自体は損得に影響しない)。
        #[test]
        fn excludes_forced_vertex_even_at_a_gain() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(1);
            sut.add_weight(0, 5);
            // When
            sut.force_unselected(0);
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(0, total);
            assert!(!selected[0]);
        }
    }

    // add_weight/add_constraint のテスト: 頂点ごとの損得と含意制約のみを
    // 組み合わせた場合の戻り値 (合計値・選択) を検証する。
    mod weight_and_constraint {
        use super::*;

        /// Scenario: 損得のみでは損をする頂点は選ばれない。
        /// - Given: 頂点0 (得+10)、頂点1 (損-3) を持つ ProjectSelection がある。
        /// - When: 特に制約を加えずに解く。
        /// - Then: 頂点0のみが選ばれ、合計値は10になる。
        #[test]
        fn skips_vertex_that_only_loses() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(2);
            sut.add_weight(0, 10);
            sut.add_weight(1, -3);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(10, total);
            assert_eq!(vec![true, false], selected);
        }

        /// Scenario: 含意制約により、単体では損な頂点も連動して選ばれる。
        /// - Given: 頂点0 (得+10)、頂点1 (損-3) を持ち、「0を選ぶなら1も選ぶ」
        ///   という制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: 両方選ばれ、合計値は7 (10-3) になる。
        #[test]
        fn forces_dependent_vertex_via_constraint() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(2);
            sut.add_weight(0, 10);
            sut.add_weight(1, -3);
            sut.add_constraint(0, 1);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(7, total);
            assert_eq!(vec![true, true], selected);
        }

        /// Scenario: 制約により連動先の損が大きすぎる場合は、そもそも選ばない。
        /// - Given: 頂点0 (得+10)、頂点1 (損-20) を持ち、「0を選ぶなら1も選ぶ」
        ///   という制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: どちらも選ばれず、合計値は0になる。
        #[test]
        fn avoids_vertex_when_forced_loss_outweighs_gain() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(2);
            sut.add_weight(0, 10);
            sut.add_weight(1, -20);
            sut.add_constraint(0, 1);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(0, total);
            assert_eq!(vec![false, false], selected);
        }
    }

    // add_and_bonus_when_selected のテスト: すべて選んだ場合にのみ得られる
    // 報酬が正しく反映されるかを検証する。
    mod and_bonus_when_selected {
        use super::*;

        /// Scenario: 損得の無い頂点同士なら、報酬を得るために両方選ばれる。
        /// - Given: 頂点0, 1 (損得0) と補助頂点2 を持ち、両方選んだ場合に
        ///   報酬5を得る制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: 頂点0, 1 が選ばれ、合計値は5になる。
        #[test]
        fn earns_bonus_when_free_to_select_both() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(3);
            sut.add_and_bonus_when_selected(&[0, 1], 5, 2);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(5, total);
            assert!(selected[0]);
            assert!(selected[1]);
        }

        /// Scenario: 一方に選ぶと損をする理由がある場合、報酬をあきらめて
        /// そちらを優先することがある。
        /// - Given: 頂点0 (損-10)、頂点1 (損得0) と補助頂点2 を持ち、両方選んだ
        ///   場合に報酬5を得る制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: 頂点0は選ばれず、合計値は0になる (報酬をあきらめる方が得)。
        #[test]
        fn gives_up_bonus_when_forcing_costs_more() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(3);
            sut.add_weight(0, -10);
            sut.add_and_bonus_when_selected(&[0, 1], 5, 2);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(0, total);
            assert!(!selected[0]);
        }
    }

    // add_and_bonus_when_unselected のテスト: すべて選ばなかった場合にのみ
    // 得られる報酬が正しく反映されるかを検証する。
    mod and_bonus_when_unselected {
        use super::*;

        /// Scenario: 損得の無い頂点同士なら、報酬を得るために両方選ばれない。
        /// - Given: 頂点0, 1 (損得0) と補助頂点2 を持ち、両方選ばなかった場合に
        ///   報酬5を得る制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: 頂点0, 1 はいずれも選ばれず、合計値は5になる。
        #[test]
        fn earns_bonus_when_free_to_leave_both_unselected() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(3);
            sut.add_and_bonus_when_unselected(&[0, 1], 5, 2);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(5, total);
            assert!(!selected[0]);
            assert!(!selected[1]);
        }

        /// Scenario: 選ぶ理由がある頂点を選ぶと、報酬をあきらめることがある。
        /// - Given: 頂点0 (得+10)、頂点1 (損得0) と補助頂点2 を持ち、両方選ば
        ///   なかった場合に報酬5を得る制約を加えた ProjectSelection がある。
        /// - When: 解く。
        /// - Then: 頂点0が選ばれ、合計値は10になる (報酬をあきらめる方が得)。
        #[test]
        fn gives_up_bonus_when_selecting_is_more_profitable() {
            // Given
            let mut sut = ProjectSelection::<i64>::new(3);
            sut.add_weight(0, 10);
            sut.add_and_bonus_when_unselected(&[0, 1], 5, 2);
            // When
            let (total, selected) = sut.solve();
            // Then
            assert_eq!(10, total);
            assert!(selected[0]);
        }
    }
}

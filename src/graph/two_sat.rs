//! 2-SAT (2充足可能性問題) を解くためのモジュールである。
//!
//! 各節 `(x_i = f) ∨ (x_j = g)` を、含意グラフ上の2本の有向辺
//! `¬(x_i = f) -> (x_j = g)` と `¬(x_j = g) -> (x_i = f)` として表現し、
//! 強連結成分分解 (SCC) により充足可能性を判定する。変数 `i` を表す2つの
//! リテラル `x_i = true` と `x_i = false` が同じ強連結成分に属するならば、
//! 一方からもう一方が導出可能 (矛盾) であり、充足不能である。

use super::graph;

/// 2-SAT のインスタンスを構築し、解を求める。
pub struct TwoSat {
    /// ブール変数の個数。
    n: usize,
    /// 含意グラフ。頂点 `2*i` はリテラル `x_i = true`、頂点 `2*i + 1` は
    /// リテラル `x_i = false` に対応する。
    graph: graph::Graph<()>,
}

impl TwoSat {
    /// `n` 個のブール変数を持ち、節を1つも持たない 2-SAT インスタンスを生成する。
    ///
    /// # Args
    /// - `n`: ブール変数の個数
    ///
    /// # Returns
    /// `Self`: 節を持たない 2-SAT インスタンス
    ///
    /// # Complexity
    /// - 時間計算量: O(n)
    #[must_use]
    pub fn new(n: usize) -> Self {
        TwoSat {
            n,
            graph: graph::Graph::new(2 * n),
        }
    }

    /// リテラル `(i, f)` (変数 `i` が `f` であること) を表す頂点番号を返す。
    fn literal(&self, i: usize, f: bool) -> usize {
        2 * i + usize::from(f)
    }

    /// 節 `(x_i = f) ∨ (x_j = g)` を追加する。
    ///
    /// # Args
    /// - `i`: 節に含める一方の変数番号であり、`0..n` の範囲でなければならない。
    /// - `f`: 変数 `i` が満たすべき値
    /// - `j`: 節に含めるもう一方の変数番号であり、`0..n` の範囲でなければならない。
    /// - `g`: 変数 `j` が満たすべき値
    ///
    /// # Panics
    /// - `i`/`j` が `0..n` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::two_sat::TwoSat;
    ///
    /// let mut sat = TwoSat::new(1);
    /// sat.add_clause(0, true, 0, true); // x_0 は true でなければならない
    /// let result = sat.solve().unwrap();
    /// assert!(result[0]);
    /// ```
    pub fn add_clause(&mut self, i: usize, f: bool, j: usize, g: bool) {
        self.graph
            .add_edge(self.literal(i, !f), self.literal(j, g), ());
        self.graph
            .add_edge(self.literal(j, !g), self.literal(i, f), ());
    }

    /// 充足可能性を判定し、可能であれば各変数への割り当てを返す。
    ///
    /// # Returns
    /// `Option<Vec<bool>>`: 充足可能な場合、各変数への割り当てであり、
    /// 充足不能な場合は `None` である。
    ///
    /// # Complexity
    /// - 時間計算量: O(V + E)
    ///   - V, E はそれぞれ含意グラフの頂点数・辺数であり、いずれも節の本数に
    ///     比例する。
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::two_sat::TwoSat;
    ///
    /// let mut sat = TwoSat::new(2);
    /// sat.add_clause(0, true, 1, true);
    /// sat.add_clause(0, false, 1, false);
    /// let result = sat.solve().unwrap();
    /// assert_ne!(result[0], result[1]);
    /// ```
    pub fn solve(&self) -> Option<Vec<bool>> {
        let scc = self.graph.scc();

        let mut assignment = vec![false; self.n];
        for i in 0..self.n {
            let id_true = scc.component_id(self.literal(i, true));
            let id_false = scc.component_id(self.literal(i, false));
            if id_true == id_false {
                // x_i = true と x_i = false が互いに導出可能であり矛盾する。
                return None;
            }
            // 強連結成分の番号は含意グラフの辺に沿って昇順になる
            // (`Graph::scc` を参照)。「後の (番号が大きい) 方」が実際に
            // 選ばれるべき値であり、これは真偽どちらのリテラルがより強い
            // 制約から導出されるかに対応する。
            assignment[i] = id_true > id_false;
        }

        Some(assignment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // add_clause/solve のテスト: 戻り値が各節を満たすかどうかを検証する。
    mod solve {
        use super::*;

        /// Scenario: 単一の節から、それを満たす一意な割り当てが得られる。
        /// - Given: `x_0 = true` を強制する節を持つインスタンスがある。
        /// - When: 充足可能性を判定する。
        /// - Then: `x_0` は `true` になる。
        #[test]
        fn forces_variable_to_required_value() {
            // Given
            let mut sut = TwoSat::new(1);
            sut.add_clause(0, true, 0, true);
            // When
            let result = sut.solve();
            // Then
            assert_eq!(Some(vec![true]), result);
        }

        /// Scenario: 矛盾する節を持つインスタンスは充足不能と判定される。
        /// - Given: `x_0 = true` と `x_0 = false` を同時に強制する節を持つ
        ///   インスタンスがある。
        /// - When: 充足可能性を判定する。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_contradictory_clauses() {
            // Given
            let mut sut = TwoSat::new(1);
            sut.add_clause(0, true, 0, true);
            sut.add_clause(0, false, 0, false);
            // When
            let result = sut.solve();
            // Then
            assert!(result.is_none());
        }

        /// Scenario: 充足可能な場合、得られた割り当てはすべての節を満たす。
        /// - Given: `x_0 ∨ x_1` と `¬x_0 ∨ ¬x_1` (排他的論理和に相当) の節を
        ///   持つインスタンスがある。
        /// - When: 充足可能性を判定する。
        /// - Then: 得られた割り当てで、両方の節が満たされる。
        #[test]
        fn returns_assignment_satisfying_all_clauses() {
            // Given
            let mut sut = TwoSat::new(2);
            let clauses = [(0, true, 1, true), (0, false, 1, false)];
            for &(i, f, j, g) in &clauses {
                sut.add_clause(i, f, j, g);
            }
            // When
            let result = sut.solve();
            // Then
            let assignment = result.unwrap();
            for (i, f, j, g) in clauses {
                assert!(assignment[i] == f || assignment[j] == g);
            }
        }

        /// Scenario: 節を1つも持たないインスタンスは、任意の割り当てで
        /// 充足可能である (境界値)。
        /// - Given: 節を持たないインスタンスがある。
        /// - When: 充足可能性を判定する。
        /// - Then: 何らかの割り当てが返る。
        #[test]
        fn returns_some_assignment_when_no_clauses_exist() {
            // Given
            let sut = TwoSat::new(3);
            // When
            let result = sut.solve();
            // Then
            assert!(result.is_some());
        }

        /// Scenario: 複数の節からなる、より複雑なインスタンスでも、すべての
        /// 節を満たす割り当てが得られる。
        /// - Given: 3変数にわたる複数の節を持つ、充足可能なインスタンスがある。
        /// - When: 充足可能性を判定する。
        /// - Then: 得られた割り当てで、すべての節が満たされる。
        #[test]
        fn satisfies_all_clauses_in_larger_instance() {
            // Given
            let mut sut = TwoSat::new(3);
            let clauses = [
                (0, true, 1, false),
                (1, true, 2, true),
                (0, false, 2, false),
            ];
            for &(i, f, j, g) in &clauses {
                sut.add_clause(i, f, j, g);
            }
            // When
            let result = sut.solve();
            // Then
            let assignment = result.unwrap();
            for (i, f, j, g) in clauses {
                assert!(assignment[i] == f || assignment[j] == g);
            }
        }
    }
}

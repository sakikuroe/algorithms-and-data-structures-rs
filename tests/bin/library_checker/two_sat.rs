use super::super::common;

/// `lc-two-sat` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-two-sat");

// lc-two-sat のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_two_sat {
    use super::*;

    /// Scenario: 充足可能なインスタンスに対して、すべての節を満たす割り当てが
    /// 出力される
    /// - Given: 問題文の公式サンプル (充足可能なインスタンス) である
    /// - When: lc-two-sat バイナリへ標準入力として渡す
    /// - Then: `SATISFIABLE` と、すべての節を満たす割り当てが出力される
    #[test]
    fn satisfies_all_clauses_for_satisfiable_sample() {
        // Given
        let input = "p cnf 5 6\n1 2 0\n-3 -1 0\n-4 -3 0\n2 -5 0\n5 -2 0\n1 4 0\n";
        let clauses = [(1, 2), (-3, -1), (-4, -3), (2, -5), (5, -2), (1, 4)];
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();
        assert_eq!(Some("s"), tokens.next());
        assert_eq!(Some("SATISFIABLE"), tokens.next());
        assert_eq!(Some("v"), tokens.next());
        let value = (1..=5)
            .map(|_| tokens.next().unwrap().parse::<i32>().unwrap())
            .collect::<Vec<i32>>();
        assert_eq!(Some("0"), tokens.next());
        assert!(tokens.next().is_none());

        let satisfies = |literal: i32| value[(literal.unsigned_abs() as usize) - 1] == literal;
        for (a, b) in clauses {
            assert!(satisfies(a) || satisfies(b));
        }
    }

    /// Scenario: 充足不能なインスタンスに対しては `UNSATISFIABLE` が出力される
    /// - Given: 問題文の公式サンプル (充足不能なインスタンス) である
    /// - When: lc-two-sat バイナリへ標準入力として渡す
    /// - Then: `UNSATISFIABLE` が出力される
    #[test]
    fn reports_unsatisfiable_for_contradictory_sample() {
        // Given
        let input = "p cnf 2 4\n1 2 0\n1 -2 0\n-1 2 0\n-1 -2 0\n";
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();
        assert_eq!(Some("s"), tokens.next());
        assert_eq!(Some("UNSATISFIABLE"), tokens.next());
        assert!(tokens.next().is_none());
    }
}

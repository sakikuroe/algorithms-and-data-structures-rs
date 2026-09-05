use rstest::rstest;

use super::super::common;

/// `spoj-qtree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_spoj-qtree");

// spoj-qtree のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod spoj_qtree {
    use super::*;

    /// Scenario: 与えられた入力を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル、および手計算で検証した小規模なケースである
    /// - When: spoj-qtree バイナリへ標準入力として渡す
    /// - Then: パス上の辺コストの最大値が期待値と一致する
    #[rstest]
    #[case::official_sample(
        "1\n\n3\n1 2 1\n2 3 2\nQUERY 1 2\nCHANGE 1 3\nQUERY 1 2\nDONE\n",
        "1\n3\n"
    )]
    #[case::multiple_test_cases(
        "2\n\n3\n1 2 1\n2 3 2\nQUERY 1 2\nCHANGE 1 3\nQUERY 1 2\nDONE\n\n4\n1 2 5\n2 3 3\n2 4 7\nQUERY 1 3\nQUERY 3 4\nQUERY 1 4\nDONE\n",
        "1\n3\n5\n7\n7\n"
    )]
    #[case::single_node_has_no_queries("1\n\n1\nDONE\n", "")]
    fn matches_expected_output(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

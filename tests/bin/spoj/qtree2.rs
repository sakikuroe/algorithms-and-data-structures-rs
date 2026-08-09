use rstest::rstest;

use super::super::common;

/// `spoj-qtree2` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_spoj-qtree2");

// spoj-qtree2 のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod spoj_qtree2 {
    use super::*;

    /// Scenario: 与えられた入力を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル、および手計算で検証した小規模なケースである
    /// - When: spoj-qtree2 バイナリへ標準入力として渡す
    /// - Then: パス上の辺コストの総和 (DIST)、およびパス上の k 番目の頂点
    ///   (KTH) が期待値と一致し、各テストケースの後に空行が1行出力される
    #[rstest]
    #[case::official_sample(
        "1\n\n6\n1 2 1\n2 4 1\n2 5 2\n1 3 1\n3 6 2\nDIST 4 6\nKTH 4 6 4\nDONE\n",
        "5\n3\n\n"
    )]
    #[case::kth_at_both_endpoints_of_a_two_node_tree(
        "1\n\n2\n1 2 100\nDIST 1 2\nKTH 1 2 1\nKTH 1 2 2\nDONE\n",
        "100\n1\n2\n\n"
    )]
    fn matches_expected_output(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

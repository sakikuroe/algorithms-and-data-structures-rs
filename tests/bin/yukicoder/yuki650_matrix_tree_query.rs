use rstest::rstest;

use super::super::common;

/// `yuki-yuki650-matrix-tree-query` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_yuki-yuki650-matrix-tree-query");

// yuki-yuki650-matrix-tree-query のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod yuki_yuki650_matrix_tree_query {
    use super::*;

    /// Scenario: 与えられた入力を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル、および手計算で検証した小規模なケースである
    /// - When: yuki-yuki650-matrix-tree-query バイナリへ標準入力として渡す
    /// - Then: パス上の行列を根側から葉側の順に掛け合わせた結果が期待値と一致する
    #[rstest]
    #[case::official_sample(
        "6\n0 1\n1 3\n0 2\n2 4\n2 5\n8\nx 2 1 3 0 2\nx 3 6 2 8 7\ng 0 2\ng 0 4\ng 0 5\ng 2 5\nx 4 7 2 4 9\ng 0 5\n",
        "1 3 0 2\n30 23 16 14\n1 3 0 2\n1 0 0 1\n19 29 8 18\n"
    )]
    #[case::identity_before_any_update(
        "2\n0 1\n3\ng 0 1\nx 0 2 3 4 5\ng 0 1\n",
        "1 0 0 1\n2 3 4 5\n"
    )]
    #[case::order_matters_for_noncommutative_matrices(
        "3\n0 1\n1 2\n3\nx 0 1 1 0 1\nx 1 1 0 1 1\ng 0 2\n",
        "2 1 1 1\n"
    )]
    fn matches_expected_output(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

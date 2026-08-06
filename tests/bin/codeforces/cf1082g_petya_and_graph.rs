use rstest::rstest;

use super::super::common;

/// `cf-cf1082g-petya-and-graph` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_cf-cf1082g-petya-and-graph");

// cf-cf1082g-petya-and-graph のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod cf_cf1082g_petya_and_graph {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: cf-cf1082g-petya-and-graph バイナリへ標準入力として渡す
    /// - Then: 部分グラフの最大重みが期待値と一致する
    #[rstest]
    #[case::sample_1("4 5\n1 5 2 2\n1 3 4\n1 4 4\n3 4 5\n3 2 2\n4 2 2\n", "8\n")]
    #[case::sample_2_empty_is_best("3 3\n9 7 8\n1 2 1\n2 3 2\n1 3 3\n", "0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

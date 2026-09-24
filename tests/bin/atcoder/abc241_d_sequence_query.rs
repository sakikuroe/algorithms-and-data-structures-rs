use rstest::rstest;

use super::super::common;

/// `ac-abc241-d-sequence-query` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc241-d-sequence-query");

// ac-abc241-d-sequence-query のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc241_d_sequence_query {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する。
    /// - Given: 問題文の公式サンプルである。
    /// - When: ac-abc241-d-sequence-query バイナリへ標準入力として渡す。
    /// - Then: 各順位クエリの結果が期待値と一致する。
    #[rstest]
    #[case::sample_1(
        "11\n1 20\n1 10\n1 30\n1 20\n3 15 1\n3 15 2\n3 15 3\n3 15 4\n2 100 5\n1 1\n2 100 5\n",
        "20\n20\n30\n-1\n-1\n1\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

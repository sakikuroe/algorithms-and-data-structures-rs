use rstest::rstest;

use super::super::common;

/// `ac-abc225-g-x` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc225-g-x");

// ac-abc225-g-x のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc225_g_x {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc225-g-x バイナリへ標準入力として渡す
    /// - Then: スコアの最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("2 2 2\n2 10\n8 3\n", "12\n")]
    #[case::sample_2_none_is_best("3 3 100\n1 1 1\n1 1 1\n1 1 1\n", "0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

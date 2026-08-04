use rstest::rstest;

use super::super::common;

/// `lc-log-of-formal-power-series` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-log-of-formal-power-series");

// lc-log-of-formal-power-series のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_log_of_formal_power_series {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-log-of-formal-power-series バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: 1 + x を表す係数列 [1, 1, 0, 0, 0, 0, 0, 0] がある
    #[case::dense_case(
        "8\n1 1 0 0 0 0 0 0\n",
        "0\n1\n499122176\n332748118\n249561088\n598946612\n831870294\n855638017\n"
    )]
    // - Given: 項数 1 の係数列 [1] がある
    #[case::boundary_length_one("1\n1\n", "0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

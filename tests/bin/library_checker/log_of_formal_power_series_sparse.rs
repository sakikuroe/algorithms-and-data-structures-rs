use rstest::rstest;

use super::super::common;

/// `lc-log-of-formal-power-series-sparse` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-log-of-formal-power-series-sparse");

// lc-log-of-formal-power-series-sparse のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_log_of_formal_power_series_sparse {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-log-of-formal-power-series-sparse バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: log_of_formal_power_series_sparse のサンプル 1 の入力である
    #[case::sample_1("5 2\n0 1\n2 1\n", "0\n0\n1\n0\n499122176\n")]
    // - Given: log_of_formal_power_series_sparse のサンプル 2 の入力である
    #[case::sample_2(
        "10 5\n0 1\n1 1\n2 499122179\n3 166374064\n4 291154613\n",
        "0\n1\n2\n3\n4\n307791995\n131712787\n793247753\n831003798\n590204334\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

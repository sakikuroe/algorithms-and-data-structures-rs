use rstest::rstest;

use super::super::common;

/// `lc-exp-of-formal-power-series` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-exp-of-formal-power-series");

// lc-exp-of-formal-power-series のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_exp_of_formal_power_series {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-exp-of-formal-power-series バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: 定数項が 0 の係数列 [0, 1, 2, 3, 4, 5] がある
    #[case::dense_case(
        "6\n0 1 2 3 4 5\n",
        "1\n1\n499122179\n166374064\n291154613\n690452363\n"
    )]
    // - Given: 項数 1 の係数列 [0] がある
    #[case::boundary_length_one("1\n0\n", "1\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

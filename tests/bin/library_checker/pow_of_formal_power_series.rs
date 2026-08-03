use rstest::rstest;

use super::super::common;

/// `lc-pow-of-formal-power-series` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-pow-of-formal-power-series");

// lc-pow-of-formal-power-series のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_pow_of_formal_power_series {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-pow-of-formal-power-series バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: 定数項が非ゼロの係数列 [1, 2, 3, 0, 0, 0] と指数 4 がある
    #[case::dense_case_with_non_zero_constant("6 4\n1 2 3 0 0 0\n", "1\n8\n36\n104\n214\n312\n")]
    // - Given: x + x^2 を表す係数列 [0, 1, 1, 0, 0, 0, 0, 0] と指数 3 がある
    #[case::sparse_case_with_leading_zero_terms(
        "8 3\n0 1 1 0 0 0 0 0\n",
        "0\n0\n0\n1\n3\n3\n1\n0\n"
    )]
    // - Given: 1 + x を表す係数列 [1, 1, 0] と指数 1000000000 がある
    #[case::huge_exponent_close_to_ten_to_the_ninth(
        "3 1000000000\n1 1 0\n",
        "1\n1755647\n856279802\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

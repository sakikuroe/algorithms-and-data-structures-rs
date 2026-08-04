use rstest::rstest;

use super::super::common;

/// `lc-exp-of-formal-power-series-sparse` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-exp-of-formal-power-series-sparse");

// lc-exp-of-formal-power-series-sparse のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_exp_of_formal_power_series_sparse {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-exp-of-formal-power-series-sparse バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: exp_of_formal_power_series_sparse のサンプル 1 の入力である
    #[case::sample_1("5 1\n2 1\n", "1\n0\n1\n0\n499122177\n")]
    // - Given: exp_of_formal_power_series_sparse のサンプル 2 の入力である
    #[case::sample_2(
        "10 4\n1 1\n2 2\n3 3\n4 4\n",
        "1\n1\n499122179\n166374064\n291154613\n690452358\n558739571\n801170355\n116437135\n935147171\n"
    )]
    // - Given: exp_of_formal_power_series_sparse のサンプル 3 の入力である (ゼロ多項式)
    #[case::sample_3_zero_polynomial("10 0\n", "1\n0\n0\n0\n0\n0\n0\n0\n0\n0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

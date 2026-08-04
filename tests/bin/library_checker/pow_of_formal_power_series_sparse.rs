use rstest::rstest;

use super::super::common;

/// `lc-pow-of-formal-power-series-sparse` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-pow-of-formal-power-series-sparse");

// lc-pow-of-formal-power-series-sparse のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_pow_of_formal_power_series_sparse {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-pow-of-formal-power-series-sparse バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: pow_of_formal_power_series_sparse のサンプル 1 の入力である
    #[case::sample_1("5 2 3\n0 1\n2 1\n", "1\n0\n3\n0\n3\n")]
    // - Given: pow_of_formal_power_series_sparse のサンプル 2 の入力である
    //   (最小次数の項が正の次数にあり、次数シフトが発生する)
    #[case::sample_2("5 2 3\n1 1\n2 1\n", "0\n0\n0\n1\n3\n")]
    // - Given: pow_of_formal_power_series_sparse のサンプル 3 の入力である (0^10)
    #[case::sample_3_zero_polynomial_positive_exponent("5 0 10\n", "0\n0\n0\n0\n0\n")]
    // - Given: pow_of_formal_power_series_sparse のサンプル 4 の入力である (0^0)
    #[case::sample_4_zero_polynomial_zero_exponent("5 0 0\n", "1\n0\n0\n0\n0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `lc-inv-of-formal-power-series-sparse` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-inv-of-formal-power-series-sparse");

// lc-inv-of-formal-power-series-sparse のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_inv_of_formal_power_series_sparse {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-inv-of-formal-power-series-sparse バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: inv_of_formal_power_series_sparse のサンプル 1 の入力である
    #[case::sample_1("5 2\n0 1\n2 1\n", "1\n0\n998244352\n0\n1\n")]
    // - Given: inv_of_formal_power_series_sparse のサンプル 2 の入力である
    #[case::sample_2(
        "10 5\n0 5\n1 4\n2 3\n3 2\n4 1\n",
        "598946612\n718735934\n862483121\n635682004\n163871793\n995241634\n275905156\n386987871\n291888821\n422779055\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

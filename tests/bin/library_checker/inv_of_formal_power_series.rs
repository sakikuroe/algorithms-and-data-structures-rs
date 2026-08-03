use rstest::rstest;

use super::super::common;

/// `lc-inv-of-formal-power-series` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-inv-of-formal-power-series");

// lc-inv-of-formal-power-series のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_inv_of_formal_power_series {
    use super::*;

    /// Scenario: 形式的べき級数の逆元が、法 998244353 上で独立に計算した値と一致する
    /// - Given: 各ケースの係数列がある
    /// - When: lc-inv-of-formal-power-series バイナリへ標準入力として渡す
    /// - Then: 独立に計算した逆元の係数と一致する
    #[rstest]
    // 定数項が非ゼロの密な係数列 [5, 4, 3, 2, 1] に対する逆元
    #[case::dense_case(
        "5\n5 4 3 2 1\n",
        "598946612\n718735934\n862483121\n635682004\n163871793\n"
    )]
    // 境界: 項数 1 の係数列 [3] は、3 の法 998244353 上の逆数のみからなる
    #[case::boundary_length_one("1\n3\n", "332748118\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

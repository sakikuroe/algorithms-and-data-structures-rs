use rstest::rstest;

use super::super::common;

/// `lc-convolution-mod` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-convolution-mod");

// lc-convolution-mod のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_convolution_mod {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-convolution-mod バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: convolution_mod のサンプル 1 の入力である
    #[case::sample_1("4 5\n1 2 3 4\n5 6 7 8 9\n", "5\n16\n34\n60\n70\n70\n59\n36\n")]
    // - Given: convolution_mod のサンプル 2 の入力である (MOD に近い値の掛け合わせ)
    #[case::sample_2("1 1\n10000000\n10000000\n", "871938225\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

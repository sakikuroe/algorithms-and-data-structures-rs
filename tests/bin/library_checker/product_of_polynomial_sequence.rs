use rstest::rstest;

use super::super::common;

/// `lc-product-of-polynomial-sequence` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-product-of-polynomial-sequence");

// lc-product-of-polynomial-sequence のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_product_of_polynomial_sequence {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-product-of-polynomial-sequence バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: product_of_polynomial_sequence のサンプル 1 の入力である
    //   (2(1+2x)(3+2x+x^2) を求める)
    #[case::sample_1("3\n0 2\n1 1 2\n2 3 2 1\n", "6\n16\n10\n4\n")]
    // - Given: product_of_polynomial_sequence のサンプル 2 の入力である (多項式 1 個)
    #[case::sample_2_single_polynomial("1\n2 3 4 5\n", "3\n4\n5\n")]
    // - Given: product_of_polynomial_sequence のサンプル 3 の入力である (多項式 0 個)
    #[case::sample_3_boundary_n_is_zero("0\n", "1\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

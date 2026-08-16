use rstest::rstest;

use super::super::common;

/// `lc-enumerate-primes` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-enumerate-primes");

// lc-enumerate-primes のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_enumerate_primes {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: `N = 100`, `A = 3`, `B = 1` である
    /// - When: lc-enumerate-primes バイナリへ標準入力として渡す
    /// - Then: `100` 以下の素数の個数と、間引いた素数列が期待通りである
    #[rstest]
    #[case::sample_1("100 3 1\n", "25 8\n3 11 19 31 43 59 71 83\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

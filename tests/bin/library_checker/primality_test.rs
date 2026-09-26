use rstest::rstest;

use super::super::common;

/// `lc-primality-test` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-primality-test");

// lc-primality-test のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_primality_test {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 素数と合成数、および `1`・`u64` に近い大きな値を含む問い合わせがある
    /// - When: lc-primality-test バイナリへ標準入力として渡す
    /// - Then: 各問い合わせに対する素数判定結果が期待通りである
    #[rstest]
    #[case::sample_1(
        "6\n1\n2\n3\n4\n998244353\n1000000000000000000\n",
        "No\nYes\nYes\nNo\nYes\nNo\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

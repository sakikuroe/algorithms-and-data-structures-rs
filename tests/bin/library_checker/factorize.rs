use rstest::rstest;

use super::super::common;

/// `lc-factorize` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-factorize");

// lc-factorize のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_factorize {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: `1` から `10` までの整数の素因数分解を問い合わせる
    /// - When: lc-factorize バイナリへ標準入力として渡す
    /// - Then: 各整数の素因数を昇順に並べた分解結果が期待通りである
    #[rstest]
    #[case::sample_1(
        "10\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n",
        "0\n1 2\n1 3\n2 2 2\n1 5\n2 2 3\n1 7\n3 2 2 2\n2 3 3\n2 2 5\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

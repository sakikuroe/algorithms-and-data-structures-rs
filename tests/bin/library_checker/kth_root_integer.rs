use rstest::rstest;

use super::super::common;

/// `lc-kth-root-integer` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-kth-root-integer");

// lc-kth-root-integer のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_kth_root_integer {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 完全累乗数・非累乗数・`u64::MAX` 付近の値など、10 個の問い合わせがある
    /// - When: lc-kth-root-integer バイナリへ標準入力として渡す
    /// - Then: 各問い合わせに対する `k` 乗根の整数部分が期待通りである
    #[rstest]
    #[case::sample_1(
        "10\n215 3\n216 3\n217 3\n9999999999 10\n10000000000 10\n10000000001 10\n18446744073709551615 1\n18446744073709551615 2\n18446744073709551615 63\n18446744073709551615 64\n",
        "5\n6\n6\n9\n10\n10\n18446744073709551615\n4294967295\n2\n1\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

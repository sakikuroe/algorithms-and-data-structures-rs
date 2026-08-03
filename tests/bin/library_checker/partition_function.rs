use rstest::rstest;

use super::super::common;

/// `lc-partition-function` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-partition-function");

// lc-partition-function のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_partition_function {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-partition-function バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: partition_function のサンプルの入力である (N = 10)
    #[case::sample_1("10\n", "1\n1\n2\n3\n5\n7\n11\n15\n22\n30\n42\n")]
    // - Given: N = 0 の入力である
    #[case::boundary_n_is_zero("0\n", "1\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

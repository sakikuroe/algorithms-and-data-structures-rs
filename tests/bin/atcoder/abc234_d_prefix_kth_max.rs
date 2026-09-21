use rstest::rstest;

use super::super::common;

/// `ac-abc234-d-prefix-kth-max` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc234-d-prefix-kth-max");

// ac-abc234-d-prefix-kth-max のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc234_d_prefix_kth_max {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc234-d-prefix-kth-max バイナリへ標準入力として渡す
    /// - Then: 各 prefix の k-th max が期待値と一致する
    #[rstest]
    #[case::sample_1("3 2\n1 2 3\n", "1\n2\n")]
    #[case::sample_2("11 5\n3 7 2 5 11 6 1 9 8 10 4\n", "2\n3\n3\n5\n6\n7\n7\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

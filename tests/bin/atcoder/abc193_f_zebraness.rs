use rstest::rstest;

use super::super::common;

/// `ac-abc193-f-zebraness` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc193-f-zebraness");

// ac-abc193-f-zebraness のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc193_f_zebraness {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc193-f-zebraness バイナリへ標準入力として渡す
    /// - Then: しまうま度の最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("2\nBB\nBW\n", "2\n")]
    #[case::sample_2("3\nBBB\nBBB\nW?W\n", "4\n")]
    #[case::sample_3_all_undetermined("5\n?????\n?????\n?????\n?????\n?????\n", "40\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

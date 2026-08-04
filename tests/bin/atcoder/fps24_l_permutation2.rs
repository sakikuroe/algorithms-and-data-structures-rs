use rstest::rstest;

use super::super::common;

/// `ac-fps24-l-permutation2` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-l-permutation2");

// ac-fps24-l-permutation2 のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_l_permutation2 {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-l-permutation2 バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_l のサンプル 1 の入力である
    #[case::sample_1("3\n", "2\n")]
    // - Given: fps_24_l のサンプル 2 の入力である (N が大きいケース)
    #[case::sample_2("123456\n", "916370671\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

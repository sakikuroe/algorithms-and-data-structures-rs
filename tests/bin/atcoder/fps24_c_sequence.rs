use rstest::rstest;

use super::super::common;

/// `ac-fps24-c-sequence` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-c-sequence");

// ac-fps24-c-sequence のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_c_sequence {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-c-sequence バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_c のサンプル 1 の入力である
    #[case::sample_1("3 2 4\n", "6\n")]
    // - Given: fps_24_c のサンプル 2 の入力である (N, M, S が大きいケース)
    #[case::sample_2("12345 678 90123\n", "226012779\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

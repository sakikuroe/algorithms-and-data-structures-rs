use rstest::rstest;

use super::super::common;

/// `ac-fps24-d-sequence2` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-d-sequence2");

// ac-fps24-d-sequence2 のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_d_sequence2 {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-d-sequence2 バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_d のサンプル 1 の入力である
    #[case::sample_1("2 3\n", "8\n")]
    // - Given: fps_24_d のサンプル 2 の入力である (N, M が大きいケース)
    #[case::sample_2("12345 67890\n", "761484871\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-fps24-i-score` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-i-score");

// ac-fps24-i-score のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_i_score {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-i-score バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_i のサンプル 1 の入力である
    #[case::sample_1("3 2\n2 3 5\n", "31\n")]
    // - Given: fps_24_i のサンプル 2 の入力である
    #[case::sample_2("10 5\n1 2 3 4 5 6 7 8 9 10\n", "902055\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

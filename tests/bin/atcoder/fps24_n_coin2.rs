use rstest::rstest;

use super::super::common;

/// `ac-fps24-n-coin2` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-n-coin2");

// ac-fps24-n-coin2 のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_n_coin2 {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-n-coin2 バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_n のサンプル 1 の入力である
    #[case::sample_1("3\n1 2 3\n", "2\n")]
    // - Given: fps_24_n のサンプル 2 の入力である
    #[case::sample_2("10\n3 1 4 1 5 9 2 6 5 3\n", "20\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

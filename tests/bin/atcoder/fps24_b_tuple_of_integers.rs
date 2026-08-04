use rstest::rstest;

use super::super::common;

/// `ac-fps24-b-tuple-of-integers` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-b-tuple-of-integers");

// ac-fps24-b-tuple-of-integers のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_b_tuple_of_integers {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-b-tuple-of-integers バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_b のサンプル 1 の入力である
    #[case::sample_1("5\n", "6\n")]
    // - Given: fps_24_b のサンプル 2 の入力である (N が 10^9 に達するケース)
    #[case::sample_2("1000000000\n", "1755648\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-abc422-g-balls-and-boxes` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc422-g-balls-and-boxes");

// ac-abc422-g-balls-and-boxes のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc422_g_balls_and_boxes {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc422-g-balls-and-boxes バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: ABC422 G のサンプル 1 の入力である
    #[case::sample_1("3 1 2 3\n", "3\n5\n")]
    // - Given: ABC422 G のサンプル 2 の入力である (N, A, B, C が大きいケース)
    #[case::sample_2("1234 56 7 89\n", "15\n535248725\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

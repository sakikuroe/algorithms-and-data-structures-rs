use rstest::rstest;

use super::super::common;

/// `ac-fps24-e-sequence3` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-e-sequence3");

// ac-fps24-e-sequence3 のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_e_sequence3 {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-e-sequence3 バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_e のサンプル 1 の入力である
    #[case::sample_1("2 3\n", "8\n")]
    // - Given: fps_24_e のサンプル 2 の入力である (N, M が制約上限のケース)
    #[case::sample_2("300 300\n", "478329414\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

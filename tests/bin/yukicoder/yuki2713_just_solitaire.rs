use rstest::rstest;

use super::super::common;

/// `yuki-yuki2713-just-solitaire` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_yuki-yuki2713-just-solitaire");

// yuki-yuki2713-just-solitaire のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod yuki_yuki2713_just_solitaire {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: yuki-yuki2713-just-solitaire バイナリへ標準入力として渡す
    /// - Then: 得ることのできる利益の最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("5 2\n10 20 30 40 50\n50 120\n3 1 2 3\n2 4 5\n", "30\n")]
    #[case::sample_2_uses_all_cards("5 2\n10 20 30 40 50\n50 120\n3 1 2 3\n5 1 2 3 4 5\n", "20\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

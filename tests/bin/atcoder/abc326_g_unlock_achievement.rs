use rstest::rstest;

use super::super::common;

/// `ac-abc326-g-unlock-achievement` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc326-g-unlock-achievement");

// ac-abc326-g-unlock-achievement のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc326_g_unlock_achievement {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc326-g-unlock-achievement バイナリへ標準入力として渡す
    /// - Then: 得られる金額の最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("2 2\n10 20\n100 50\n3 1\n1 4\n", "80\n")]
    #[case::sample_2("2 2\n10 20\n100 50\n3 2\n1 4\n", "70\n")]
    #[case::sample_3(
        "10 10\n10922 23173 32300 22555 29525 16786 3135 17046 11245 20310\n177874 168698 202247 31339 10336 14825 56835 6497 12440 110702\n2 1 4 1 3 4 4 5 1 4\n2 3 4 4 5 3 5 5 2 3\n2 3 5 1 4 2 2 2 2 5\n3 5 5 3 5 2 2 1 5 4\n3 1 1 4 4 1 1 5 3 1\n1 2 3 2 4 2 4 3 3 1\n4 4 4 2 5 1 4 2 2 2\n5 3 1 2 3 4 2 5 2 2\n5 4 3 4 3 1 5 1 5 4\n2 3 2 5 2 3 1 2 2 4\n",
        "66900\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

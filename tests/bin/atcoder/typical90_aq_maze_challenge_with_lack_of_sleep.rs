use rstest::rstest;

use super::super::common;

/// `ac-typical90-aq-maze-challenge-with-lack-of-sleep` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-aq-maze-challenge-with-lack-of-sleep");

// ac-typical90-aq-maze-challenge-with-lack-of-sleep のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_aq_maze_challenge_with_lack_of_sleep {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-typical90-aq-maze-challenge-with-lack-of-sleep バイナリへ標準入力として渡す
    /// - Then: 方向転換回数の最小値が期待値と一致する
    #[rstest]
    #[case::sample_1("3 3\n1 1\n3 3\n..#\n#.#\n#..\n", "2\n")]
    #[case::sample_2_zero_turns("3 3\n2 1\n2 3\n#.#\n...\n#.#\n", "0\n")]
    #[case::sample_3("4 6\n2 1\n1 5\n...#..\n.#.##.\n.#....\n...##.\n", "5\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

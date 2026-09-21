use rstest::rstest;

use super::super::common;

/// `ac-abc405-f-chord-crossing` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc405-f-chord-crossing");

// ac-abc405-f-chord-crossing のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc405_f_chord_crossing {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc405-f-chord-crossing バイナリへ標準入力として渡す
    /// - Then: 各クエリの交差数が期待値と一致する
    #[rstest]
    #[case::sample_1("4 2\n2 4\n6 8\n3\n1 3\n3 7\n1 5\n", "1\n2\n0\n")]
    #[case::sample_2(
        "20 7\n24 34\n26 28\n18 38\n2 14\n8 12\n30 32\n20 22\n10\n7 29\n31 39\n9 21\n19 29\n15 21\n11 39\n17 21\n15 31\n5 25\n25 31\n",
        "3\n3\n4\n1\n2\n2\n2\n3\n3\n1\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-abc209-d-collision` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc209-d-collision");

// ac-abc209-d-collision のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc209_d_collision {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc209-d-collision バイナリへ標準入力として渡す
    /// - Then: 各クエリの出会う場所 (Town/Road) が期待値と一致する
    #[rstest]
    // - Given: 出会う場所が道の途中 (Road) になるクエリである
    #[case::sample_1("4 1\n1 2\n2 3\n2 4\n1 2\n", "Road\n")]
    // - Given: 出会う場所がともに町 (Town) になる2クエリである
    #[case::sample_2("5 2\n1 2\n2 3\n3 4\n4 5\n1 3\n1 5\n", "Town\nTown\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

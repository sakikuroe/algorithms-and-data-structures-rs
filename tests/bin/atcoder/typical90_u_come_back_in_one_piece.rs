use rstest::rstest;

use super::super::common;

/// `ac-typical90-u-come-back-in-one-piece` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-u-come-back-in-one-piece");

// ac-typical90-u-come-back-in-one-piece のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_u_come_back_in_one_piece {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-typical90-u-come-back-in-one-piece バイナリへ標準入力として渡す
    /// - Then: 条件を満たす頂点対の個数が期待値と一致する
    #[rstest]
    #[case::sample_1("4 7\n1 2\n2 1\n2 3\n4 3\n4 1\n1 4\n2 3\n", "3\n")]
    #[case::sample_2_no_cycle("100 1\n1 2\n", "0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-typical90-an-get-more-money` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-an-get-more-money");

// ac-typical90-an-get-more-money のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_an_get_more_money {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-typical90-an-get-more-money バイナリへ標準入力として渡す
    /// - Then: 最終的に得る金額の最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("5 5\n5 2 10 3 6\n1 3\n1 3\n0\n1 5\n0\n", "2\n")]
    #[case::sample_2_none_is_best("6 10\n8 6 9 1 2 0\n1 3\n2 3 4\n1 5\n1 5\n1 6\n0\n", "0\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

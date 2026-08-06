use rstest::rstest;

use super::super::common;

/// `ac-arc085-e-mul` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-arc085-e-mul");

// ac-arc085-e-mul のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_arc085_e_mul {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-arc085-e-mul バイナリへ標準入力として渡す
    /// - Then: 得られるお金の最大値が期待値と一致する
    #[rstest]
    #[case::sample_1("6\n1 2 -6 4 5 3\n", "12\n")]
    #[case::sample_2("6\n100 -100 -100 -100 100 -100\n", "200\n")]
    #[case::sample_3_all_negative("5\n-1 -2 -3 -4 -5\n", "0\n")]
    #[case::sample_4("2\n-1000 100000\n", "99000\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-abc177-e-coprime` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc177-e-coprime");

// ac-abc177-e-coprime のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc177_e_coprime {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc177-e-coprime バイナリへ標準入力として渡す
    /// - Then: pairwise/setwise/not coprime の判定結果が期待値と一致する
    #[rstest]
    // - Given: どの2つを取っても互いに素である
    #[case::sample_1("3\n3 4 5\n", "pairwise coprime\n")]
    // - Given: 全体の gcd は 1 だが、2つずつ見ると素因数を共有する組がある
    #[case::sample_2("3\n6 10 15\n", "setwise coprime\n")]
    // - Given: 全体の gcd が 1 でない
    #[case::sample_3("3\n6 10 16\n", "not coprime\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `lc-cycle-detection` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-cycle-detection");

// lc-cycle-detection のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_cycle_detection {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-cycle-detection バイナリへ標準入力として渡す
    /// - Then: 見つけた閉路 (または閉路なし) が期待通りである
    #[rstest]
    // - Given: 閉路 0->4->2->1->0 を含むグラフである
    #[case::sample_1("5 7\n0 3\n0 4\n4 2\n4 3\n4 0\n2 1\n1 0\n", "4\n1\n2\n5\n6\n")]
    // - Given: 閉路を含まない単純パスのみのグラフである
    #[case::sample_2_no_cycle("2 1\n1 0\n", "-1\n")]
    // - Given: 複数の閉路を含むグラフである
    #[case::sample_3_multiple_cycles("4 6\n0 1\n1 2\n2 0\n0 1\n1 3\n3 0\n", "3\n0\n1\n2\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

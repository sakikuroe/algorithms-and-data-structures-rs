use rstest::rstest;

use super::super::common;

/// `lc-kth-term-of-linearly-recurrent-sequence` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-kth-term-of-linearly-recurrent-sequence");

// lc-kth-term-of-linearly-recurrent-sequence のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_kth_term_of_linearly_recurrent_sequence {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-kth-term-of-linearly-recurrent-sequence バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: kth_term_of_linearly_recurrent_sequence のサンプル 1 の入力である
    #[case::sample_1_fibonacci("2 5\n1 1\n1 1\n", "8\n")]
    // - Given: 3 項の初期値を持つ数列と、初期項の個数未満の項番号 1 がある
    #[case::boundary_k_smaller_than_degree("3 1\n7 11 13\n2 3 5\n", "11\n")]
    // - Given: 5 項の初期値と係数を持つ線形漸化式と、項番号 40 がある
    #[case::larger_k_matches_naive_recurrence("5 40\n1 2 3 4 5\n6 7 8 9 10\n", "824364119\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

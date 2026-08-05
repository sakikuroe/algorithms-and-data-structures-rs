use rstest::rstest;

use super::super::common;

/// `aoj-alds1-11-b-depth-first-search` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-alds1-11-b-depth-first-search");

// aoj-alds1-11-b-depth-first-search のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod aoj_alds1_11_b_depth_first_search {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 自己ループを含む6頂点の有向グラフである
    /// - When: aoj-alds1-11-b-depth-first-search バイナリへ標準入力として渡す
    /// - Then: 各頂点の発見時刻・完了時刻が期待通りである
    #[rstest]
    #[case::sample_1(
        "6\n1 2 2 4\n2 1 5\n3 2 5 6\n4 0\n5 1 4\n6 1 6\n",
        "1 1 8\n2 2 7\n3 9 12\n4 4 5\n5 3 6\n6 10 11\n"
    )]
    fn matches_official_sample(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `aoj-alds1-11-c-breadth-first-search` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-alds1-11-c-breadth-first-search");

// aoj-alds1-11-c-breadth-first-search のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod aoj_alds1_11_c_breadth_first_search {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 自己ループを含む6頂点の有向グラフである
    /// - When: aoj-alds1-11-c-breadth-first-search バイナリへ標準入力として渡す
    /// - Then: 頂点1からの各頂点への距離が期待通りである
    #[rstest]
    #[case::sample_1(
        "6\n1 2 2 4\n2 1 5\n3 2 5 6\n4 0\n5 1 4\n6 1 6\n",
        "1 0\n2 1\n3 -1\n4 1\n5 2\n6 -1\n"
    )]
    fn matches_official_sample(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `aoj-grl-1-c-all-pairs-shortest-path-johnson` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-grl-1-c-all-pairs-shortest-path-johnson");

// aoj-grl-1-c-all-pairs-shortest-path-johnson のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod aoj_grl_1_c_all_pairs_shortest_path_johnson {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: aoj-grl-1-c-all-pairs-shortest-path-johnson バイナリへ標準入力として渡す
    /// - Then: 全頂点対の最短コスト (または `NEGATIVE CYCLE`) が期待通りである
    #[rstest]
    // - Given: すべての辺の重みが非負であるグラフである
    #[case::sample_1(
        "4 6\n0 1 1\n0 2 5\n1 2 2\n1 3 4\n2 3 1\n3 2 7\n",
        "0 1 3 4\nINF 0 2 3\nINF INF 0 1\nINF INF 7 0\n"
    )]
    // - Given: 負の重みを持つ辺を含むが、負閉路は存在しないグラフである
    #[case::sample_2_negative_edge(
        "4 6\n0 1 1\n0 2 -5\n1 2 2\n1 3 4\n2 3 1\n3 2 7\n",
        "0 1 -5 -4\nINF 0 2 3\nINF INF 0 1\nINF INF 7 0\n"
    )]
    // - Given: 負閉路を含むグラフである
    #[case::sample_3_negative_cycle(
        "4 6\n0 1 1\n0 2 5\n1 2 2\n1 3 4\n2 3 1\n3 2 -7\n",
        "NEGATIVE CYCLE\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

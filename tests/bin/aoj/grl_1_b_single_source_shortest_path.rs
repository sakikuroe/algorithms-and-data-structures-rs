use rstest::rstest;

use super::super::common;

/// `aoj-grl-1-b-single-source-shortest-path` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-grl-1-b-single-source-shortest-path");

// aoj-grl-1-b-single-source-shortest-path のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod aoj_grl_1_b_single_source_shortest_path {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: aoj-grl-1-b-single-source-shortest-path バイナリへ標準入力として渡す
    /// - Then: 各頂点への最短コスト (または `NEGATIVE CYCLE`) が期待通りである
    #[rstest]
    // - Given: 負の重みを持つ辺を含むが、負閉路は存在しないグラフである
    #[case::sample_1(
        "4 5 0\n0 1 2\n0 2 3\n1 2 -5\n1 3 1\n2 3 2\n",
        "0\n2\n-3\n-1\n"
    )]
    // - Given: 始点 0 から到達可能な負閉路を含むグラフである
    #[case::sample_2_negative_cycle(
        "4 6 0\n0 1 2\n0 2 3\n1 2 -5\n1 3 1\n2 3 2\n3 1 0\n",
        "NEGATIVE CYCLE\n"
    )]
    // - Given: 同じグラフだが、始点を1に変えると頂点0が到達不能になる
    #[case::sample_3_unreachable_start(
        "4 5 1\n0 1 2\n0 2 3\n1 2 -5\n1 3 1\n2 3 2\n",
        "INF\n0\n-5\n-3\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

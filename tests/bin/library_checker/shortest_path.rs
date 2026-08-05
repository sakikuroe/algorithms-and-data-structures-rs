use rstest::rstest;

use super::super::common;

/// `lc-shortest-path` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-shortest-path");

// lc-shortest-path のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_shortest_path {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: lc-shortest-path バイナリへ標準入力として渡す
    /// - Then: 最短距離・経路 (または到達不能) が期待通りである
    #[rstest]
    // - Given: 頂点 2 から頂点 3 への最短路が 2->1->0->3 (距離11) であるグラフである
    #[case::sample_1(
        "5 7 2 3\n0 3 5\n0 4 3\n2 4 2\n4 3 10\n4 0 7\n2 1 5\n1 0 1\n",
        "11\n3\n2\n1\n1\n0\n0\n3\n"
    )]
    // - Given: 始点から終点への辺が逆向きにしか無く、到達不能なグラフである
    #[case::sample_2_unreachable("2 1 0 1\n1 0 10\n", "-1\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

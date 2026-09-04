use rstest::rstest;

use super::super::common;

/// `ac-abc294-g-distance-queries-on-a-tree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc294-g-distance-queries-on-a-tree");

// ac-abc294-g-distance-queries-on-a-tree のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc294_g_distance_queries_on_a_tree {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc294-g-distance-queries-on-a-tree バイナリへ標準入力として渡す
    /// - Then: 辺の重み変更を反映した、2頂点間の距離が期待値と一致する
    #[rstest]
    #[case::sample_1(
        "5\n1 2 3\n1 3 6\n1 4 9\n4 5 10\n4\n2 2 3\n2 1 5\n1 3 1\n2 1 5\n",
        "9\n19\n11\n"
    )]
    #[case::sample_2_answer_exceeds_32bit(
        "7\n1 2 1000000000\n2 3 1000000000\n3 4 1000000000\n4 5 1000000000\n5 6 1000000000\n6 7 1000000000\n3\n2 1 6\n1 1 294967296\n2 1 6\n",
        "5000000000\n4294967296\n"
    )]
    #[case::sample_3_single_vertex("1\n1\n2 1 1\n", "0\n")]
    #[case::sample_4(
        "8\n1 2 105\n1 3 103\n2 4 105\n2 5 100\n5 6 101\n3 7 106\n3 8 100\n18\n2 2 8\n2 3 6\n1 4 108\n2 3 4\n2 3 5\n2 5 5\n2 3 1\n2 4 3\n1 1 107\n2 3 1\n2 7 6\n2 3 8\n2 1 5\n2 7 6\n2 4 7\n2 1 7\n2 5 3\n2 8 6\n",
        "308\n409\n313\n316\n0\n103\n313\n103\n525\n100\n215\n525\n421\n209\n318\n519\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

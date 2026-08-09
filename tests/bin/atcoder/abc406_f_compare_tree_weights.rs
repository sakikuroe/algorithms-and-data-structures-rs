use super::super::common;

/// `ac-abc406-f-compare-tree-weights` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc406-f-compare-tree-weights");

// ac-abc406-f-compare-tree-weights のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_abc406_f_compare_tree_weights {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-abc406-f-compare-tree-weights バイナリへ標準入力として渡す
    /// - Then: 各クエリでの2つの部分木の重みの差が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "6\n1 2\n1 3\n2 4\n4 5\n4 6\n5\n2 1\n1 1 3\n2 1\n1 4 10\n2 5\n";
        let expected = "2\n1\n17\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

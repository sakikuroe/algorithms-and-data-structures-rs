use super::super::common;

/// `ac-typical90-am-tree-distance` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-am-tree-distance");

// ac-typical90-am-tree-distance のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_typical90_am_tree_distance {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル1, 2, 3である
    /// - When: ac-typical90-am-tree-distance バイナリへ標準入力として渡す
    /// - Then: 全頂点対の距離の総和が期待値と一致する
    #[test]
    fn matches_official_samples() {
        let cases = [
            ("2\n1 2\n", "1\n"),
            ("4\n1 2\n1 3\n1 4\n", "9\n"),
            (
                "12\n1 2\n3 1\n4 2\n2 5\n3 6\n3 7\n8 4\n4 9\n10 5\n11 7\n7 12\n",
                "211\n",
            ),
        ];
        for (input, expected) in cases {
            // Given, When
            let result = common::run_binary(BIN, input);
            // Then
            assert_eq!(expected, result);
        }
    }
}

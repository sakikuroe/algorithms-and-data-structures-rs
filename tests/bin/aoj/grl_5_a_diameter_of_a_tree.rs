use super::super::common;

/// `aoj-grl-5-a-diameter-of-a-tree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-grl-5-a-diameter-of-a-tree");

// aoj-grl-5-a-diameter-of-a-tree のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod aoj_grl_5_a_diameter_of_a_tree {
    use super::*;

    /// Scenario: 問題文の公式サンプル1を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル1である
    /// - When: aoj-grl-5-a-diameter-of-a-tree バイナリへ標準入力として渡す
    /// - Then: 木の直径が期待値と一致する
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "4\n0 1 2\n1 2 1\n1 3 3\n";
        let expected = "5\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 問題文の公式サンプル2を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル2である (単純パス状の木)
    /// - When: aoj-grl-5-a-diameter-of-a-tree バイナリへ標準入力として渡す
    /// - Then: 木の直径が期待値と一致する
    #[test]
    fn matches_official_sample_2() {
        // Given
        let input = "4\n0 1 1\n1 2 2\n2 3 4\n";
        let expected = "7\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use super::super::common;

/// `poj-poj1741-tree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_poj-poj1741-tree");

// poj-1741-tree のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod poj_1741_tree {
    use super::*;

    /// Scenario: 問題文のサンプルを解いたときの標準出力を検証する
    /// - Given: 問題文のサンプル (5頂点、K=4) である
    /// - When: poj-1741-tree バイナリへ標準入力として渡す
    /// - Then: 距離がK以下となる頂点対の個数が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "5 4\n1 2 3\n1 3 1\n1 4 2\n3 5 1\n0 0\n";
        let expected = "8\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 複数のテストケースを続けて処理できる (境界値)
    /// - Given: 同じサンプルを2回連続で与える入力である
    /// - When: poj-1741-tree バイナリへ標準入力として渡す
    /// - Then: 2行とも同じ期待値が出力される
    #[test]
    fn handles_multiple_test_cases() {
        // Given
        let input = "5 4\n1 2 3\n1 3 1\n1 4 2\n3 5 1\n5 4\n1 2 3\n1 3 1\n1 4 2\n3 5 1\n0 0\n";
        let expected = "8\n8\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

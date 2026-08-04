use super::super::common;

/// `lc-lca-by-euler-tour` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-lca-by-euler-tour");

// lc-lca-by-euler-tour のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_lca_by_euler_tour {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: lc-lca-by-euler-tour バイナリへ標準入力として渡す
    /// - Then: 各クエリの LCA が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "5 5\n0 0 2 2\n0 1\n0 4\n1 2\n2 3\n3 4\n";
        let expected = "0\n0\n0\n2\n2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

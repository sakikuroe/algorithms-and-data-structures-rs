use super::super::common;

/// `bj-bj15480-lca-and-query` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_bj-bj15480-lca-and-query");

// bj-15480-lca-and-query のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod bj_15480_lca_and_query {
    use super::*;

    /// Scenario: 問題文のサンプルを解いたときの標準出力を検証する
    /// - Given: CodeChef TALCA と同じ木・クエリ構造のサンプルである
    ///   (根を差し替えたときに LCA が変化することを確認する)
    /// - When: bj-15480-lca-and-query バイナリへ標準入力として渡す
    /// - Then: 各クエリで指定した根のもとでの LCA が期待値と一致する
    #[test]
    fn matches_hand_verified_sample() {
        // Given
        let input = "4\n1 2\n2 3\n1 4\n2\n1 4 2\n2 4 2\n";
        let expected = "1\n2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

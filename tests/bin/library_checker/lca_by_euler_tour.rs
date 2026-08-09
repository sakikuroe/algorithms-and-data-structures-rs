use super::super::common;

/// `lc-lca-by-euler-tour` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-lca-by-euler-tour");

// lc-lca-by-euler-tour のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_lca_by_euler_tour {
    use super::*;

    /// Scenario: 手計算で検証したサンプルを解いたときの標準出力を検証する
    /// - Given: Library Checker は JS SPA であり公式サンプルを直接取得
    ///   できないため、手元で構築し正しさを手計算で確認した木とクエリ
    ///   (`lca` (HLD ベース) のテストと同じ入力であり、実装技術の異なる
    ///   両者が同じ答えを返すことも兼ねて確認できる) である
    /// - When: lc-lca-by-euler-tour バイナリへ標準入力として渡す
    /// - Then: 各クエリの LCA が、手計算した期待値と一致する
    #[test]
    fn matches_hand_verified_sample() {
        // Given
        let input = "5 5\n0 0 2 2\n0 1\n0 4\n1 2\n2 3\n3 4\n";
        let expected = "0\n0\n0\n2\n2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

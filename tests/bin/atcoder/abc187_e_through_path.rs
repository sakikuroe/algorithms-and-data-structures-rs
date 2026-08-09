use super::super::common;

/// `ac-abc187-e-through-path` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc187-e-through-path");

// ac-abc187-e-through-path のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_abc187_e_through_path {
    use super::*;

    /// Scenario: 問題文の公式サンプル1を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル1である
    /// - When: ac-abc187-e-through-path バイナリへ標準入力として渡す
    /// - Then: 全クエリ処理後の各頂点の値が期待値と一致する
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "5\n1 2\n2 3\n2 4\n4 5\n4\n1 1 1\n1 4 10\n2 1 100\n2 2 1000\n";
        let expected = "11\n110\n1110\n110\n100\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 問題文の公式サンプル2を解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプル2である。サンプル1と異なり、根の子の
    ///   部分木が頂点番号の並び上で末尾まで続く辺を含む
    /// - When: ac-abc187-e-through-path バイナリへ標準入力として渡す
    /// - Then: 全クエリ処理後の各頂点の値が期待値と一致する
    #[test]
    fn matches_official_sample_2() {
        // Given
        let input = "7\n2 1\n2 3\n4 2\n4 5\n6 1\n3 7\n7\n2 2 1\n1 3 2\n2 2 4\n1 6 8\n1 3 16\n2 4 32\n2 1 64\n";
        let expected = "72\n8\n13\n26\n58\n72\n5\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

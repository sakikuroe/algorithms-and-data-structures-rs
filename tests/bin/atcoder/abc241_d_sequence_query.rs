use super::super::common;

/// `ac-abc241-d-sequence-query` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc241-d-sequence-query");

// ac-abc241-d-sequence-query のテスト: 標準入出力を検証する。
mod ac_abc241_d_sequence_query {
    use super::*;

    /// Scenario: 上限・下限付きの順位クエリを処理する。
    /// - Given: 問題文の公式サンプル 1 がある。
    /// - When: バイナリへ標準入力として渡す。
    /// - Then: 順位クエリの結果が公式出力と一致する。
    #[test]
    fn matches_official_sample() {
        // Given
        let input =
            "11\n1 20\n1 10\n1 30\n1 20\n3 15 1\n3 15 2\n3 15 3\n3 15 4\n2 100 5\n1 1\n2 100 5\n";
        let expected = "20\n20\n30\n-1\n-1\n1\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

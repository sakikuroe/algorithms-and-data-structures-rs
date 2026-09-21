use super::super::common;

/// `ac-abc405-f-chord-crossing` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc405-f-chord-crossing");

// ac-abc405-f-chord-crossing のテスト: 標準入出力を検証する。
mod ac_abc405_f_chord_crossing {
    use super::*;

    /// Scenario: 指定した弦と交差する既存の弦の本数を出力する。
    /// - Given: 問題文の公式サンプル 1 がある。
    /// - When: バイナリへ標準入力として渡す。
    /// - Then: 各クエリの交差数が公式出力と一致する。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "4 2\n2 4\n6 8\n3\n1 3\n3 7\n1 5\n";
        let expected = "1\n2\n0\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

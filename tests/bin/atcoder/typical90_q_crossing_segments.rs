use super::super::common;

/// `ac-typical90-q-crossing-segments` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-q-crossing-segments");

// ac-typical90-q-crossing-segments のテスト: 標準入出力を検証する。
mod ac_typical90_q_crossing_segments {
    use super::*;

    /// Scenario: 線分の交差する組数を出力する。
    /// - Given: 問題文の公式サンプル 1 がある。
    /// - When: バイナリへ標準入力として渡す。
    /// - Then: 交差する組数が公式出力と一致する。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "6 3\n2 5\n1 4\n1 3\n";
        let expected = "2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

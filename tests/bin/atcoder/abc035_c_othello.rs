use super::super::common;

/// `ac-abc035-c-othello` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc035-c-othello");

// ac-abc035-c-othello のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_abc035_c_othello {
    use super::*;

    /// Scenario: 問題文の公式サンプル 1 を解いたときの標準出力を検証する。
    /// - Given: N=5, Q=4 で、反転操作が (1,4), (2,5), (3,3), (1,5) である。
    /// - When: ac-abc035-c-othello バイナリへ標準入力として渡す。
    /// - Then: 全操作後の盤面が "01010" と一致する。
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "5 4\n1 4\n2 5\n3 3\n1 5\n";
        let expected = "01010\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 問題文の公式サンプル 2 を解いたときの標準出力を検証する。
    /// - Given: N=20, Q=8 で、8 回の反転操作が与えられる。
    /// - When: ac-abc035-c-othello バイナリへ標準入力として渡す。
    /// - Then: 全操作後の盤面が "10110000011110000000" と一致する。
    #[test]
    fn matches_official_sample_2() {
        // Given
        let input = "20 8\n1 8\n4 13\n8 8\n3 18\n5 20\n19 20\n2 7\n4 9\n";
        let expected = "10110000011110000000\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

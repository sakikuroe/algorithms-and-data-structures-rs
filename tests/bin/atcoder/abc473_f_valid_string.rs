use super::super::common;

/// `ac-abc473-f-valid-string` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc473-f-valid-string");

// ac-abc473-f-valid-string のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_abc473_f_valid_string {
    use super::*;

    /// Scenario: 問題文の公式サンプル 1 を解いたときの標準出力を検証する。
    /// - Given: N=10 の初期文字列 "AABBAABABB" と 6 件のクエリがある。
    /// - When: ac-abc473-f-valid-string バイナリへ標準入力として渡す。
    /// - Then: 各判定クエリの結果が "Yes", "No", "Yes", "Yes" と一致する。
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "\
10\n\
AABBAABABB\n\
6\n\
2 1 10\n\
1 5 B\n\
2 1 10\n\
2 6 8\n\
1 3 A\n\
2 1 10\n";
        let expected = "Yes\nNo\nYes\nYes\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

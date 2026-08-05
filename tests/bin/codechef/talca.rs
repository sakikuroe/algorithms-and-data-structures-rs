use super::super::common;

/// `cc-talca` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_cc-talca");

// cc-talca のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod cc_talca {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: cc-talca バイナリへ標準入力として渡す
    /// - Then: クエリごとに指定した根のもとでの LCA が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "4\n1 2\n2 3\n1 4\n2\n1 4 2\n2 4 2\n";
        let expected = "1\n2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

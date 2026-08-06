use super::super::common;

/// `ac-typical90-m-passing` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-m-passing");

// ac-typical90-m-passing のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_m_passing {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-typical90-m-passing バイナリへ標準入力として渡す
    /// - Then: 各頂点を経由したときの移動時間の最小値が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "7 9\n1 2 2\n1 3 3\n2 5 2\n3 4 1\n3 5 4\n4 7 5\n5 6 1\n5 7 6\n6 7 3\n";
        let expected = "8\n8\n9\n9\n8\n8\n8\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

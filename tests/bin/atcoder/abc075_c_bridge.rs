use rstest::rstest;

use super::super::common;

/// `ac-abc075-c-bridge` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc075-c-bridge");

// ac-abc075-c-bridge のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc075_c_bridge {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc075-c-bridge バイナリへ標準入力として渡す
    /// - Then: 橋の本数が期待値と一致する
    #[rstest]
    // - Given: 橋が4本存在するグラフである
    #[case::sample_1("7 7\n1 3\n2 7\n3 4\n4 5\n4 6\n5 6\n6 7\n", "4\n")]
    // - Given: 3頂点の閉路であり、橋が存在しないグラフである
    #[case::sample_2_no_bridge("3 3\n1 2\n1 3\n2 3\n", "0\n")]
    // - Given: 単純パスであり、すべての辺が橋であるグラフである
    #[case::sample_3_all_bridges("6 5\n1 2\n2 3\n3 4\n4 5\n5 6\n", "5\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

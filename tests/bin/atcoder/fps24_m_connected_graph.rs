use rstest::rstest;

use super::super::common;

/// `ac-fps24-m-connected-graph` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-fps24-m-connected-graph");

// ac-fps24-m-connected-graph のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_fps24_m_connected_graph {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-fps24-m-connected-graph バイナリへ標準入力として渡す
    /// - Then: 各ケースの期待値と一致する
    #[rstest]
    // - Given: fps_24_m のサンプル 1 の入力である
    #[case::sample_1("3\n", "4\n")]
    // - Given: fps_24_m のサンプル 2 の入力である (N が大きいケース)
    #[case::sample_2("123456\n", "317059543\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `lc-lca` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-lca");

// lc-lca のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_lca {
    use super::*;

    /// Scenario: 手元で構築した小さな木に対するクエリを解いたときの標準出力を検証する
    /// - Given: 頂点0を根とし、`0-1`, `0-2`, `1-3`, `1-4` の辺を持つ木である
    /// - When: lc-lca バイナリへ標準入力として渡す
    /// - Then: 各クエリの LCA が期待通りである
    #[rstest]
    // - Given: 頂点1と頂点2、頂点3と頂点4、頂点2と頂点4のLCAをそれぞれ問う
    #[case::sample_1("5 3\n0 0 1 1\n1 2\n3 4\n2 4\n", "0\n1\n0\n")]
    fn matches_hand_verified_example(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

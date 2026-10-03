use rstest::rstest;

use super::super::common;

/// 疎な遅延版の ABC035 C バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc035-c-othello-sparse-lazy-online");

/// 疎な遅延セグメント木による反転結果を確認する。
mod ac_abc035_c_othello_sparse_lazy_online {
    use super::*;

    /// Scenario: 反転作用の合成後も最終盤面が正しい。
    /// - Given: 公式サンプルの反転区間がある。
    /// - When: 疎な遅延版のバイナリへ渡す。
    /// - Then: 最終盤面が公式出力と一致する。
    #[rstest]
    #[case::sample1("5 4\n1 4\n2 5\n3 3\n1 5\n", "01010\n")]
    #[case::sample2(
        "20 8\n1 8\n4 13\n8 8\n3 18\n5 20\n19 20\n2 7\n4 9\n",
        "10110000011110000000\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

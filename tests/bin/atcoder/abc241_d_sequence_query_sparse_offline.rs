use rstest::rstest;

use super::super::common;

/// 疎な offline 版を使う ABC241 D バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc241-d-sequence-query-sparse-offline");

/// 圧縮した挿入座標で順位クエリを処理できることを確認する。
mod ac_abc241_d_sequence_query_sparse_offline {
    use super::*;

    /// Scenario: 重複と広い座標範囲を含む順位クエリに正しく答える。
    /// - Given: 公式サンプルまたは最大座標を含む入力がある。
    /// - When: 疎な offline 版のバイナリへ標準入力として渡す。
    /// - Then: 各順位クエリの結果が期待値と一致する。
    #[rstest]
    #[case::official(
        "11\n1 20\n1 10\n1 30\n1 20\n3 15 1\n3 15 2\n3 15 3\n3 15 4\n2 100 5\n1 1\n2 100 5\n",
        "20\n20\n30\n-1\n-1\n1\n"
    )]
    #[case::wide_coordinates(
        "9\n1 1000000000000000000\n1 2\n1 1000000000000000000\n2 1000000000000000000 1\n2 1000000000000000000 2\n2 1000000000000000000 3\n3 3 2\n3 1000000000000000000 3\n2 1 1\n",
        "1000000000000000000\n1000000000000000000\n2\n1000000000000000000\n-1\n-1\n"
    )]
    fn matches_expected_order_statistics(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

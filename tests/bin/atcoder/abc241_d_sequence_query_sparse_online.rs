use rstest::rstest;

use super::super::common;

/// 疎な online 版を使う ABC241 D バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc241-d-sequence-query-sparse-online");

/// 入力中に初めて現れる座標でも順位クエリを処理できることを確認する。
mod ac_abc241_d_sequence_query_sparse_online {
    use super::*;

    /// Scenario: 重複する挿入値と後から現れる最大座標を正しく扱う。
    /// - Given: 公式サンプルまたは挿入前後に問い合わせる入力がある。
    /// - When: 疎な online 版のバイナリへ標準入力として渡す。
    /// - Then: 各順位クエリの結果が期待値と一致する。
    #[rstest]
    #[case::official(
        "11\n1 20\n1 10\n1 30\n1 20\n3 15 1\n3 15 2\n3 15 3\n3 15 4\n2 100 5\n1 1\n2 100 5\n",
        "20\n20\n30\n-1\n-1\n1\n"
    )]
    #[case::maximum_coordinate_after_queries(
        "5\n2 1000000000000000000 1\n3 1 1\n1 1000000000000000000\n3 1000000000000000000 1\n2 1000000000000000000 1\n",
        "-1\n-1\n1000000000000000000\n1000000000000000000\n"
    )]
    fn matches_expected_order_statistics(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

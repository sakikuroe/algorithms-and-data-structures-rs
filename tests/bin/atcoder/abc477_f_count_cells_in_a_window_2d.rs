use rstest::rstest;

use super::super::common;

/// 疎な offline 2 次元版の ABC477 F バイナリへのパス。
const OFFLINE_BIN: &str =
    env!("CARGO_BIN_EXE_ac-abc477-f-count-cells-in-a-window-2d-sparse-offline");

/// 2 次元の疎なセグメント木で黒マスの矩形和を数える。
mod ac_abc477_f_count_cells_in_a_window_2d {
    use super::*;

    /// Scenario: 公式サンプルの矩形内の黒マス数を返す。
    /// - Given: 各行に一つの黒区間がある入力を受け取る。
    /// - When: 疎な offline 2D 木に差分の点を置いて矩形を集約する。
    /// - Then: 公式出力と一致する。
    #[rstest]
    #[case::sample1("3 6 3\n2 4\n1 1\n4 6\n1 2 1 4\n2 3 3 6\n1 1 5 6\n", "4\n3\n0\n")]
    #[case::sample2(
        "10 20 12\n3 8\n1 4\n12 19\n5 14\n2 2\n9 17\n1 20\n6 11\n16 20\n7 7\n2 8 4 15\n6 10 1 9\n3 4 1 4\n1 10 1 20\n4 9 10 18\n5 5 1 20\n1 6 8 8\n8 10 13 20\n2 7 1 5\n6 9 6 16\n3 10 18 20\n7 10 7 12\n",
        "40\n15\n0\n70\n27\n1\n2\n5\n11\n26\n8\n12\n"
    )]
    #[case::wide_columns(
        "2 200000 3\n1 200000\n100000 100000\n1 2 1 200000\n2 2 99999 100001\n1 1 200000 200000\n",
        "200001\n1\n1\n"
    )]
    fn matches_expected_counts(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(OFFLINE_BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

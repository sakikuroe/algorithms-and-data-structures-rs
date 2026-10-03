use rstest::rstest;

use super::super::common;

/// `point_add_rectangle_sum` の疎な online 版バイナリへのパス。
const ONLINE_BIN: &str = env!("CARGO_BIN_EXE_lc-point-add-rectangle-sum-sparse-online");
/// `point_add_rectangle_sum` の疎な offline 版バイナリへのパス。
const OFFLINE_BIN: &str = env!("CARGO_BIN_EXE_lc-point-add-rectangle-sum-sparse-offline");

/// `point_add_rectangle_sum` の疎な 2 次元セグメント木実装を確認する。
mod lc_point_add_rectangle_sum {
    use super::*;

    /// Scenario: 初期点、点加算、矩形境界を含む操作列を処理する。
    /// - Given: 重複座標、空矩形、全体矩形、境界上の点を含む入力がある。
    /// - When: 疎な online 版と offline 版のバイナリへ同じ入力を渡す。
    /// - Then: 素朴に計算した矩形総和と両実装の出力が一致する。
    #[rstest]
    #[case::point_add_and_rectangle_sum(
        "4 10\n1 2 5\n3 4 2\n1 2 7\n8 9 10\n0 1 2 3\n1 0 0 4 5\n1 1 2 1 3\n1 0 0 10 10\n0 8 9 1\n1 8 9 9 10\n0 100 100 4\n1 99 99 101 101\n0 1 2 5\n1 0 2 2 3\n",
        "17\n0\n27\n11\n4\n20\n"
    )]
    fn matches_hand_verified_case(
        #[values(ONLINE_BIN, OFFLINE_BIN)] bin: &str,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // When
        let result = common::run_binary(bin, input);
        // Then
        assert_eq!(expected, result);
    }
}

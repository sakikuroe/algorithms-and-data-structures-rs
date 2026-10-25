//! abc481_e の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc481-e-most-brilliant-starry-sky");

mod abc481_e_most_brilliant_starry_sky {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1(
        "3\n4\n2 1\n2 -1\n-1 2\n-2 -3\n3\n1 0\n2 1\n3 -2\n20\n351656128 486890400\n43833334 189818465\n244758369 410609525\n82390518 959687989\n37964139 450230904\n-27725133 788196612\n392638907 -544184247\n-305242056 -60014958\n-928747557 460755669\n-415081310 638281062\n307908966 -865172261\n580339611 459847409\n42654321 -744472144\n-286500328 34461456\n-874976232 -245368444\n462187336 508372206\n-229275227 74979882\n-180126519 740406850\n877877433 -211582842\n256315745 -928285605\n",
        "3\n3\n15\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 境界をまたぐ半平面の点数が直接走査と一致します。
    /// - Given: 原点と同一直線上に並ばない小さい点群があります。
    /// - When: 偏角順の区間検索で最大点数を求めます。
    /// - Then: 各境界を直接調べた最大点数と一致します。
    #[test]
    fn matches_direct_halfplane_counts() {
        // Given
        for seed in 0..24 {
            let mut points = Vec::new();
            for point in geometry_common::create_points(60, seed) {
                if point != (0, 0) && points.iter().all(|&p| geometry_common::det(p, point) != 0) {
                    points.push(point);
                }
                if points.len() == 9 {
                    break;
                }
            }
            assert_eq!(9, points.len());
            let input = format!("1\n{}", geometry_common::format_points(&points));
            let expected = format!("{}\n", geometry_common::reference_halfplane(&points));
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            assert_eq!(expected, result);
        }
    }

    /// Scenario: 一点と角度の切れ目に近い点を扱います。
    /// - Given: 星が一個、または負の x 軸の両側に配置されています。
    /// - When: 最大点数を求めます。
    /// - Then: 一点を数え落とさず、切れ目をまたいだ領域も数えます。
    #[rstest::rstest]
    #[case::one_star("1\n1\n-1000000000 1\n", "1\n")]
    #[case::wraps_negative_axis("1\n3\n-1000000000 -1\n-1000000000 1\n-1 1000000000\n", "3\n")]
    fn handles_single_and_wrapped_ranges(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

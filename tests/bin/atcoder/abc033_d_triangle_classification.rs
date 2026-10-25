//! abc033_d の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc033-d-triangle-classification");

mod abc033_d_triangle_classification {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("5\n1 3\n2 2\n3 2\n4 1\n4 3\n", "1 2 7\n")]
    #[case::sample_2("9\n2 0\n1 1\n3 1\n1 2\n5 2\n0 3\n4 3\n2 4\n4 4\n", "27 14 43\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 角度区間による分類が、三角形ごとの内積判定と一致します。
    /// - Given: どの三点も共線でない放物線上の点群があります。
    /// - When: 直角と鈍角を方向の区間で数えます。
    /// - Then: 全三角形を内積で分類した結果と一致します。
    #[test]
    fn matches_triangle_enumeration() {
        // Given
        for seed in 0..18_i64 {
            let points = (-4..=4_i64)
                .map(|x| {
                    let y = x * x + seed % 3 * x + seed / 3;
                    if seed % 2 == 0 { (x, y) } else { (y, x) }
                })
                .collect::<Vec<_>>();
            let input = geometry_common::format_points(&points);
            let (acute, right, obtuse) = geometry_common::reference_triangle_counts(&points);
            let expected = format!("{acute} {right} {obtuse}\n");
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            assert_eq!(expected, result);
        }
    }
}

//! sort_points_by_argument の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_lc-sort-points-by-argument");

mod sort_points_by_argument {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 入力の重複を保ち、偏角が非減少になります。
    #[rstest::rstest]
    #[case::sample_1("8\n1 0\n0 0\n-1 0\n0 1\n0 -1\n1 1\n2 2\n-10 -1\n")]
    fn matches_official_samples(#[case] input: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_argument_sort(input, &result);
    }

    /// Scenario: 原点と同偏角、反対方向と近接した偏角を整列できます。
    /// - Given: 軸上、同一座標、誤差で潰れやすい方向があります。
    /// - When: 偏角だけを比較します。
    /// - Then: 点の重複数を保ち、厳密な順序になります。
    #[test]
    fn handles_zero_axes_duplicates_and_near_angles() {
        // Given
        let points = vec![
            (0, 0),
            (1, 0),
            (2, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (0, 0),
            (1, 1),
            (2, 2),
            (-1, -1),
            (1000000000, 999999999),
            (999999999, 999999998),
        ];
        let input = geometry_common::format_points(&points);
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        geometry_common::assert_argument_sort(&input, &result);
    }

    /// Scenario: 正負の座標を含む点群も偏角順になります。
    /// - Given: 再現可能な小さい点群があります。
    /// - When: バイナリーで整列します。
    /// - Then: 出力は入力の順列で、偏角が非減少になります。
    #[test]
    fn preserves_arbitrary_point_multisets() {
        // Given
        for seed in 0..20 {
            let input = geometry_common::format_points(&geometry_common::create_points(30, seed));
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            geometry_common::assert_argument_sort(&input, &result);
        }
    }
}

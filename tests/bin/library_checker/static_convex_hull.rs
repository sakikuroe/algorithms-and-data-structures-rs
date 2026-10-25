//! static_convex_hull の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_lc-static-convex-hull");

mod static_convex_hull {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 始点によらず、共線点を除く反時計回りの凸包になります。
    #[rstest::rstest]
    #[case::sample_1("1\n9\n0 0\n0 1\n0 2\n1 0\n1 1\n1 2\n2 0\n2 1\n2 2\n")]
    #[case::sample_2("4\n0\n1\n1 1\n3\n1 1\n1 1\n1 1\n4\n-2 -2\n-1 -1\n0 0\n0 0\n")]
    fn matches_official_samples(#[case] input: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_hulls(input, &result);
    }

    /// Scenario: 空・一点・線分・重複・辺上の点も凸包として扱います。
    /// - Given: 退化した点群と、再現可能な複数の小さい点群があります。
    /// - When: 凸包を構築します。
    /// - Then: Gift wrapping の頂点集合と反時計回りの境界に一致します。
    #[test]
    fn matches_independent_hulls_and_degenerate_inputs() {
        // Given
        let mut cases = vec![
            vec![],
            vec![(3, 4)],
            vec![(3, 4); 5],
            vec![(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (0, 0)],
            vec![(0, 0), (1, 0), (2, 0), (2, 2), (0, 2), (1, 1), (0, 0)],
        ];
        for seed in 0..25 {
            cases.push(geometry_common::create_points(12, seed));
        }
        let input = geometry_common::format_cases(&cases);
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        geometry_common::assert_hulls(&input, &result);
    }
}

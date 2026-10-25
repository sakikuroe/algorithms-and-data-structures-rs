//! abc296_g の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc296-g-polygon-and-points");

mod abc296_g_polygon_and_points {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("4\n0 4\n-2 2\n-1 0\n3 1\n3\n-1 3\n0 2\n2 0\n", "ON\nIN\nOUT\n")]
    #[case::sample_2("3\n0 0\n1 0\n0 1\n3\n0 0\n1 0\n0 1\n", "ON\nON\nON\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 凸多角形の問い合わせが、全辺の走査と一致します。
    /// - Given: 小さい凸包と、頂点を含む多様な問い合わせ点があります。
    /// - When: 公開 API で内部・境界・外部を調べます。
    /// - Then: 全辺の向きの判定と一致します。
    #[test]
    fn matches_all_edge_classification() {
        // Given
        for seed in 0..16 {
            let vertices =
                geometry_common::reference_hull(&geometry_common::create_points(12, seed));
            assert!(vertices.len() >= 3);
            let mut queries = geometry_common::create_points(35, seed + 100);
            queries.extend_from_slice(&vertices);
            let mut input = geometry_common::format_points(&vertices);
            input.push_str(&geometry_common::format_points(&queries));
            let expected = queries
                .iter()
                .map(|&point| {
                    format!(
                        "{}\n",
                        geometry_common::reference_location(&vertices, point)
                    )
                })
                .collect::<String>();
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            assert_eq!(expected, result);
        }
    }

    /// Scenario: 大きな座標の辺上、頂点、内部、辺の延長上を区別します。
    /// - Given: 座標上限付近まで広がる正方形があります。
    /// - When: 境界に近い点を問い合わせます。
    /// - Then: 線分の延長上の外部を境界と誤認しません。
    #[test]
    fn distinguishes_large_boundary_and_extensions() {
        // Given
        let input = "4\n-999999999 -999999999\n999999999 -999999999\n999999999 999999999\n-999999999 999999999\n5\n0 0\n999999999 0\n999999999 999999999\n1000000000 999999999\n-999999999 -1000000000\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!("IN\nON\nON\nOUT\nOUT\n", result);
    }
}

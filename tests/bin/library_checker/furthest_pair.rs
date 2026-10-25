//! furthest_pair の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_lc-furthest-pair");

mod lc_furthest_pair {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 異なる元添字の組で、距離が最大になります。
    #[rstest::rstest]
    #[case::sample_1(
        "4\n5\n-1 -1\n-6 4\n-9 -7\n2 5\n-7 6\n2\n1 2\n3 4\n3\n1 1\n1 1\n1 1\n2\n-1000000000 1000000000\n1000000000 -1000000000\n"
    )]
    fn matches_official_samples(#[case] input: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_furthest_pairs(input, &result);
    }

    /// Scenario: 同率・同一座標でも異なる元添字の最遠対を返します。
    /// - Given: 同一点の重複、正方形、広い座標、小さい点群があります。
    /// - When: 最遠点対を取得します。
    /// - Then: 添字が異なり、距離が全点対の最大値と一致します。
    #[test]
    fn preserves_original_indices_and_distance_ties() {
        // Given
        let mut cases = vec![
            vec![(7, 8); 4],
            vec![(0, 0), (1, 0), (1, 1), (0, 1)],
            vec![(-1000000000, -1000000000), (1000000000, 1000000000), (0, 0)],
            vec![(3, 4), (3, 4), (-3, -4), (-3, -4)],
        ];
        for seed in 0..25 {
            cases.push(geometry_common::create_points(12, seed));
        }
        let input = geometry_common::format_cases(&cases);
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        geometry_common::assert_furthest_pairs(&input, &result);
    }
}

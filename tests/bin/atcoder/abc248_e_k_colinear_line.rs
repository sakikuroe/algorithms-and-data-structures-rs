//! abc248_e の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc248-e-k-colinear-line");

mod abc248_e_k_colinear_line {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("5 2\n0 0\n1 0\n0 1\n-1 0\n0 -1\n", "6\n")]
    #[case::sample_2("1 1\n0 0\n", "Infinity\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 無向方向で数えた直線が、整数係数の全点対列挙と一致します。
    /// - Given: 正負の座標と複数の必要点数があります。
    /// - When: 基準点ごとに共線方向を集約します。
    /// - Then: 同一直線を重複して数えず、期待する本数を返します。
    #[test]
    fn matches_integer_line_enumeration() {
        // Given
        for seed in 0..12 {
            let points = geometry_common::create_points(9, seed);
            for k in [1, 2, 3, 5, 9] {
                let mut input = format!("{} {k}\n", points.len());
                for &(x, y) in &points {
                    input.push_str(&format!("{x} {y}\n"));
                }
                let expected = geometry_common::reference_colinear_lines(&points, k);
                // When
                let result = common::run_binary(BIN, &input);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    /// Scenario: 原点の両側にある点を一つの直線として扱います。
    /// - Given: 五つの点が同一直線上にあります。
    /// - When: 三点以上を含む直線を数えます。
    /// - Then: 向きが逆でも一つの直線になります。
    #[test]
    fn merges_opposite_directions_on_one_line() {
        // Given
        let input = "5 3\n-1000000000 -1000000000\n-1 -1\n0 0\n2 2\n1000000000 1000000000\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!("1\n", result);
    }
}

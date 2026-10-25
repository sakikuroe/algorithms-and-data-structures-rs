//! abc139_f の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc139-f-engines");

mod ac_abc139_f_engines {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 許容誤差の範囲で公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1(
        "3\n0 10\n5 -5\n-5 -5\n",
        "10.000000000000000000000000000000000000000000000000\n"
    )]
    #[case::sample_2(
        "5\n1 1\n1 0\n0 1\n-1 0\n0 -1\n",
        "2.828427124746190097603377448419396157139343750753\n"
    )]
    #[case::sample_3(
        "5\n1 1\n2 2\n3 3\n4 4\n5 5\n",
        "21.213203435596425732025330863145471178545078130654\n"
    )]
    #[case::sample_4(
        "3\n0 0\n0 1\n1 0\n",
        "1.414213562373095048801688724209698078569671875376\n"
    )]
    #[case::sample_5(
        "1\n90447 91000\n",
        "128303.000000000000000000000000000000000000000000000000\n"
    )]
    #[case::sample_6(
        "2\n96000 -72000\n-72000 54000\n",
        "120000.000000000000000000000000000000000000000000000000\n"
    )]
    #[case::sample_7(
        "10\n1 2\n3 4\n5 6\n7 8\n9 10\n11 12\n13 14\n15 16\n17 18\n19 20\n",
        "148.660687473185055226120082139313966514489855137208\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_float(expected.trim().parse().unwrap(), &result, 1e-10);
    }

    /// Scenario: 連続する方向の列挙が、全てのエンジン部分集合と一致します。
    /// - Given: 原点・同方向・逆方向を含みうる短いベクトル列があります。
    /// - When: 偏角順の連続部分の和を求めます。
    /// - Then: 全部分集合の最大距離と一致します。
    #[test]
    fn matches_all_subsets() {
        // Given
        let mut cases = vec![
            vec![(0, 0), (0, 0)],
            vec![(1, 0), (2, 0), (-4, 0), (0, 3)],
            vec![(10, 0), (5, 9), (-8, 2)],
        ];
        for seed in 0..20 {
            cases.push(geometry_common::create_points(8, seed));
        }
        for points in cases {
            let input = geometry_common::format_points(&points);
            let expected = geometry_common::reference_engines(&points);
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            geometry_common::assert_float(expected, &result, 1e-10);
        }
    }
}

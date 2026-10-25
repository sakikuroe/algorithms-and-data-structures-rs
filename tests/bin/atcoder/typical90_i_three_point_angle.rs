//! typical90_i の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-i-three-point-angle");

mod ac_typical90_i_three_point_angle {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 許容誤差の範囲で公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("3\n0 0\n0 10\n10 10\n", "90\n")]
    #[case::sample_2("5\n8 6\n9 1\n2 0\n1 0\n0 1\n", "171.869897645844\n")]
    #[case::sample_3("10\n0 0\n1 7\n2 6\n2 8\n3 5\n5 5\n6 7\n7 1\n7 9\n8 8\n", "180\n")]
    #[case::sample_4(
        "40\n298750376 229032640\n602876667 944779015\n909539868 533609371\n231368330 445484152\n408704870 850216874\n349286798 30417810\n807260002 554049450\n40706045 380488344\n749325840 801881841\n459457853 66691229\n5235900 8100458\n46697277 997429858\n827651689 790051948\n981897272 271364774\n536232393 997361572\n449659237 602191750\n294800444 346669663\n792837293 277667068\n997282249 468293808\n444906878 702693341\n894286137 845317003\n27053625 926547765\n739689211 447395911\n902031510 326127348\n582956343 842918193\n235655766 844300842\n438389323 406413067\n862896425 464876303\n68833418 76340212\n911399808 745744264\n551223563 854507876\n196296968 52144186\n431165823 275217640\n424495332 847375861\n337078801 83054466\n648322745 694789156\n301518763 319851750\n432518459 772897937\n630628124 581390864\n313132255 350770227\n",
        "179.9834340684232\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_float(expected.trim().parse().unwrap(), &result, 1e-7);
    }

    /// Scenario: 逆方向付近の探索結果が全三点の角度と一致します。
    /// - Given: 同方向や一直線上の三点も現れる点群があります。
    /// - When: 円周上で逆方向を挟む二候補から最大角度を求めます。
    /// - Then: 全ての角の列挙と一致します。
    #[test]
    fn matches_all_point_triples() {
        // Given
        for seed in 0..20 {
            let points = geometry_common::create_points(8, seed);
            let expected = geometry_common::reference_max_angle(&points);
            let input = geometry_common::format_points(&points);
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            geometry_common::assert_float(expected, &result, 1e-7);
        }
    }

    /// Scenario: 同一直線と最大座標付近の角度を求めます。
    /// - Given: 180 度になる配置と、広い座標での直角があります。
    /// - When: 角度を出力します。
    /// - Then: 180 度または 90 度が許容誤差内で得られます。
    #[rstest::rstest]
    #[case::straight("3\n0 0\n1 1\n1000000000 1000000000\n", 180.0)]
    #[case::large_right_angle("3\n0 0\n1000000000 0\n0 1000000000\n", 90.0)]
    fn handles_straight_and_large_angles(#[case] input: &str, #[case] expected: f64) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        geometry_common::assert_float(expected, &result, 1e-7);
    }
}

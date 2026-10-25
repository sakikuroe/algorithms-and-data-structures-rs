//! abc225_e の公式サンプルと境界・素朴解との照合です。

use super::super::{common, geometry_common};

/// 公開 API を使う解答バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc225-e-seven");

mod ac_abc225_e_seven {
    use super::*;

    /// Scenario: 公開 API を組み合わせた解答が公式サンプルを満たします。
    /// - Given: 問題の公式サンプルがあります。
    /// - When: 解答バイナリーを実行します。
    /// - Then: 公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("3\n1 1\n2 1\n1 2\n", "2\n")]
    #[case::sample_2(
        "10\n414598724 87552841\n252911401 309688555\n623249116 421714323\n605059493 227199170\n410455266 373748111\n861647548 916369023\n527772558 682124751\n356101507 249887028\n292258775 110762985\n850583108 796044319\n",
        "10\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 方向区間の貪欲な選択が、全削除パターンと一致します。
    /// - Given: 第一象限の小さなフの字の集合があります。
    /// - When: 終方向順に内部の重ならない区間を選びます。
    /// - Then: 全部分集合から選んだ最大個数と一致します。
    #[test]
    fn matches_visible_subsets() {
        // Given
        for seed in 0..20 {
            let points = geometry_common::create_points(8, seed)
                .into_iter()
                .map(|(x, y)| (x + 21, y + 21))
                .collect::<Vec<_>>();
            let input = geometry_common::format_points(&points);
            let expected = format!("{}\n", geometry_common::reference_visible_sevens(&points));
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            assert_eq!(expected, result);
        }
    }

    /// Scenario: 同じ端方向で接する区間を両方選べます。
    /// - Given: 二つのフの字が角度 45 度でだけ接します。
    /// - When: 可視な個数を求めます。
    /// - Then: 両方が選ばれます。
    #[test]
    fn allows_touching_end_directions() {
        // Given
        let input = "2\n2 1\n1 2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!("2\n", result);
    }
}

//! 疎な遅延セグメント木の両版で ABC255 H の収穫量を検証します。

use rstest::rstest;

use super::super::common;

/// オンライン版の解答バイナリーへのパス。
const ONLINE_BIN: &str = env!("CARGO_BIN_EXE_ac-abc255-h-range-harvest-query-sparse-lazy-online");
/// オフライン版の解答バイナリーへのパス。
const OFFLINE_BIN: &str = env!("CARGO_BIN_EXE_ac-abc255-h-range-harvest-query-sparse-lazy-offline");

/// 巨大な座標と区間更新の合成を標準入出力経由で検証します。
mod ac_abc255_h_range_harvest_query {
    use super::*;

    /// Scenario: 各収穫で前回の収穫後に実った個数を返します。
    /// - Given: 公式サンプルまたは収穫区間が重なる入力があります。
    /// - When: オンライン版とオフライン版のバイナリーへ入力します。
    /// - Then: 各日の収穫量が期待値と一致します。
    #[rstest]
    #[case::sample1("5 3\n2 2 3\n3 3 4\n5 1 5\n", "10\n15\n50\n")]
    #[case::sample2(
        "711741968710511029 1\n82803157126515475 516874290286751784 588060532191410838\n",
        "603657470\n"
    )]
    #[case::overlapping_and_nested_ranges(
        "6 6\n1 1 6\n2 2 5\n4 3 4\n5 2 5\n7 1 1\n8 1 6\n",
        "21\n14\n14\n28\n6\n85\n"
    )]
    #[case::single_tree("1 3\n1 1 1\n3 1 1\n10 1 1\n", "1\n2\n7\n")]
    #[case::maximum_coordinates_and_days(
        "1000000000000000000 4\n1 1 1000000000000000000\n2 1000000000000000000 1000000000000000000\n3 1 1000000000000000000\n1000000000000000000 1 1000000000000000000\n",
        "75433847\n716070898\n433041149\n117798419\n"
    )]
    fn matches_expected_harvests(
        #[values(ONLINE_BIN, OFFLINE_BIN)] bin: &str,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // When
        let result = common::run_binary(bin, input);
        // Then
        assert_eq!(expected, result);
    }
}

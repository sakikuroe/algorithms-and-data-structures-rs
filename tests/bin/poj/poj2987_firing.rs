use rstest::rstest;

use super::super::common;

/// `poj-poj2987-firing` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_poj-poj2987-firing");

// poj-poj2987-firing のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod poj_poj2987_firing {
    use super::*;

    /// Scenario: 与えられた入力を解いたときの標準出力を検証する
    /// - Given: 公式サンプル、および手計算で検証した小規模なケースである
    /// - When: poj-poj2987-firing バイナリへ標準入力として渡す
    /// - Then: 最大利益を達成する最小の解雇人数と、その最大利益が期待値と一致する
    #[rstest]
    #[case::official_sample("5 5\n8\n-9\n-20\n12\n-10\n1 2\n2 5\n1 4\n3 4\n4 5\n", "2 2\n")]
    #[case::firing_costs_more_than_it_saves("1 0\n-5\n", "0 0\n")]
    #[case::firing_alone_is_profitable("1 0\n5\n", "1 5\n")]
    fn matches_expected_output(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

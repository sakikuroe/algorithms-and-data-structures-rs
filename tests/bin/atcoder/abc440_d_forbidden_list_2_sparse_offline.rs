use rstest::rstest;

use super::super::common;

/// 疎な offline 版を使う ABC440 D バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc440-d-forbidden-list-2-sparse-offline");

/// 禁止数を挟む許可数の順位を調べる。
mod ac_abc440_d_forbidden_list_2_sparse_offline {
    use super::*;

    /// Scenario: 禁止数の連続区間・広い空き区間・末尾の先を扱える。
    /// - Given: 公式サンプルまたは連続する禁止数を含む入力がある。
    /// - When: 疎な offline 版のバイナリへ標準入力として渡す。
    /// - Then: 各質問の答えが期待値と一致する。
    #[rstest]
    #[case::official(
        "5 4\n16 9 2 3 1\n6 10\n12 4\n1 1\n1000000000 1000000000\n",
        "17\n15\n4\n1999999999\n"
    )]
    #[case::consecutive_and_tail(
        "4 6\n1 2 3 10\n1 1\n4 6\n4 7\n10 1\n11 1\n3 1\n",
        "4\n9\n11\n11\n11\n4\n"
    )]
    fn answers_order_statistics(#[case] input: &str, #[case] expected: &str) {
        let result = common::run_binary(BIN, input);
        assert_eq!(expected, result);
    }
}

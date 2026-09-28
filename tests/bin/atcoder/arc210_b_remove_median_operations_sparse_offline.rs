use rstest::rstest;

use super::super::common;

/// 疎な offline 版を使う ARC210 B バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-arc210-b-remove-median-operations-sparse-offline");

/// 値の重複と更新を含む中央値除去の結果を調べる。
mod ac_arc210_b_remove_median_operations_sparse_offline {
    use super::*;

    /// Scenario: 両端の要素数と合計を更新後にも取得できる。
    /// - Given: 公式サンプルまたは重複と最大座標を含む入力がある。
    /// - When: 疎な offline 版のバイナリへ標準入力として渡す。
    /// - Then: 残る要素の総和が期待値と一致する。
    #[rstest]
    #[case::official_1("4 2 2\n5 1 4 3\n1 2\n1 3 3\n2 1 5\n", "10\n13\n")]
    #[case::official_2(
        "6 7 5\n7 1 2 5 5 3\n4 2 5 1 3 3 8\n2 1 3\n2 7 1\n1 3 6\n2 4 2\n1 5 1\n",
        "24\n20\n21\n22\n21\n"
    )]
    #[case::duplicates_and_maximum(
        "2 1 4\n5 5\n5\n1 1 1\n2 1 1\n1 2 1\n2 1 1000000000\n",
        "6\n6\n2\n1000000001\n"
    )]
    fn answers_after_replacements(#[case] input: &str, #[case] expected: &str) {
        let result = common::run_binary(BIN, input);
        assert_eq!(expected, result);
    }
}

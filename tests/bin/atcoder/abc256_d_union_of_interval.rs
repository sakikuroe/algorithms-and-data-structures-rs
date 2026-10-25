//! ABC256 D のバイナリーを、公式サンプルと密な配列上の被覆で検証します。

use super::super::common;

/// ABC256 D の実行バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc256-d-union-of-interval");

// 統合テスト: 1 の区間の結合と、昇順での全区間列挙を検証します。
mod ac_abc256_d_union_of_interval {
    use super::*;

    /// Scenario: 公式サンプルの和集合を最小個数の区間にまとめます。
    /// - Given: 隣接区間または重複区間を含む公式サンプルがあります。
    /// - When: バイナリーに入力します。
    /// - Then: 公式の出力と一致する半開区間を返します。
    #[rstest::rstest]
    #[case::sample_1("3\n10 20\n20 30\n40 50\n", "10 30\n40 50\n")]
    #[case::sample_2("3\n10 40\n30 60\n20 50\n", "10 60\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 挿入順に依存せず、重複・包含・上限付近の区間を扱います。
    /// - Given: 区間の並びや包含関係が異なる入力があります。
    /// - When: バイナリーで和集合を列挙します。
    /// - Then: 最小個数の半開区間が昇順で出力されます。
    #[rstest::rstest]
    #[case::unsorted_duplicates(
        "6\n40 50\n10 20\n20 30\n15 25\n40 50\n1 2\n",
        "1 2\n10 30\n40 50\n"
    )]
    #[case::contained_at_upper_bound("3\n199999 200000\n1 200000\n2 3\n", "1 200000\n")]
    fn handles_unsorted_and_contained_intervals(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 区間の和集合が、各位置を塗る素朴な配列と一致します。
    /// - Given: 小さな座標上に複数成分を作る、100 個の入力区間があります。
    /// - When: 素朴な配列から期待する半開区間列を求めます。
    /// - Then: バイナリーの出力する区間列と一致します。
    #[test]
    fn matches_dense_coverage() {
        // Given
        let n = 100;
        let mut covered = [false; 64];
        let mut input = format!("{n}\n");
        for step in 0..n {
            let left = 1 + (step % 3) * 20 + step * 7 % 10;
            let right = left + 1 + step % 5;
            input.push_str(&format!("{left} {right}\n"));
            covered[left..right].fill(true);
        }
        let mut expected = String::new();
        let mut x = 0;
        while x < covered.len() {
            if !covered[x] {
                x += 1;
                continue;
            }
            let start = x;
            while x < covered.len() && covered[x] {
                x += 1;
            }
            expected.push_str(&format!("{start} {x}\n"));
        }
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        assert_eq!(expected, result);
    }
}

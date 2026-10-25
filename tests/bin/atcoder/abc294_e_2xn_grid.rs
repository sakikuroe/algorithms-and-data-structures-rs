//! ABC294 E のバイナリーを、公式サンプルと展開した 2 行の比較で検証します。

use super::super::common;

/// ABC294 E の実行バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc294-e-2xn-grid");

// 統合テスト: 値ごとの区間集合の構築、AND、共通位置数の集計を検証します。
mod ac_abc294_e_2xn_grid {
    use super::*;

    /// Scenario: 公式サンプルの一致する列数を求めます。
    /// - Given: 連長圧縮された 2 行を表す公式サンプルがあります。
    /// - When: バイナリーで値ごとの位置集合を AND します。
    /// - Then: 一致列数が公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("8 4 3\n1 2\n3 2\n2 3\n3 1\n1 4\n2 1\n3 3\n", "4\n")]
    #[case::sample_2("10000000000 1 1\n1 10000000000\n1 10000000000\n", "10000000000\n")]
    #[case::sample_3(
        "1000 4 7\n19 79\n33 463\n19 178\n33 280\n19 255\n33 92\n34 25\n19 96\n12 11\n19 490\n33 31\n",
        "380\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 共通しない値、繰り返す値、境界と大きな連長を扱います。
    /// - Given: 値と区間の対応が異なる連長圧縮列があります。
    /// - When: バイナリーで一致列数を求めます。
    /// - Then: 交差した位置だけを数え、境界に接するだけの区間は数えません。
    #[rstest::rstest]
    #[case::different_values("7 1 1\n1 7\n2 7\n", "0\n")]
    #[case::repeated_values("12 4 4\n1 3\n2 2\n1 4\n3 3\n1 1\n2 4\n1 5\n3 2\n", "9\n")]
    #[case::touching_boundaries("4 2 2\n1 2\n2 2\n2 2\n1 2\n", "0\n")]
    #[case::large_lengths(
        "1000000000000 1 2\n1000000000 1000000000000\n1000000000 999999999999\n1 1\n",
        "999999999999\n"
    )]
    fn handles_values_and_interval_boundaries(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 小さな列を実際に展開して比較した結果と一致します。
    /// - Given: 繰り返す値と異なる区切りを持つ 20 組の短い列があります。
    /// - When: 入力を連長圧縮し、元の配列を列ごとに比較して期待値を求めます。
    /// - Then: バイナリーの一致列数が期待値と一致します。
    #[test]
    fn matches_expanded_rows() {
        // Given
        let length = 80;
        let encode = |values: &[usize]| {
            let mut runs = Vec::<(usize, usize)>::new();
            for &value in values {
                if let Some((previous, count)) = runs.last_mut()
                    && *previous == value
                {
                    *count += 1;
                } else {
                    runs.push((value, 1));
                }
            }
            runs
        };
        for seed in 0..20 {
            let upper = (0..length)
                .map(|i| (i / 3 + seed) % 4 + 1)
                .collect::<Vec<_>>();
            let lower = (0..length)
                .map(|i| {
                    if i % 7 < 3 {
                        upper[i]
                    } else {
                        (i / 5 + seed * 3) % 5 + 1
                    }
                })
                .collect::<Vec<_>>();
            let upper_runs = encode(&upper);
            let lower_runs = encode(&lower);
            let mut input = format!("{length} {} {}\n", upper_runs.len(), lower_runs.len());
            for (value, count) in upper_runs.into_iter().chain(lower_runs) {
                input.push_str(&format!("{value} {count}\n"));
            }
            let expected = format!(
                "{}\n",
                upper.iter().zip(&lower).filter(|(a, b)| a == b).count()
            );
            // When
            let result = common::run_binary(BIN, &input);
            // Then
            assert_eq!(expected, result);
        }
    }
}

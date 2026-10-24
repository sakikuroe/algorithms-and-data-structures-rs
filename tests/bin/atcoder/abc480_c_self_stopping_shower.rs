//! ABC480 C のバイナリーを、公式サンプルと小さな問題の最適解で検証します。

use super::super::common;

/// ABC480 C の実行バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc480-c-self-stopping-shower");

// 統合テスト: 0 の区間の列挙、範囲での切り取り、給湯時間の計算を検証します。
mod ac_abc480_c_self_stopping_shower {
    use super::*;

    /// Scenario: 公式サンプルの最小操作数を求めます。
    /// - Given: 問題文の公式サンプルがあります。
    /// - When: バイナリーに入力します。
    /// - Then: 最小操作数が公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("3 4 20\n6 7 12\n", "4\n")]
    #[case::sample_2("9 37 480\n12 123 132 213 231 312 321 500 1000\n", "11\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 隣接・重複・対象時刻外の給湯区間を正しく扱います。
    /// - Given: 対象範囲の境界や区間のつながりが異なる入力があります。
    /// - When: バイナリーに入力します。
    /// - Then: 途切れない給湯では 0 回、隙間がある場合は必要回数を返します。
    #[rstest::rstest]
    #[case::fully_covered("1 4 4\n0\n", "0\n")]
    #[case::adjacent("2 4 8\n0 4\n", "0\n")]
    #[case::starts_at_end("1 4 8\n8\n", "2\n")]
    #[case::after_end("1 4 8\n1000000000\n", "2\n")]
    #[case::unsorted_duplicates("4 4 12\n8 0 8 4\n", "0\n")]
    #[case::separated_gaps("2 4 12\n2 8\n", "2\n")]
    fn handles_interval_boundaries(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 小さな入力で、到達時刻に対する動的計画法の最適解と一致します。
    /// - Given: 給湯時間と無料操作の配置が異なる 24 件の入力があります。
    /// - When: 各入力をバイナリーと区間列挙を使わない動的計画法で解きます。
    /// - Then: 両方の最小操作数が一致します。
    #[test]
    fn matches_optimal_reachability_on_small_cases() {
        // Given
        let t = 10;
        for k in 1..=4 {
            for seed in 0..6 {
                let free = (0..5).map(|i| (seed * 3 + i * i) % 14).collect::<Vec<_>>();
                let mut input = format!("{} {k} {t}\n", free.len());
                for a in &free {
                    input.push_str(&format!("{a} "));
                }
                input.push('\n');
                let mut dp = vec![usize::MAX; t + 1];
                dp[0] = 0;
                for reached in 0..t {
                    if dp[reached] == usize::MAX {
                        continue;
                    }
                    let next = (reached + k).min(t);
                    dp[next] = dp[next].min(dp[reached] + 1);
                    for &a in &free {
                        if a <= reached && reached < a + k {
                            let next = (a + k).min(t);
                            dp[next] = dp[next].min(dp[reached]);
                        }
                    }
                }
                let expected = format!("{}\n", dp[t]);
                // When
                let result = common::run_binary(BIN, &input);
                // Then
                assert_eq!(expected, result, "input: {input}");
            }
        }
    }
}

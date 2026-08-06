use super::super::common;

/// `ac-typical90-bs-fuzzy-priority` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-bs-fuzzy-priority");

// ac-typical90-bs-fuzzy-priority のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_bs_fuzzy_priority {
    use super::*;

    /// 出力された各行が、1..=n の順列であり、かつすべての制約 (a は b より前) を
    /// 満たしているかどうかを検証する。
    fn is_valid_permutation_satisfying_constraints(
        line: &str,
        n: usize,
        constraints: &[(u32, u32)],
    ) -> bool {
        let values = line
            .split_whitespace()
            .map(|s| s.parse::<u32>().unwrap())
            .collect::<Vec<u32>>();
        if values.len() != n {
            return false;
        }
        let mut position = vec![0_usize; n + 1];
        let mut seen = vec![false; n + 1];
        for (i, &v) in values.iter().enumerate() {
            if v == 0 || v as usize > n || seen[v as usize] {
                return false;
            }
            seen[v as usize] = true;
            position[v as usize] = i;
        }
        constraints
            .iter()
            .all(|&(a, b)| position[a as usize] < position[b as usize])
    }

    /// Scenario: 制約を満たす順列が存在しない場合は `-1` のみを出力する
    /// - Given: `1 が 3 より前` と `3 が 1 より前` という矛盾する制約を持つ入力である
    /// - When: ac-typical90-bs-fuzzy-priority バイナリへ標準入力として渡す
    /// - Then: `-1` のみが出力される
    #[test]
    fn matches_official_sample_when_impossible() {
        // Given
        let input = "5 2 1\n1 3\n3 1\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!("-1\n", result);
    }

    /// Scenario: 制約を満たす相異なる K 個の順列を出力する
    /// - Given: 問題文の公式サンプル1 (`1 が 2 より前`、`3 が 4 より前`、K=3) である
    /// - When: ac-typical90-bs-fuzzy-priority バイナリへ標準入力として渡す
    /// - Then: 3行の、相異なりすべての制約を満たす順列が出力される (順列自体は
    ///   問題文にある通り複数の正解があり得るため、内容そのものは検証しない)
    #[test]
    fn returns_k_distinct_valid_permutations_for_sample_1() {
        // Given
        let input = "5 2 3\n1 2\n3 4\n";
        let n = 5;
        let constraints = vec![(1, 2), (3, 4)];
        // When
        let result = common::run_binary(BIN, input);
        // Then
        let lines = result.lines().collect::<Vec<&str>>();
        assert_eq!(3, lines.len());
        for line in &lines {
            assert!(is_valid_permutation_satisfying_constraints(
                line,
                n,
                &constraints
            ));
        }
        let unique = lines
            .iter()
            .collect::<std::collections::HashSet<&&str>>();
        assert_eq!(3, unique.len());
    }

    /// Scenario: より多くの制約・より多い K でも、相異なる妥当な順列を出力する
    /// - Given: 問題文の公式サンプル3 (10頂点、15制約、K=10) である
    /// - When: ac-typical90-bs-fuzzy-priority バイナリへ標準入力として渡す
    /// - Then: 10行の、相異なりすべての制約を満たす順列が出力される
    #[test]
    fn returns_k_distinct_valid_permutations_for_sample_3() {
        // Given
        let input = "10 15 10\n8 4\n9 4\n10 2\n6 2\n10 6\n1 3\n7 4\n6 8\n8 1\n5 6\n10 9\n3 7\n8 3\n3 9\n2 3\n";
        let n = 10;
        let constraints = vec![
            (8, 4),
            (9, 4),
            (10, 2),
            (6, 2),
            (10, 6),
            (1, 3),
            (7, 4),
            (6, 8),
            (8, 1),
            (5, 6),
            (10, 9),
            (3, 7),
            (8, 3),
            (3, 9),
            (2, 3),
        ];
        // When
        let result = common::run_binary(BIN, input);
        // Then
        let lines = result.lines().collect::<Vec<&str>>();
        assert_eq!(10, lines.len());
        for line in &lines {
            assert!(is_valid_permutation_satisfying_constraints(
                line,
                n,
                &constraints
            ));
        }
        let unique = lines
            .iter()
            .collect::<std::collections::HashSet<&&str>>();
        assert_eq!(10, unique.len());
    }
}

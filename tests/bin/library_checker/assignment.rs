use super::super::common;

/// `lc-assignment` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-assignment");

// lc-assignment のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_assignment {
    use super::*;

    /// Scenario: 出力された割り当てが、実際に最小コストの順列になっている
    /// - Given: 問題文の公式サンプルである
    /// - When: lc-assignment バイナリへ標準入力として渡す
    /// - Then: 出力されたコストが期待値 (9) と一致し、出力された `p_i` が
    ///   `0..n` の順列をなし、その順列で行列を評価した合計が出力された
    ///   コストと一致する
    #[test]
    fn produces_permutation_matching_minimum_cost() {
        // Given
        let input = "3\n4 3 5\n3 5 9\n4 1 4\n";
        let matrix = [[4, 3, 5], [3, 5, 9], [4, 1, 4]];
        let expected_cost = 9;
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();

        let cost = tokens.next().unwrap().parse::<i64>().unwrap();
        assert_eq!(expected_cost, cost);

        let n = matrix.len();
        let assignment = (0..n)
            .map(|_| tokens.next().unwrap().parse::<usize>().unwrap())
            .collect::<Vec<usize>>();
        assert!(tokens.next().is_none());

        // p が 0..n の順列であること (同じ列が2回使われていないこと) を、
        // 出現した列を記録して確認する。
        let mut used_columns = vec![false; n];
        for &p in &assignment {
            assert!(p < n, "column index {p} out of range");
            assert!(!used_columns[p], "column {p} is used more than once");
            used_columns[p] = true;
        }

        let actual_cost = (0..n).map(|i| matrix[i][assignment[i]]).sum::<i64>();
        assert_eq!(expected_cost, actual_cost);
    }
}

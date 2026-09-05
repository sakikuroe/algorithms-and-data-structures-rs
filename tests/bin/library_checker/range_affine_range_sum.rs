use super::super::common;

/// `lc-range-affine-range-sum` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-range-affine-range-sum");

// lc-range-affine-range-sum のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_range_affine_range_sum {
    use super::*;

    /// Scenario: Library Checker の公式サンプルを解いたときの標準出力を検証する。
    /// - Given: N=5, Q=7 で、初期配列が [1, 2, 3, 4, 5] である。区間一次変換
    ///   クエリと区間和クエリが混在する。
    /// - When: lc-range-affine-range-sum バイナリへ標準入力として渡す。
    /// - Then: 各区間和クエリの結果が期待値と一致する。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "\
5 7
1 2 3 4 5
1 0 5
0 2 4 100 101
1 0 3
0 1 3 102 103
1 2 5
0 2 5 104 105
1 0 5
";
        let expected = "\
15
404
41511
4317767
";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

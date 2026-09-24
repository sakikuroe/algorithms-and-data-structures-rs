use super::super::common;

/// `lc-range-kth-smallest` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-range-kth-smallest");

// lc-range-kth-smallest のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_range_kth_smallest {
    use super::*;

    /// Scenario: Library Checker の公式サンプルを解いたときの標準出力を検証する。
    /// - Given: 数列 `[1, 4, 0, 1, 3]` がある。
    /// - When: lc-range-kth-smallest バイナリへ公式サンプルを標準入力として渡す。
    /// - Then: 各クエリの k 番目に小さい値が出力される。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "5 3\n1 4 0 1 3\n0 5 2\n1 3 1\n3 4 0\n";
        let expected = "1\n4\n1\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 単一要素と重複値のみの境界値を検証する。
    /// - Given: 要素数 1 の数列と、全要素が等しい数列がある。
    /// - When: lc-range-kth-smallest バイナリへ境界値クエリを渡す。
    /// - Then: k = 0 および最大の k で正しい値を返す。
    #[test]
    fn handles_single_element_and_duplicates() {
        // Given
        let input = "1 2\n7\n0 1 0\n0 1 0\n";
        let expected = "7\n7\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);

        // Given
        let input = "4 2\n5 5 5 5\n0 4 3\n2 4 0\n";
        let expected = "5\n5\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

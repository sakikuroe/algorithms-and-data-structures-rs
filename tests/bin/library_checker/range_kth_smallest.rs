use super::super::common;

/// `lc-range-kth-smallest` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-range-kth-smallest");

// lc-range-kth-smallest のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_range_kth_smallest {
    use super::*;

    /// 状況: Library Checker の公式サンプルを解いたときの標準出力を検証する。
    /// - 前提: 数列 `[1, 4, 0, 1, 3]` がある。
    /// - 操作: lc-range-kth-smallest バイナリへ公式サンプルを標準入力として渡す。
    /// - 結果: 各クエリの k 番目に小さい値が出力される。
    #[test]
    fn matches_official_sample() {
        // 前提
        let input = "5 3\n1 4 0 1 3\n0 5 2\n1 3 1\n3 4 0\n";
        let expected = "1\n4\n1\n";
        // 操作
        let result = common::run_binary(BIN, input);
        // 結果
        assert_eq!(expected, result);
    }

    /// 状況: 単一要素と重複値のみの境界値を検証する。
    /// - 前提: 要素数 1 の数列と、全要素が等しい数列がある。
    /// - 操作: lc-range-kth-smallest バイナリへ境界値クエリを渡す。
    /// - 結果: k = 0 および最大の k で正しい値を返す。
    #[test]
    fn handles_single_element_and_duplicates() {
        // 前提
        let input = "1 2\n7\n0 1 0\n0 1 0\n";
        let expected = "7\n7\n";
        // 操作
        let result = common::run_binary(BIN, input);
        // 結果
        assert_eq!(expected, result);

        // 前提
        let input = "4 2\n5 5 5 5\n0 4 3\n2 4 0\n";
        let expected = "5\n5\n";
        // 操作
        let result = common::run_binary(BIN, input);
        // 結果
        assert_eq!(expected, result);
    }
}

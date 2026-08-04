use super::super::common;

/// `lc-vertex-add-path-sum` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-vertex-add-path-sum");

// lc-vertex-add-path-sum のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_vertex_add_path_sum {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - Given: 問題文の公式サンプルである (パス総和クエリと、途中に加算
    ///   クエリを挟むケースを含む)
    /// - When: lc-vertex-add-path-sum バイナリへ標準入力として渡す
    /// - Then: 各パス総和クエリの結果が期待値と一致する
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "5 5\n1 10 100 1000 10000\n0 1\n1 2\n2 3\n1 4\n1 0 3\n1 2 4\n0 1 100000\n1 1 3\n1 3 2\n";
        let expected = "1111\n10110\n101110\n1100\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

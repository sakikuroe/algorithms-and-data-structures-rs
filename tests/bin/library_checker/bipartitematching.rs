use std::collections;

use super::super::common;

/// `lc-bipartitematching` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-bipartitematching");

// lc-bipartitematching のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_bipartitematching {
    use super::*;

    /// Scenario: 出力されたマッチングが、実際に矛盾のない最大マッチングに
    /// なっている
    /// - Given: 問題文の公式サンプルである
    /// - When: lc-bipartitematching バイナリへ標準入力として渡す
    /// - Then: 出力された本数が期待値 (3) と一致し、各辺が入力に存在する
    ///   辺であり、かつ左右どちらの頂点も2回以上使われていない
    #[test]
    fn produces_valid_maximum_matching() {
        // Given
        let input = "4 4 7\n1 1\n2 2\n0 0\n3 1\n1 2\n2 0\n3 2\n";
        let edges = [(1, 1), (2, 2), (0, 0), (3, 1), (1, 2), (2, 0), (3, 2)];
        let expected_size = 3;
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();

        let size = tokens.next().unwrap().parse::<usize>().unwrap();
        assert_eq!(expected_size, size);

        let mut used_left = collections::HashSet::new();
        let mut used_right = collections::HashSet::new();
        for _ in 0..size {
            let u = tokens.next().unwrap().parse::<usize>().unwrap();
            let v = tokens.next().unwrap().parse::<usize>().unwrap();
            assert!(
                edges.contains(&(u, v)),
                "({u}, {v}) is not an edge of the input graph"
            );
            assert!(
                used_left.insert(u),
                "left vertex {u} is used more than once"
            );
            assert!(
                used_right.insert(v),
                "right vertex {v} is used more than once"
            );
        }
        assert!(tokens.next().is_none());
    }
}

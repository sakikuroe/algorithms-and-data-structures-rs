use super::super::common;

/// `lc-scc` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-scc");

// lc-scc のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_scc {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの出力が、各辺の順序制約
    /// (辺 (a, b) について、b を含む行が a を含む行より前に来てはならない) を
    /// 満たす
    /// - Given: 問題文の公式サンプルの入力である
    /// - When: lc-scc バイナリへ標準入力として渡す
    /// - Then: 出力されたグループ分けが、頂点数・辺の順序制約をすべて満たす
    #[test]
    fn produces_grouping_respecting_edge_order_constraint() {
        // Given
        let input = "6 7\n1 4\n5 2\n3 0\n5 5\n4 1\n0 3\n4 2\n";
        let edges = [(1, 4), (5, 2), (3, 0), (5, 5), (4, 1), (0, 3), (4, 2)];
        // When
        let output = common::run_binary(BIN, input);
        // Then
        // 出力は値ごとに改行されるが、testlib ベースのチェッカーと同様に
        // トークン単位 (空白・改行を区別しない) で読み取る。
        let mut tokens = output
            .split_whitespace()
            .map(|s| s.parse::<usize>().unwrap());
        let k = tokens.next().unwrap();
        let mut position = [usize::MAX; 6];
        for group_idx in 0..k {
            let l = tokens.next().unwrap();
            for _ in 0..l {
                let v = tokens.next().unwrap();
                assert_eq!(usize::MAX, position[v], "vertex {v} used twice");
                position[v] = group_idx;
            }
        }
        assert!(tokens.next().is_none(), "unexpected trailing tokens");
        assert!(
            position.iter().all(|&p| p != usize::MAX),
            "every vertex must appear exactly once"
        );
        for (a, b) in edges {
            assert!(
                position[a] <= position[b],
                "edge ({a}, {b}) violates topological order"
            );
        }
    }
}

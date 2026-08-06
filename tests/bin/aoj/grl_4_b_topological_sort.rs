use super::super::common;

/// `aoj-grl-4-b-topological-sort` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_aoj-grl-4-b-topological-sort");

// aoj-grl-4-b-topological-sort のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod aoj_grl_4_b_topological_sort {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたとき、出力された頂点列がすべての
    /// 辺の順序制約 (辺 `s -> t` について、`s` が `t` より前に現れる) を満たす。
    /// トポロジカルソートは複数の正しい順序を持ちうるため、公式サンプルの出力
    /// 文字列そのものと比較するのではなく、この制約を満たすかどうかで検証する
    /// (`lc-scc` のテストと同じ方針)。
    /// - Given: 問題文の公式サンプルの入力である。
    /// - When: aoj-grl-4-b-topological-sort バイナリへ標準入力として渡す。
    /// - Then: 出力された頂点列が、頂点数・辺の順序制約をすべて満たす。
    #[test]
    fn produces_order_respecting_all_edges_for_official_sample() {
        // Given
        let input = "6 6\n0 1\n1 2\n3 1\n3 4\n4 5\n5 2\n";
        let edges = [(0, 1), (1, 2), (3, 1), (3, 4), (4, 5), (5, 2)];
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let order: Vec<usize> = output
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(6, order.len());

        let mut position = [usize::MAX; 6];
        for (i, &v) in order.iter().enumerate() {
            assert_eq!(usize::MAX, position[v], "vertex {v} used twice");
            position[v] = i;
        }
        assert!(
            position.iter().all(|&p| p != usize::MAX),
            "every vertex must appear exactly once"
        );
        for (s, t) in edges {
            assert!(
                position[s] < position[t],
                "edge ({s}, {t}) violates topological order"
            );
        }
    }
}

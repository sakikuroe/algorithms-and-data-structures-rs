use super::super::common;
use anmitsu::ds::union_find;

/// `lc-minimum-spanning-tree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-minimum-spanning-tree");

// lc-minimum-spanning-tree のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_minimum_spanning_tree {
    use super::*;
    use rstest::rstest;

    /// Scenario: 出力された辺の一覧が、実際に最小重みの全域木をなす
    /// - Given: 問題文の公式サンプルである (自己ループを含むケース、
    ///   最初から木をなすケース、重みが `u32` の範囲を超えて合計されるケースの
    ///   3通り)
    /// - When: lc-minimum-spanning-tree バイナリへ標準入力として渡す
    /// - Then: 出力された合計重みが期待値と一致し、出力された辺番号が
    ///   ちょうど N-1 本あり、それらが全頂点を連結する木をなし、かつ
    ///   その辺の重みの合計が出力された合計重みと一致する
    #[rstest]
    // - Given: 自己ループ (辺6) を含み、他の辺の重みがすべて異なるグラフである
    #[case::sample_1_with_self_loop(
        "4 7\n0 1 4\n0 2 2\n0 3 3\n1 2 6\n1 3 8\n2 3 1\n1 1 0\n",
        4,
        vec![(0, 1, 4_u64), (0, 2, 2), (0, 3, 3), (1, 2, 6), (1, 3, 8), (2, 3, 1), (1, 1, 0)],
        7_u64,
    )]
    // - Given: 辺数がちょうど N-1 本であり、すでに木をなすグラフである
    #[case::sample_2_already_a_tree(
        "4 3\n0 1 1\n1 2 2\n3 1 3\n",
        4,
        vec![(0, 1, 1_u64), (1, 2, 2), (3, 1, 3)],
        6_u64,
    )]
    // - Given: 辺の重みが大きく、合計重みが u32 の範囲を超えるグラフである
    #[case::sample_3_total_weight_exceeds_u32(
        "6 5\n0 1 1000000000\n1 2 1000000000\n2 3 1000000000\n3 4 1000000000\n4 5 1000000000\n",
        6,
        vec![
            (0, 1, 1_000_000_000_u64),
            (1, 2, 1_000_000_000),
            (2, 3, 1_000_000_000),
            (3, 4, 1_000_000_000),
            (4, 5, 1_000_000_000),
        ],
        5_000_000_000_u64,
    )]
    fn produces_valid_minimum_spanning_tree(
        #[case] input: &str,
        #[case] n: usize,
        #[case] edges: Vec<(usize, usize, u64)>,
        #[case] expected_weight: u64,
    ) {
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();

        let total_weight = tokens.next().unwrap().parse::<u64>().unwrap();
        assert_eq!(expected_weight, total_weight);

        let mut uf = union_find::UnionFind::new(n);
        let mut sum = 0_u64;
        let mut used_edge_count = 0;
        for _ in 0..n.saturating_sub(1) {
            let e = tokens.next().unwrap().parse::<usize>().unwrap();
            let (a, b, w) = edges[e];
            assert!(
                !uf.is_same(a, b),
                "edge {e} does not connect a new component"
            );
            uf.union(a, b);
            sum += w;
            used_edge_count += 1;
        }
        assert_eq!(n.saturating_sub(1), used_edge_count);
        assert_eq!(expected_weight, sum);
        for v in 1..n {
            assert!(uf.is_same(0, v), "vertex {v} is not connected to vertex 0");
        }
        assert!(tokens.next().is_none());
    }
}

use super::super::common;

/// `lc-tree-diameter` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-tree-diameter");

// lc-tree-diameter のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_tree_diameter {
    use super::*;
    use std::collections::HashMap;

    /// Scenario: 直径だけでなく、出力された経路自体が正当であることを
    /// 検証する (この問題は特別ジャッジであり、経路は一意とは限らない)
    /// - Given: 0-1 (重み2), 1-2 (重み1), 1-3 (重み3) の木がある
    ///   (直径は頂点0-3間の5)
    /// - When: lc-tree-diameter バイナリへ標準入力として渡す
    /// - Then: 直径の値が5になり、出力された経路が実在する辺のみを辿り、
    ///   同じ頂点を二度訪れず、辺の重みの総和が直径と一致する
    #[test]
    fn outputs_valid_path_matching_diameter() {
        // Given
        let input = "4\n0 1 2\n1 2 1\n1 3 3\n";
        let edges: HashMap<(u64, u64), u64> =
            HashMap::from([((0, 1), 2), ((1, 2), 1), ((1, 3), 3)]);
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace().map(|s| s.parse::<u64>().unwrap());
        let diameter = tokens.next().unwrap();
        assert_eq!(5, diameter);

        let length = tokens.next().unwrap();
        let path = tokens.by_ref().take(length as usize).collect::<Vec<u64>>();
        assert!(tokens.next().is_none(), "unexpected trailing tokens");
        assert_eq!(length as usize, path.len());

        let mut visited = std::collections::HashSet::new();
        let mut cost = 0;
        for w in path.windows(2) {
            let (u, v) = (w[0].min(w[1]), w[0].max(w[1]));
            let weight = edges
                .get(&(u, v))
                .unwrap_or_else(|| panic!("edge ({u}, {v}) does not exist"));
            cost += weight;
        }
        for &v in &path {
            assert!(visited.insert(v), "vertex {v} visited twice");
        }
        assert_eq!(diameter, cost, "path weight does not match the diameter");
    }
}

use super::super::common;

/// `lc-eulerian-trail-directed` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-eulerian-trail-directed");

// lc-eulerian-trail-directed のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_eulerian_trail_directed {
    use super::*;

    /// Scenario: 各ケースについて、出力された経路がすべての辺をちょうど1回
    /// ずつ使い、頂点列と辺列が整合している
    /// - Given: 問題文の公式サンプル (3ケース) である
    /// - When: lc-eulerian-trail-directed バイナリへ標準入力として渡す
    /// - Then: `Yes` の場合は経路が全辺を1回ずつ正しく使い、`No` の場合は
    ///   3ケース目までのトークン数が入力と整合する
    #[test]
    fn produces_valid_trail_using_every_edge_exactly_once() {
        // Given
        let input = "3\n4 7\n0 1\n2 0\n0 2\n3 0\n1 3\n2 3\n3 3\n4 6\n0 1\n2 0\n0 3\n1 2\n3 1\n2 3\n6 10\n0 3\n1 2\n4 0\n5 1\n4 4\n2 3\n3 1\n3 2\n1 4\n1 5\n";
        let cases = [
            (
                4,
                vec![(0, 1), (2, 0), (0, 2), (3, 0), (1, 3), (2, 3), (3, 3)],
            ),
            (4, vec![(0, 1), (2, 0), (0, 3), (1, 2), (3, 1), (2, 3)]),
            (
                6,
                vec![
                    (0, 3),
                    (1, 2),
                    (4, 0),
                    (5, 1),
                    (4, 4),
                    (2, 3),
                    (3, 1),
                    (3, 2),
                    (1, 4),
                    (1, 5),
                ],
            ),
        ];
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut tokens = output.split_whitespace();
        for (_n, edges) in cases {
            let m = edges.len();
            match tokens.next().unwrap() {
                "No" => {}
                "Yes" => {
                    let vertices = (0..=m)
                        .map(|_| tokens.next().unwrap().parse::<usize>().unwrap())
                        .collect::<Vec<usize>>();
                    let mut used = vec![false; m];
                    for i in 0..m {
                        let e = tokens.next().unwrap().parse::<usize>().unwrap();
                        assert!(!used[e], "edge {e} used twice");
                        used[e] = true;
                        assert_eq!(edges[e], (vertices[i], vertices[i + 1]));
                    }
                    assert!(
                        used.iter().all(|&u| u),
                        "every edge must be used exactly once"
                    );
                }
                other => panic!("unexpected token: {other}"),
            }
        }
        assert!(tokens.next().is_none());
    }
}

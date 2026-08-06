use super::super::common;

/// `ac-practice2-d-maxflow` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-practice2-d-maxflow");

// ac-practice2-d-maxflow のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_practice2_d_maxflow {
    use super::*;

    /// Scenario: 出力されたドミノの配置が、実際に矛盾なく敷き詰められている
    /// - Given: 問題文の公式サンプルである
    /// - When: ac-practice2-d-maxflow バイナリへ標準入力として渡す
    /// - Then: 出力された枚数が期待値 (3) と一致し、盤面の各ドミノが
    ///   隣接する2マスの組として正しく対応しており、障害物 `#` の位置も
    ///   保たれている
    #[test]
    fn produces_valid_domino_tiling() {
        // Given
        let input = "3 3\n#..\n..#\n...\n";
        let grid = ["#..", "..#", "..."];
        let expected_count = 3;
        // When
        let output = common::run_binary(BIN, input);
        // Then
        let mut lines = output.lines();

        let count = lines.next().unwrap().parse::<usize>().unwrap();
        assert_eq!(expected_count, count);

        let board = lines
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect::<Vec<_>>();
        assert_eq!(grid.len(), board.len());

        let n = board.len();
        let m = board[0].len();
        let mut placed_count = 0;
        for (i, row) in board.iter().enumerate() {
            assert_eq!(m, row.len());
            for (j, &ch) in row.iter().enumerate() {
                let original = grid[i].as_bytes()[j] as char;
                if original == '#' {
                    assert_eq!('#', ch, "obstacle at ({i}, {j}) must be preserved");
                    continue;
                }

                match ch {
                    '.' => {}
                    '>' => {
                        assert!(j + 1 < m, "'>' at ({i}, {j}) has no right neighbor");
                        assert_eq!(
                            '<',
                            board[i][j + 1],
                            "'>' at ({i}, {j}) is not paired with '<'"
                        );
                        placed_count += 1;
                    }
                    '<' => {
                        assert!(j >= 1, "'<' at ({i}, {j}) has no left neighbor");
                        assert_eq!(
                            '>',
                            board[i][j - 1],
                            "'<' at ({i}, {j}) is not paired with '>'"
                        );
                    }
                    'v' => {
                        assert!(i + 1 < n, "'v' at ({i}, {j}) has no neighbor below");
                        assert_eq!(
                            '^',
                            board[i + 1][j],
                            "'v' at ({i}, {j}) is not paired with '^'"
                        );
                        placed_count += 1;
                    }
                    '^' => {
                        assert!(i >= 1, "'^' at ({i}, {j}) has no neighbor above");
                        assert_eq!(
                            'v',
                            board[i - 1][j],
                            "'^' at ({i}, {j}) is not paired with 'v'"
                        );
                    }
                    other => panic!("unexpected character '{other}' at ({i}, {j})"),
                }
            }
        }
        assert_eq!(expected_count, placed_count);
    }
}

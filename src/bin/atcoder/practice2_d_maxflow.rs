// AtCoder: Library Practice Contest D - Maxflow
// https://atcoder.jp/contests/practice2/tasks/practice2_d

use anmitsu::graph::flow_graph::FlowGraph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let grid = (0..n).map(|_| io.chars()).collect::<Vec<Vec<char>>>();
    let cell_id = |i: usize, j: usize| i * m + j;
    let source = n * m;
    let sink = n * m + 1;

    // 市松模様に2色 (黒: i+j が偶数, 白: i+j が奇数) へ塗り分けると、隣接する
    // マスは必ず異なる色になる。黒マスを始点側、白マスを終点側とみなし、
    // 隣接するマス同士に容量1の辺を張れば、ドミノを1枚も重ねずに敷き詰める
    // 問題が、二部マッチング (最大流) に帰着できる。
    let mut g = FlowGraph::<i64>::new(n * m + 2);

    // ドミノの配置を復元できるよう、黒マスから白マスへの辺の番号と、両端の
    // マスの座標を記録しておく。
    let mut domino_edges = Vec::new();
    for i in 0..n {
        for j in 0..m {
            if grid[i][j] == '#' {
                continue;
            }
            if (i + j) % 2 == 0 {
                g.add_edge(source, cell_id(i, j), 1);
            } else {
                g.add_edge(cell_id(i, j), sink, 1);
            }
            // 右隣・下隣のみを見ることで、隣接するマスの組をちょうど1回だけ
            // 辺として張る。どちらの色が (i, j) 側かは色によって変わるため、
            // 黒マス側から白マス側へ向くように毎回向きを決め直す。
            for (ni, nj) in [(i, j + 1), (i + 1, j)] {
                if ni < n && nj < m && grid[ni][nj] != '#' {
                    let (black, white) = if (i + j) % 2 == 0 {
                        (cell_id(i, j), cell_id(ni, nj))
                    } else {
                        (cell_id(ni, nj), cell_id(i, j))
                    };
                    let id = g.add_edge(black, white, 1);
                    domino_edges.push((id, i, j, ni, nj));
                }
            }
        }
    }

    let max_flow = g.max_flow(source, sink);

    // 流量が1になった辺が、実際に置かれたドミノに対応する。
    let mut output = grid;
    for (id, i, j, ni, nj) in domino_edges {
        let (_, _, _, flow) = g.get_edge(id);
        if flow == 1 {
            if ni == i {
                // 横向きのドミノ: 左マスに '>'、右マスに '<' を置く。
                output[i][j] = '>';
                output[ni][nj] = '<';
            } else {
                // 縦向きのドミノ: 上マスに 'v'、下マスに '^' を置く。
                output[i][j] = 'v';
                output[ni][nj] = '^';
            }
        }
    }

    io.writeln(max_flow);
    for row in output {
        for ch in row {
            io.write(ch);
        }
        io.write('\n');
    }

    io.flush();
}

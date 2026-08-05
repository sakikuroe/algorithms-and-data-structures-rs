// AtCoder: ABC176 D - Wizard in Maze
// https://atcoder.jp/contests/abc176/tasks/abc176_d

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let h = io.u32() as usize;
    let w = io.u32() as usize;
    let ch = io.usize1();
    let cw = io.usize1();
    let dh = io.usize1();
    let dw = io.usize1();

    let grid = (0..h).map(|_| io.chars()).collect::<Vec<Vec<char>>>();

    // マス (r, c) を頂点番号 r * w + c として、通常移動と魔法によるワープを
    // それぞれ辺として持つグラフを構築する。
    let mut g = Graph::new(h * w);
    for r in 0..h {
        for c in 0..w {
            // 通常移動: 上下左右に隣接し、壁でないマスへコスト0で移動できる。
            for (dr, dc) in [(-1_i32, 0_i32), (1, 0), (0, -1), (0, 1)] {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr < 0 || nr >= h as i32 || nc < 0 || nc >= w as i32 {
                    continue;
                }
                let (nr, nc) = (nr as usize, nc as usize);
                if grid[nr][nc] == '.' {
                    g.add_edge(r * w + c, nr * w + nc, false);
                }
            }

            // 魔法によるワープ: チェビシェフ距離2以内 (自身を除く5x5の範囲) の、
            // 盤面内かつ壁でないマスへコスト1で移動できる。通常移動と同様、
            // 到達先が壁でないことは要求されるが、途中の経路上の壁は無視できる。
            for dr in -2_i32..=2 {
                for dc in -2_i32..=2 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr < 0 || nr >= h as i32 || nc < 0 || nc >= w as i32 {
                        continue;
                    }
                    let (nr, nc) = (nr as usize, nc as usize);
                    if grid[nr][nc] == '.' {
                        g.add_edge(r * w + c, nr * w + nc, true);
                    }
                }
            }
        }
    }

    let result = g.zero_one_bfs(&[ch * w + cw], |&is_one| is_one);
    let ans = result.distance(dh * w + dw).map(|d| d as i64).unwrap_or(-1);
    io.writeln(ans);

    io.flush();
}

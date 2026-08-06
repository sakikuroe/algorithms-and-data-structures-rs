// AtCoder: Typical90 043 - Maze Challenge with Lack of Sleep
// https://atcoder.jp/contests/typical90/tasks/typical90_aq

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let h = io.u32() as usize;
    let w = io.u32() as usize;
    let rs = io.usize1();
    let cs = io.usize1();
    let rt = io.usize1();
    let ct = io.usize1();
    let grid = (0..h).map(|_| io.chars()).collect::<Vec<Vec<char>>>();

    // 頂点 (r*w+c)*4+d は「マス (r,c) に、直前の移動方向 d のまま到着した」
    // 状態を表す。頂点 h*w*4 は仮想始点であり、最初の1歩をコスト0で選べる
    // ようにするために使う。
    let dirs: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let start_vertex = h * w * 4;
    let mut g = Graph::new(start_vertex + 1);

    let idx = |r: usize, c: usize, d: usize| (r * w + c) * 4 + d;
    let step = |r: usize, c: usize, dr: i32, dc: i32| -> Option<(usize, usize)> {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr < 0 || nr >= h as i32 || nc < 0 || nc >= w as i32 {
            return None;
        }
        let (nr, nc) = (nr as usize, nc as usize);
        if grid[nr][nc] == '#' {
            return None;
        }
        Some((nr, nc))
    };

    for (r, row) in grid.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == '#' {
                continue;
            }
            for (d, &(dr, dc)) in dirs.iter().enumerate() {
                let Some((nr, nc)) = step(r, c, dr, dc) else {
                    continue;
                };
                // 同じ方向へ直進するのは方向転換ではないため、コスト0。
                g.add_edge(idx(r, c, d), idx(nr, nc, d), false);
            }
            // 現在地から、直前の方向によらず改めて別の方向へ進み出すのは
            // 方向転換1回分なので、コスト1。到着状態の direction (d) は
            // ここでは使わないため、d=0..3 のどの頂点からでも同じ辺を張る。
            for (d2, &(dr2, dc2)) in dirs.iter().enumerate() {
                let Some((nr2, nc2)) = step(r, c, dr2, dc2) else {
                    continue;
                };
                for d in 0..4 {
                    if d == d2 {
                        continue;
                    }
                    g.add_edge(idx(r, c, d), idx(nr2, nc2, d2), true);
                }
            }
        }
    }

    // 出発地点からの最初の一歩は、まだ一度も移動していないため
    // 方向転換として数えない。
    for (d, &(dr, dc)) in dirs.iter().enumerate() {
        if let Some((nr, nc)) = step(rs, cs, dr, dc) {
            g.add_edge(start_vertex, idx(nr, nc, d), false);
        }
    }

    let result = g.zero_one_bfs(&[start_vertex], |&is_one| is_one);
    let ans = (0..4)
        .filter_map(|d| result.distance(idx(rt, ct, d)))
        .min()
        .unwrap();
    io.writeln(ans as u32);

    io.flush();
}

// AtCoder: ABC193-F - Zebraness
// https://atcoder.jp/contests/abc193/tasks/abc193_f

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let grid = (0..n).map(|_| io.chars()).collect::<Vec<Vec<char>>>();

    let cell = |r: usize, c: usize| r * n + c;
    let n_cells = n * n;
    let n_edges = 2 * n * n.saturating_sub(1);
    let mut psp = ProjectSelection::<i64>::new(n_cells + 2 * n_edges);
    let mut next_aux = n_cells;

    // グリッド上で隣接する2マスは、必ずマス目の (行番号+列番号) の偶奇が
    // 異なる。そこで各マスに y = (色) XOR (行番号+列番号の偶奇) という値を
    // 割り当てると、隣接する2マスの色が異なることと y が等しいことが同値に
    // なる (偶奇の差がちょうど反転を打ち消すため)。この y を「選ぶ/選ばない」
    // の2値とみなし、色が確定しているマスは force_selected/force_unselected
    // で固定する。
    for (r, row) in grid.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == '?' {
                continue;
            }
            let color = usize::from(ch == 'B');
            let parity = (r + c) % 2;
            if color ^ parity == 1 {
                psp.force_selected(cell(r, c));
            } else {
                psp.force_unselected(cell(r, c));
            }
        }
    }

    let mut add_edge_bonus = |psp: &mut ProjectSelection<i64>, u: usize, v: usize| {
        let aux1 = next_aux;
        next_aux += 1;
        let aux2 = next_aux;
        next_aux += 1;
        // y が両方1、または両方0 (すなわち色が異なる) であれば、しまうま度が
        // 1 増える。
        psp.add_and_bonus_when_selected(&[u, v], 1, aux1);
        psp.add_and_bonus_when_unselected(&[u, v], 1, aux2);
    };
    for r in 0..n {
        for c in 0..n.saturating_sub(1) {
            add_edge_bonus(&mut psp, cell(r, c), cell(r, c + 1));
        }
    }
    for r in 0..n.saturating_sub(1) {
        for c in 0..n {
            add_edge_bonus(&mut psp, cell(r, c), cell(r + 1, c));
        }
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

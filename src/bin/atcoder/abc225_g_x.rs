// AtCoder: ABC225-G - X
// https://atcoder.jp/contests/abc225/tasks/abc225_g

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let h = io.u32() as usize;
    let w = io.u32() as usize;
    let c = io.i64();
    let a = (0..h)
        .map(|_| (0..w).map(|_| io.i64()).collect::<Vec<i64>>())
        .collect::<Vec<Vec<i64>>>();

    let cell = |r: usize, cl: usize| r * w + cl;
    let n_cells = h * w;
    // 斜めに隣接するマスの組ごとに、線分を共有できた場合の得を表す補助頂点を
    // 1つずつ使う。組の総数は高々 2 * (h-1) * (w-1) である。
    let n_aux = 2 * h.saturating_sub(1) * w.saturating_sub(1);
    let mut psp = ProjectSelection::<i64>::new(n_cells + n_aux);
    let mut next_aux = n_cells;

    for (r, row) in a.iter().enumerate() {
        for (cl, &value) in row.iter().enumerate() {
            // マスを選ぶ (バツ印を付ける) と、そのマスの値を得られる代わりに、
            // 自分自身の左上-右下・右上-左下の2本の線分の分だけコストがかかる。
            psp.add_weight(cell(r, cl), value - 2 * c);
        }
    }
    for r in 0..h.saturating_sub(1) {
        for cl in 0..w.saturating_sub(1) {
            // "\" 方向に隣接するマスの左上-右下の線分は、両方選ばれていれば
            // 1本にまとめて描けるため、その分のコスト c が浮く。
            let aux = next_aux;
            next_aux += 1;
            psp.add_and_bonus_when_selected(&[cell(r, cl), cell(r + 1, cl + 1)], c, aux);
        }
    }
    for r in 0..h.saturating_sub(1) {
        for cl in 1..w {
            // "/" 方向に隣接するマスの右上-左下の線分も、同様にまとめられる。
            let aux = next_aux;
            next_aux += 1;
            psp.add_and_bonus_when_selected(&[cell(r, cl), cell(r + 1, cl - 1)], c, aux);
        }
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

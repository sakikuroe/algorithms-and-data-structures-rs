// AtCoder: ABC326-G - Unlock Achievement
// https://atcoder.jp/contests/abc326/tasks/abc326_g

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let c = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();
    let a = (0..m).map(|_| io.i64()).collect::<Vec<i64>>();

    // スキル j をレベル k (2..=5) まで上げた状態を表す頂点。レベル1は
    // 何もしなくても最初から達成しているため、頂点を用意しない。
    let skill_vertex = |j: usize, k: usize| j * 4 + (k - 2);
    let n_skill_vertices = n * 4;

    let mut psp = ProjectSelection::<i64>::new(n_skill_vertices + m);
    for (j, &cj) in c.iter().enumerate() {
        for k in 2..=5 {
            // レベル k まで上げるには、レベル k-1 からレベル k へ上げる分の
            // コスト c[j] が必ずかかる。
            psp.add_weight(skill_vertex(j, k), -cj);
        }
        for k in 3..=5 {
            // レベル k まで上げているなら、レベル k-1 までも上げていなければ
            // ならない。
            psp.add_constraint(skill_vertex(j, k), skill_vertex(j, k - 1));
        }
    }

    let l = (0..m)
        .map(|_| (0..n).map(|_| io.u32() as usize).collect::<Vec<usize>>())
        .collect::<Vec<Vec<usize>>>();

    for (i, row) in l.iter().enumerate() {
        // レベル1以上の条件は最初から満たされているため、レベル2以上を
        // 要求する条件だけを集める。
        let required = (0..n)
            .filter(|&j| row[j] > 1)
            .map(|j| skill_vertex(j, row[j]))
            .collect::<Vec<usize>>();
        psp.add_and_bonus_when_selected(&required, a[i], n_skill_vertices + i);
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

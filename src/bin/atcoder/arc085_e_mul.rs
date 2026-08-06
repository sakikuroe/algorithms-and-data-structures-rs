// AtCoder: ARC085-E - MUL
// https://atcoder.jp/contests/arc085/tasks/arc085_c

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let a = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();

    let mut psp = ProjectSelection::<i64>::new(n);
    for (i, &ai) in a.iter().enumerate() {
        psp.add_weight(i, ai);
    }
    for j in 2..=n {
        for i in 1..j {
            if j % i == 0 {
                // 宝石 j (1-indexed) が生き残るなら、その約数 i も生き残って
                // いなければならない (i を割る操作を行うと j も同時に割れて
                // しまうため)。
                psp.add_constraint(j - 1, i - 1);
            }
        }
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

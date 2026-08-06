// AtCoder: Typical90 040 - Get More Money
// https://atcoder.jp/contests/typical90/tasks/typical90_an

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let w = io.i64();
    let a = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();

    let mut psp = ProjectSelection::<i64>::new(n);
    for (i, &ai) in a.iter().enumerate() {
        // 家 i に入ると現金 ai を得るが、必ず料金 w を支払う。
        psp.add_weight(i, ai - w);
    }
    for i in 0..n {
        let k = io.u32() as usize;
        for _ in 0..k {
            let c = io.usize1();
            // 家 c に入るには、家 c の鍵を持つ家 i にも入っていなければならない。
            psp.add_constraint(c, i);
        }
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

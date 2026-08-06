// Codeforces: 1082G - Petya and Graph
// https://codeforces.com/problemset/problem/1082/G

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let a = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();

    let mut psp = ProjectSelection::<i64>::new(n + m);
    for (i, &ai) in a.iter().enumerate() {
        // 頂点 i を部分グラフに使うと、その分の重み ai を失う。
        psp.add_weight(i, -ai);
    }
    for i in 0..m {
        let u = io.usize1();
        let v = io.usize1();
        let w = io.i64();
        // 辺 i は、両端点をともに部分グラフに使った場合にのみ、その重み w を得る。
        psp.add_and_bonus_when_selected(&[u, v], w, n + i);
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

// yukicoder: No.2713 Just Solitaire
// https://yukicoder.me/problems/no/2713

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let a = (0..n).map(|_| io.i64()).collect::<Vec<i64>>();
    let b = (0..m).map(|_| io.i64()).collect::<Vec<i64>>();

    let mut psp = ProjectSelection::<i64>::new(n + m);
    for (i, &ai) in a.iter().enumerate() {
        // カード i を使うと、その分のお金 ai を消費する。
        psp.add_weight(i, -ai);
    }
    for (i, &bi) in b.iter().enumerate() {
        let k = io.u32() as usize;
        let cards = (0..k).map(|_| io.usize1()).collect::<Vec<usize>>();
        // ボーナス i は、指定されたカードをすべて使った場合にのみ得られる。
        psp.add_and_bonus_when_selected(&cards, bi, n + i);
    }

    let (ans, _) = psp.solve();
    io.writeln(ans);

    io.flush();
}

// POJ: 2987 - Firing
// http://poj.org/problem?id=2987

use anmitsu::graph::project_selection::ProjectSelection;
use anmitsu::io::fastio::Fastio;

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「人数と利益を同じ行に並べる」形式には使えない。
// char の書き込みだけは改行を伴わないため、あらかじめ文字列化したトークン列を
// 1文字ずつ書き込むことで、任意の内容を1行にまとめて出力する。
fn write_line(io: &mut Fastio, tokens: &[String]) {
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            io.write(' ');
        }
        for c in token.chars() {
            io.write(c);
        }
    }
    io.write('\n');
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut psp = ProjectSelection::<i64>::new(n);
    for i in 0..n {
        let b = io.i64();
        // 従業員 i を解雇すると、その分の損得 b を得る。
        psp.add_weight(i, b);
    }
    for _ in 0..m {
        let boss = io.usize1();
        let underling = io.usize1();
        // 上司を解雇するなら、その部下も解雇しなければならない。
        psp.add_constraint(boss, underling);
    }

    let (profit, fired) = psp.solve();
    // 最小カットの S 側 (始点から到達できる側) は、最適値を達成する解雇者集合の
    // うち包含関係で最小のものと一致するため、解雇人数も追加の計算なしに求まる。
    let count = fired.iter().filter(|&&is_fired| is_fired).count();
    write_line(&mut io, &[count.to_string(), profit.to_string()]);

    io.flush();
}

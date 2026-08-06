// Library Checker: Two SAT
// https://judge.yosupo.jp/problem/two_sat

use anmitsu::graph::two_sat::TwoSat;
use anmitsu::io::fastio::Fastio;

/// 文字列トークンを、空白を挟まずにそのまま出力バッファへ書き込む。
fn write_token(io: &mut Fastio, token: &str) {
    for c in token.chars() {
        io.write(c);
    }
}

fn main() {
    let mut io = Fastio::new();

    io.chars(); // "p"
    io.chars(); // "cnf"
    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut sat = TwoSat::new(n);
    for _ in 0..m {
        let a = io.i32();
        let b = io.i32();
        io.i32(); // 節の終端を表す 0

        let (i, f) = if a > 0 {
            (a as usize - 1, true)
        } else {
            ((-a) as usize - 1, false)
        };
        let (j, g) = if b > 0 {
            (b as usize - 1, true)
        } else {
            ((-b) as usize - 1, false)
        };
        sat.add_clause(i, f, j, g);
    }

    match sat.solve() {
        Some(assignment) => {
            write_token(&mut io, "s");
            io.write('\n');
            write_token(&mut io, "SATISFIABLE");
            io.write('\n');
            write_token(&mut io, "v");
            io.write('\n');
            for (i, &value) in assignment.iter().enumerate() {
                let literal = if value { i as i32 + 1 } else { -(i as i32 + 1) };
                io.writeln(literal);
            }
            io.writeln(0_i32);
        }
        None => {
            write_token(&mut io, "s");
            io.write('\n');
            write_token(&mut io, "UNSATISFIABLE");
            io.write('\n');
        }
    }

    io.flush();
}

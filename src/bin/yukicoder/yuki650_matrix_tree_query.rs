// yukicoder: No.650 行列木クエリ
// https://yukicoder.me/problems/no/650

use anmitsu::algebra::monoid::Monoid;
use anmitsu::algebra::semi_group::SemiGroup;
use anmitsu::graph::graph::Graph;
use anmitsu::graph::hld_path_query::HldEdgePathQuery;
use anmitsu::io::fastio::Fastio;

const MOD: u64 = 1_000_000_007;

// この問題専用の 2x2 行列モノイド。root 側を左、leaf 側を右にして掛け合わせる
// 規約であり, op(a, b) は「a の後ろに b を掛ける (a * b)」を表す。
struct Matrix2x2Monoid;

impl SemiGroup for Matrix2x2Monoid {
    type S = [[u64; 2]; 2];
    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        let mut c = [[0_u64; 2]; 2];
        for (i, row) in c.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let mut sum = 0_u128;
                for k in 0..2 {
                    sum += u128::from(a[i][k]) * u128::from(b[k][j]);
                }
                *cell = (sum % u128::from(MOD)) as u64;
            }
        }
        c
    }
}

impl Monoid for Matrix2x2Monoid {
    fn id() -> Self::S {
        [[1, 0], [0, 1]]
    }
}

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「行列の4要素を同じ行に並べる」形式には使えない。
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
    let edges = (0..n - 1)
        .map(|_| {
            let a = io.u32() as usize;
            let b = io.u32() as usize;
            (a, b)
        })
        .collect::<Vec<(usize, usize)>>();

    let mut g = Graph::new(n);
    for &(a, b) in &edges {
        g.add_undirected_edge(a, b, ());
    }

    let hld = g.try_hld(0).unwrap();
    let mut path_query = HldEdgePathQuery::<Matrix2x2Monoid>::new(&hld);

    let q = io.u32() as usize;
    for _ in 0..q {
        let kind = io.char();
        if kind == 'x' {
            let i = io.u32() as usize;
            let x00 = io.u64();
            let x01 = io.u64();
            let x10 = io.u64();
            let x11 = io.u64();
            let (a, b) = edges[i];
            path_query.set_edge(a, b, [[x00, x01], [x10, x11]]);
        } else {
            let i = io.u32() as usize;
            let j = io.u32() as usize;
            let m = path_query.fold_edge_path(i, j);
            write_line(
                &mut io,
                &[
                    m[0][0].to_string(),
                    m[0][1].to_string(),
                    m[1][0].to_string(),
                    m[1][1].to_string(),
                ],
            );
        }
    }

    io.flush();
}

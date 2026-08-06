// AtCoder: Typical90 071 - Fuzzy Priority
// https://atcoder.jp/contests/typical90/tasks/typical90_bs

use std::collections::{HashSet, VecDeque};

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

// Fastio の数値書き込みは write/writeln のどちらを使っても改行文字まで出力してしまうため、
// この問題のように「1行に複数の数値をスペース区切りで並べる」形式には使えない。
// char の書き込みだけは改行を伴わないため、数値を文字列化して1文字ずつ書き込むことで
// 同一行への複数値出力を実現する。
fn write_line(io: &mut Fastio, values: &[u32]) {
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            io.write(' ');
        }
        for c in v.to_string().chars() {
            io.write(c);
        }
    }
    io.write('\n');
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;
    let k = io.u32() as usize;

    let mut g = Graph::new(n);
    // adjacent[u] は、u と直接の前後関係の制約がある頂点の集合である。制約が
    // 無い2頂点は、トポロジカル順序の中で隣り合っていても自由に入れ替えられる。
    let mut adjacent = vec![HashSet::new(); n];
    for _ in 0..m {
        let a = io.usize1();
        let b = io.usize1();
        g.add_edge(a, b, ());
        adjacent[a].insert(b);
        adjacent[b].insert(a);
    }

    let Some(base) = g.topological_sort() else {
        io.writeln(-1_i32);
        io.flush();
        return;
    };

    // トポロジカル順序の中で隣り合う2頂点の間に制約が無ければ、その2つを
    // 入れ替えても妥当な順序のままである。この「隣接swap」を辺とみなすと、
    // すべての妥当な順序同士は互いに行き来できる (よく知られた性質) ため、
    // 既知の1つの妥当な順序から幅優先探索で辿っていけば、まだ見ていない
    // 妥当な順序を次々に見つけられる。
    let mut visited = HashSet::new();
    visited.insert(base.clone());
    let mut queue = VecDeque::new();
    queue.push_back(base);
    let mut results = Vec::new();

    while let Some(order) = queue.pop_front() {
        results.push(order.clone());
        if results.len() == k {
            break;
        }
        for i in 0..n - 1 {
            let (u, v) = (order[i], order[i + 1]);
            if adjacent[u].contains(&v) {
                continue;
            }
            let mut next = order.clone();
            next.swap(i, i + 1);
            if visited.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }

    if results.len() < k {
        // 幅優先探索がこれ以上広がらずに尽きたのは、妥当な順序が K 個
        // 存在しないことを意味する。
        io.writeln(-1_i32);
    } else {
        for order in results {
            let line = order.iter().map(|&v| v as u32 + 1).collect::<Vec<u32>>();
            write_line(&mut io, &line);
        }
    }

    io.flush();
}

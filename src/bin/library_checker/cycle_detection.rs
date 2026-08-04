// Library Checker: Cycle Detection (Directed)
// https://judge.yosupo.jp/problem/cycle_detection

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

/// DFS における頂点の訪問状態。
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    /// 未訪問。
    Unvisited,
    /// 現在の DFS スタック上にある (訪問中)。
    InStack,
    /// 子孫をすべて訪れ終えた (訪問完了)。
    Done,
}

/// 有向グラフから、辺素な閉路を1つ探す。
///
/// # Args
/// - `g`: 探索対象のグラフ。各辺のペイロードには、その辺の (入力での) 番号を
///   持たせておく。
/// - `src_of`: `src_of[e]` は、辺番号 `e` の始点である。
///
/// # Returns
/// `Option<Vec<u32>>`: 閉路が存在する場合、それを構成する辺番号を、通る順に
/// 並べた列。閉路が存在しない場合は `None`。
///
/// # Complexity
/// - 時間計算量: O(V + E)
///   - V は頂点数、E は辺数である。
/// - 空間計算量: O(V + E)
fn find_cycle(g: &Graph<u32>, src_of: &[u32]) -> Option<Vec<u32>> {
    let n = g.vertex_count();
    let mut state = vec![State::Unvisited; n];
    // parent_edge[v] は、v に最初に訪れた際に使った辺の番号である。
    let mut parent_edge = vec![None; n];

    for start in 0..n {
        if state[start] != State::Unvisited {
            continue;
        }

        // (頂点, 隣接辺のスナップショット, 次に見るインデックス) を1フレームとする
        // 非再帰の DFS を行う。隣接辺を頂点ごとに1度だけ収集しておくことで、
        // 各フレームでの添字アクセスを O(1) にする。
        let neighbors = g.edges(start).map(|(v, &e)| (v, e)).collect::<Vec<(usize, u32)>>();
        let mut stack = vec![(start, neighbors, 0_usize)];
        state[start] = State::InStack;

        while let Some(&mut (u, ref neighbors, ref mut idx)) = stack.last_mut() {
            if *idx < neighbors.len() {
                let (v, edge_id) = neighbors[*idx];
                *idx += 1;

                match state[v] {
                    State::Unvisited => {
                        state[v] = State::InStack;
                        parent_edge[v] = Some(edge_id);
                        let next_neighbors = g
                            .edges(v)
                            .map(|(w, &e)| (w, e))
                            .collect::<Vec<(usize, u32)>>();
                        stack.push((v, next_neighbors, 0));
                    }
                    State::InStack => {
                        // v はまだスタック上にあるため、u -> v は後退辺であり、
                        // v から u への木上のパスと合わせて閉路をなす。
                        let mut cycle = vec![edge_id];
                        let mut cur = u;
                        while cur != v {
                            let e = parent_edge[cur].unwrap();
                            cycle.push(e);
                            cur = src_of[e as usize] as usize;
                        }
                        cycle.reverse();
                        return Some(cycle);
                    }
                    State::Done => {}
                }
            } else {
                state[u] = State::Done;
                stack.pop();
            }
        }
    }

    None
}

fn main() {
    let mut io = Fastio::new();

    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut src_of = Vec::with_capacity(m);
    let mut g = Graph::new(n);
    for i in 0..m {
        let u = io.u32() as usize;
        let v = io.u32() as usize;
        src_of.push(u as u32);
        g.add_edge(u, v, i as u32);
    }

    match find_cycle(&g, &src_of) {
        Some(cycle) => {
            io.writeln(cycle.len() as u32);
            for e in cycle {
                io.writeln(e);
            }
        }
        None => {
            io.writeln(-1_i32);
        }
    }

    io.flush();
}

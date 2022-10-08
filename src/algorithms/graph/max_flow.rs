//! verified by
//! - Aizu Online Judge | [GRL_6_A Maximum Flow](https://onlinejudge.u-aizu.ac.jp/problems/GRL_6_A) ([submittion](https://onlinejudge.u-aizu.ac.jp/status/users/Kurosaki96/submissions/1/GRL_6_A/judge/6915995/Rust))
//! - AtCoder | [AtCoder Beginner Contest 091 C - 2D Plane 2N Points](https://atcoder.jp/contests/abc091/tasks/arc092_a) ([submittion](https://atcoder.jp/contests/abc091/submissions/34308482))

use std::collections::VecDeque;

const INF: usize = std::usize::MAX;

#[derive(Clone, Copy)]
struct FlowEdge {
    src: usize,
    dst: usize,
    cap: usize,
    rev: usize,
}

impl FlowEdge {
    fn new(src: usize, dst: usize, cap: usize, rev: usize) -> FlowEdge {
        FlowEdge { src, dst, cap, rev }
    }
}

pub struct Dinic {
    edges: Vec<Vec<FlowEdge>>,
    iter: Vec<usize>,
    level: Vec<isize>,
    que: VecDeque<usize>,
}

impl Dinic {
    pub fn new(n: usize) -> Self {
        Dinic {
            edges: vec![vec![]; n],
            iter: vec![0; n],
            level: vec![-1; n],
            que: VecDeque::new(),
        }
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, cap: usize) {
        let src_len = self.edges[dst].len();
        self.edges[src].push(FlowEdge::new(src, dst, cap, src_len));
        let dst_len = self.edges[src].len();
        self.edges[dst].push(FlowEdge::new(dst, src, 0, dst_len - 1));
    }

    fn bfs(&mut self, start: usize, goal: usize) {
        self.level.iter_mut().for_each(|x| *x = -1);
        self.level[start] = 0;
        self.que.clear();
        self.que.push_back(start);
        while let Some(v) = self.que.pop_front() {
            for e in &self.edges[v] {
                if e.cap > 0 && self.level[e.dst] < 0 {
                    self.level[e.dst] = self.level[e.src] + 1;
                    if e.dst == goal {
                        return;
                    }
                    self.que.push_back(e.dst);
                }
            }
        }
    }

    fn dfs(&mut self, start: usize, goal: usize, flow: usize) -> usize {
        if start == goal {
            return flow;
        }
        for i in self.iter[start]..self.edges[start].len() {
            self.iter[start] = i;
            let e = self.edges[start][i];
            if e.cap > 0 && self.level[start] < self.level[e.dst] {
                let d = self.dfs(e.dst, goal, std::cmp::min(flow, e.cap));
                if d > 0 {
                    self.edges[start][i].cap -= d;
                    self.edges[e.dst][e.rev].cap += d;
                    return d;
                }
            }
        }
        return 0;
    }

    pub fn max_flow(&mut self, start: usize, goal: usize) -> usize {
        let mut flow = 0;
        loop {
            self.bfs(start, goal);
            if self.level[goal] < 0 {
                return flow;
            }
            self.iter.iter_mut().for_each(|x| *x = 0);
            loop {
                let f = self.dfs(start, goal, INF);
                if f == 0 {
                    break;
                }
                flow += f;
            }
        }
    }
}

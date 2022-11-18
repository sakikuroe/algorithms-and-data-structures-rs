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
    pub fn new(src: usize, dst: usize, cap: usize, rev: usize) -> FlowEdge {
        FlowEdge { src, dst, cap, rev }
    }
}

pub struct Dinic {
    n: usize,
    edges: Vec<Vec<FlowEdge>>,
    iter: Vec<usize>,
    level: Vec<isize>,
    que: VecDeque<usize>,
}

impl Dinic {
    pub fn new(n: usize) -> Self {
        Dinic {
            n,
            edges: vec![vec![]; n],
            iter: vec![0; n],
            level: vec![-1; n],
            que: VecDeque::new(),
        }
    }

    pub fn add_node(&mut self) {
        self.n += 1;
        self.edges.push(vec![]);
        self.iter.push(0);
        self.level.push(-1);
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

#[derive(PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

pub struct BurnBuryProblem {
    n: usize,
    sum: usize,
    mf: Dinic,
}

impl BurnBuryProblem {
    pub fn new(n: usize) -> Self {
        BurnBuryProblem {
            n,
            sum: 0,
            mf: Dinic::new(n + 2),
        }
    }

    // penalty w if nodes[idx] is an element of the direct group
    pub fn add_penalty1(&mut self, idx: usize, direct: Direction, w: usize) {
        match direct {
            Direction::Left => {
                self.mf.add_edge(idx + 2, 1, w);
            }
            Direction::Right => {
                self.mf.add_edge(0, idx + 2, w);
            }
        }
    }

    // penalty w
    //    if nodes[idx1] is an element of the direct1 group and
    //       nodes[idx2] is an element of the direct2 group
    pub fn add_penalty2(
        &mut self,
        idx1: usize,
        direct1: Direction,
        idx2: usize,
        direct2: Direction,
        w: usize,
    ) {
        match (direct1, direct2) {
            (Direction::Left, Direction::Right) => {
                self.mf.add_edge(idx1 + 2, idx2 + 2, w);
            }
            (Direction::Right, Direction::Left) => {
                self.mf.add_edge(idx2 + 2, idx1 + 2, w);
            }
            _ => {
                panic!()
            }
        }
    }

    // reward w
    //    if nodes[idx1] is an elements of the direct group
    pub fn add_reward1(&mut self, idx: usize, direct: Direction, w: usize) {
        self.sum += w;
        match direct {
            Direction::Left => {
                self.add_penalty1(idx, Direction::Right, w);
            }
            Direction::Right => {
                self.add_penalty1(idx, Direction::Left, w);
            }
        }
    }

    // reward w
    //    if nodes[idx1] is an element of the direct1 group and
    //       nodes[idx2] is an element of the direct2 group
    pub fn add_reward2(
        &mut self,
        idx1: usize,
        direct1: Direction,
        idx2: usize,
        direct2: Direction,
        w: usize,
    ) {
        self.sum += w;
        self.n += 1;
        self.mf.add_node();
        match (direct1, direct2) {
            (Direction::Left, Direction::Left) => {
                self.add_penalty1(self.n - 1, Direction::Right, w);
                self.mf.add_edge(self.n - 1, idx1 + 2, INF);
                self.mf.add_edge(self.n - 1, idx2 + 2, INF);
            }
            (Direction::Right, Direction::Right) => {
                self.add_penalty1(self.n - 1, Direction::Left, w);
                self.mf.add_edge(idx1 + 2, self.n - 1, INF);
                self.mf.add_edge(idx2 + 2, self.n - 1, INF);
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn get_value(&mut self) -> usize {
        std::cmp::max(self.mf.max_flow(0, 1) as isize - self.sum as isize, 0) as usize
    }
}
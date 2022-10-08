use crate::data_structures::graph::Graph;
use std::collections::VecDeque;

impl Graph<usize> {
    pub fn cycle(&self) -> Vec<usize> {
        let mut indeg = self.edges.iter().map(|e| e.len()).collect::<Vec<_>>();
        let mut que = (0..self.len())
            .filter(|i| indeg[*i] == 1)
            .map(|i| (i, None))
            .collect::<VecDeque<_>>();

        while let Some((v, p)) = que.pop_front() {
            indeg[v] -= 1;
            for e in self.edges[v].iter().filter(|e| Some(e.dst) != p) {
                indeg[e.dst] -= 1;
                if indeg[e.dst] == 1 {
                    que.push_back((e.dst, Some(e.src)));
                }
            }
        }

        (0..self.len())
            .filter(|i| indeg[*i] == 2)
            .collect::<Vec<_>>()
    }

    /// verified by
    /// - AtCoder | [AtCoder Beginner Contest 266  F - Well-defined Path Queries on a Namori](https://atcoder.jp/contests/abc266/tasks/abc266_f) ([submittion](https://atcoder.jp/contests/abc266/submissions/34414332))
    pub fn namori(&self) -> (Vec<usize>, Vec<(usize, Vec<usize>)>) {
        let face = self.cycle();
        let mut tentacles = vec![];
        let mut visited = vec![false; self.len()];
        face.iter().for_each(|i| visited[*i] = true);

        for root in face.iter() {
            let mut tentacle = vec![];
            let mut que = VecDeque::new();
            que.push_back(*root);
            visited[*root] = true;
            while let Some(u) = que.pop_front() {
                for e in self.edges[u].iter() {
                    if visited[e.dst] {
                        continue;
                    }
                    tentacle.push(e.dst);
                    que.push_back(e.dst);
                    visited[e.dst] = true;
                }
            }
            tentacles.push((*root, tentacle));
        }

        (face, tentacles)
    }
}

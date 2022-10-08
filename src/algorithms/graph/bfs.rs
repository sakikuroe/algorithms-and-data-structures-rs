//! verified by
//! - AtCoder | [AtCoder Typical Contest 001 A - 深さ優先探索](https://atcoder.jp/contests/atc001/tasks/dfs_a), ([submittion](https://atcoder.jp/contests/abc007/submissions/34308057))

use crate::data_structures::graph::Graph;
use std::collections::VecDeque;

pub struct Bfs {
    start: usize,
    dist: Vec<Option<usize>>,
    prev: Vec<Option<usize>>,
}

impl<T> Graph<T>
where
    T: Clone,
{
    pub fn bfs(&self, start: usize) -> Bfs {
        let mut dist = vec![None; self.len()];
        dist[start] = Some(0);
        let mut que = VecDeque::new();
        que.push_back(start);
        let mut prev = vec![None; self.len()];

        while let Some(node) = que.pop_front() {
            for e in &self.edges[node] {
                if dist[e.dst].is_none() {
                    prev[e.dst] = Some(e.src);
                    dist[e.dst] = Some(dist[e.src].unwrap() + 1);
                    que.push_back(e.dst);
                }
            }
        }

        Bfs { start, dist, prev }
    }
}

impl Bfs {
    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_distance(&self, dst: usize) -> Option<usize> {
        self.dist[dst]
    }

    pub fn get_path(&self, dst: usize) -> Option<Vec<usize>> {
        self.dist[dst]?;

        let mut prev = self.prev[dst];
        let mut res = vec![dst];
        while let Some(p) = prev {
            res.push(p);
            prev = self.prev[p];
        }
        res.reverse();

        Some(res)
    }
}

use crate::data_structures::graph::Graph;
use std::{cmp, collections::BinaryHeap};

#[allow(unused_macros)]
pub mod macros {
    macro_rules! min { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); std::cmp::min($x, y) } }}
    macro_rules! max { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); std::cmp::max($x, y) } }}
    macro_rules! chmin { ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); if $x > y { $x = y; true } else { false } }}}
    macro_rules! chmax { ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); if $x < y { $x = y; true } else { false } }}}
    macro_rules! multi_vec { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_vec![$element; ($($lens),*)]; $len] ); ($element: expr; ($len: expr)) => ( vec![$element; $len] ); }
    macro_rules! multi_box_array { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_box_array![$element; ($($lens),*)]; $len].into_boxed_slice() ); ($element: expr; ($len: expr)) => ( vec![$element; $len].into_boxed_slice() ); }
    #[allow(unused_imports)]
    pub(super) use {chmax, chmin, max, min, multi_box_array, multi_vec};
}

impl<T> Graph<T>
where
    T: Clone,
{
    /// verified by
    /// - AtCoder | [AtCoder Beginner Contest 223 D - Restricted Permutation](https://atcoder.jp/contests/abc223/tasks/abc223_d), ([submittion](https://atcoder.jp/contests/abc223/submissions/34361828))
    /// - Aizu Online Judge | [GRL_4_B トポロジカルソート](https://onlinejudge.u-aizu.ac.jp/problems/GRL_4_B), ([submittion](https://onlinejudge.u-aizu.ac.jp/status/users/Kurosaki96/submissions/1/GRL_4_B/judge/6923498/Rust))
    pub fn topological_sort(&self) -> Result<Vec<usize>, &str> {
        let mut indegree = vec![0; self.len()];
        for e in self.edges.iter().flatten() {
            indegree[e.dst] += 1
        }

        let mut que = (0..self.len())
            .filter(|i| indegree[*i] == 0)
            .map(cmp::Reverse)
            .collect::<BinaryHeap<_>>();

        let mut sorted = vec![];
        while let Some(cmp::Reverse(priority)) = que.pop() {
            sorted.push(priority);
            for e in &self.edges[priority] {
                indegree[e.dst] -= 1;
                if indegree[e.dst] == 0 {
                    que.push(cmp::Reverse(e.dst))
                }
            }
        }

        if sorted.len() == self.len() {
            Ok(sorted)
        } else {
            Err("The graph could not be sorted in topological order.")
        }
    }

    pub fn is_dag(&self) -> bool {
        self.topological_sort().is_ok()
    }

    /// verified by
    /// - AtCoder | [AtCoder Beginner Contest 139 E - League](https://atcoder.jp/contests/abc139/tasks/abc139_e), ([submittion](https://atcoder.jp/contests/abc139/submissions/34435354))
    pub fn dag_longest(&self) -> usize {
        let mut indegree = vec![0; self.len()];
        for e in self.edges.iter().flatten() {
            indegree[e.dst] += 1
        }
        let mut dist = vec![0; self.len()];
        let mut que = (0..self.len())
            .filter(|i| indegree[*i] == 0)
            .map(cmp::Reverse)
            .collect::<BinaryHeap<_>>();

        let mut sorted = vec![];
        while let Some(cmp::Reverse(priority)) = que.pop() {
            sorted.push(priority);
            for e in &self.edges[priority] {
                indegree[e.dst] -= 1;
                macros::chmax!(dist[e.dst], dist[e.src] + 1);
                if indegree[e.dst] == 0 {
                    que.push(cmp::Reverse(e.dst))
                }
            }
        }

        dist.into_iter().max().unwrap() + 1
    }
}

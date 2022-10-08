//! verified by
//! - AtCoder | [第７回日本情報オリンピック 予選（過去問） F - 船旅](https://atcoder.jp/contests/joi2008yo/tasks/joi2008yo_f), ([submittion]())
//! - AtCoder | [AtCoder Beginner Contest 126 D - Even Relation](https://atcoder.jp/contests/abc126/tasks/abc126_d), ([submittion]())

use crate::{algorithms::number_theory::ntt::ModInt, data_structures::graph::Graph};
use std::{cmp::Ordering, collections::BinaryHeap};

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

#[derive(Clone, PartialEq)]
pub struct State<T> {
    pub id: usize,
    pub priority: T,
}

impl<T> Eq for State<T> where T: PartialEq {}

impl<T> Ord for State<T>
where
    T: PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (self.priority)
            .partial_cmp(&(other.priority))
            .unwrap()
            .reverse()
    }
}

impl<T> PartialOrd for State<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Dijkstra<T> {
    start: usize,
    dist: Vec<Option<T>>,
    prev: Vec<Option<usize>>,
    cases: Box<[ModInt]>,
}

impl<T> Dijkstra<T>
where
    Option<T>: Copy,
{
    pub fn get_distance(&self, v: usize) -> Option<T> {
        self.dist[v]
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_path(&self, v: usize) -> Option<Vec<usize>> {
        self.dist[v]?;

        let mut id = v;
        let mut res = vec![id];
        while let Some(prev_node) = self.prev[id] {
            id = prev_node;
            res.push(id);
        }
        res.reverse();

        Some(res)
    }

    pub fn get_cases(&self, v: usize) -> ModInt {
        self.cases[v]
    }
}

impl Graph<usize> {
    pub fn dijkstra(&self, start: usize) -> Dijkstra<usize> {
        let mut dist = vec![std::usize::MAX; self.len()];
        let mut prev = vec![None; self.len()];
        let mut cases = macros::multi_box_array!(ModInt::new(0); (self.len()));

        dist[start] = 0;
        cases[start] = ModInt::new(1);
        let mut que = BinaryHeap::new();
        que.push(State {
            id: start,
            priority: 0,
        });

        while let Some(State { id, priority }) = que.pop() {
            if priority > dist[id] {
                continue;
            }
            for e in &self.edges[id] {
                if macros::chmin!(dist[e.dst], dist[e.src] + e.weight) {
                    prev[e.dst] = Some(e.src);
                    que.push(State {
                        id: e.dst,
                        priority: dist[e.dst],
                    });
                    cases[e.dst] = cases[e.src];
                } else if dist[e.dst] == dist[e.src] + e.weight {
                    cases[e.dst] += cases[e.src];
                }
            }
        }

        Dijkstra {
            start,
            dist: dist
                .into_iter()
                .map(|d| if d < std::usize::MAX { Some(d) } else { None })
                .collect(),
            prev,
            cases,
        }
    }
}

impl Graph<f64> {
    pub fn dijkstra(&self, start: usize) -> Dijkstra<f64> {
        let mut dist = vec![std::f64::MAX; self.len()];
        let mut prev = vec![None; self.len()];
        let mut cases = macros::multi_box_array!(ModInt::new(0); (self.len()));

        dist[start] = 0.0;
        let mut que = BinaryHeap::new();
        que.push(State {
            id: start,
            priority: 0.0,
        });

        while let Some(State { id, priority }) = que.pop() {
            if priority > dist[id] {
                continue;
            }
            for e in &self.edges[id] {
                if macros::chmin!(dist[e.dst], dist[e.src] + e.weight) {
                    prev[e.dst] = Some(e.src);
                    que.push(State {
                        id: e.dst,
                        priority: dist[e.dst],
                    });
                    cases[e.dst] = cases[e.src];
                } else if dist[e.dst] == dist[e.src] + e.weight {
                    cases[e.dst] += cases[e.src];
                }
            }
        }

        Dijkstra {
            start,
            dist: dist
                .into_iter()
                .map(|d| if d < std::f64::MAX { Some(d) } else { None })
                .collect(),
            prev,
            cases,
        }
    }
}

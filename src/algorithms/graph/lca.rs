/// verified by
/// - Library Checker | [Jump on Tree](https://judge.yosupo.jp/problem/jump_on_tree), ([submittion](https://judge.yosupo.jp/submission/117545))
use crate::data_structures::graph::Graph;
use std::{collections::VecDeque, mem};
const INF: usize = std::usize::MAX;

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

pub struct LCA {
    root: usize,
    height: usize,
    parent: Vec<Vec<Option<usize>>>,
    depth: Vec<usize>,
    dist_from_root: Vec<usize>,
    min_weight: Vec<Vec<usize>>,
    max_weight: Vec<Vec<usize>>,
}

impl LCA {
    pub fn new(root: usize, g: &Graph<usize>) -> LCA {
        let bfs = |start: usize,
                   parent: &mut [Vec<Option<usize>>],
                   depth: &mut Vec<usize>,
                   dist_from_root: &mut Vec<usize>,
                   min_weight: &mut Vec<Vec<usize>>,
                   max_weight: &mut Vec<Vec<usize>>| {
            let mut que = VecDeque::new();

            depth[start] = 0;
            dist_from_root[start] = 0;
            que.push_back((start, None));

            while let Some((now, p)) = que.pop_front() {
                for e in g.edges[now].iter().filter(|e| p != Some(e.dst)) {
                    depth[e.dst] = depth[e.src] + 1;
                    dist_from_root[e.dst] = dist_from_root[e.src] + e.weight;
                    parent[0][e.dst] = Some(e.src);
                    min_weight[0][e.dst] = e.weight;
                    max_weight[0][e.dst] = e.weight;
                    que.push_back((e.dst, Some(e.src)));
                }
            }
        };

        let height = (1..).find(|&x| (1 << x) >= g.len()).unwrap();
        let mut parent = vec![vec![None; g.len()]; height];
        let mut depth = vec![INF; g.len()];
        let mut dist_from_root = vec![INF; g.len()];
        let mut min_weight = vec![vec![std::usize::MAX; g.len()]; height];
        let mut max_weight = vec![vec![0; g.len()]; height];

        bfs(
            root,
            &mut parent,
            &mut depth,
            &mut dist_from_root,
            &mut min_weight,
            &mut max_weight,
        );

        for k in 1..height {
            for i in 0..g.len() {
                if let Some(j) = parent[k - 1][i] {
                    parent[k][i] = parent[k - 1][j];
                    min_weight[k][i] = std::cmp::min(min_weight[k - 1][j], min_weight[k - 1][i]);
                    max_weight[k][i] = std::cmp::max(max_weight[k - 1][j], max_weight[k - 1][i]);
                }
            }
        }

        LCA {
            root,
            height,
            parent,
            depth,
            dist_from_root,
            min_weight,
            max_weight,
        }
    }

    pub fn get_root(&self) -> usize {
        self.root
    }

    pub fn get_lca(&self, mut u: usize, mut v: usize) -> usize {
        if self.depth[u] > self.depth[v] {
            mem::swap(&mut u, &mut v);
        }

        for k in (0..self.height).rev() {
            if self.depth[v] - self.depth[u] >= 1 << k {
                v = self.parent[k][v].unwrap();
            }
        }

        if u == v {
            u
        } else {
            for k in (0..self.height).rev() {
                if self.parent[k][u] != self.parent[k][v] {
                    u = self.parent[k][u].unwrap();
                    v = self.parent[k][v].unwrap();
                }
            }
            self.parent[0][u].unwrap()
        }
    }

    pub fn get_dist(&self, u: usize, v: usize) -> usize {
        self.dist_from_root[u] + self.dist_from_root[v]
            - 2 * self.dist_from_root[self.get_lca(u, v)]
    }

    pub fn is_on_path(&self, u: usize, v: usize, a: usize) -> bool {
        self.get_dist(u, v) == self.get_dist(u, a) + self.get_dist(a, v)
    }

    /// Returns:
    ///     Vertex k proceeded from u towards v
    pub fn jump(&self, u: usize, v: usize, k: usize) -> Option<usize> {
        let d = self.get_dist(u, v);
        if d < k {
            return None;
        }

        let climb = |mut u: usize, k: usize| -> usize {
            for i in (0..self.height).filter(|&i| k >> i & 1 == 1) {
                u = self.parent[i][u].unwrap();
            }
            u
        };

        let e = self.get_dist(u, self.get_lca(u, v));
        if k <= e {
            Some(climb(u, k))
        } else {
            Some(climb(v, d - k))
        }
    }

    pub fn get_min_weight(&self, mut u: usize, mut v: usize) -> usize {
        let w = self.get_lca(u, v);
        let mut ans = std::usize::MAX;

        {
            let d = self.depth[u] - self.depth[w];
            for i in 0..self.height {
                if d >> i & 1 == 1 {
                    macros::chmin!(ans, self.min_weight[i][u]);
                    u = self.parent[i][u].unwrap();
                }
            }
        }

        {
            let d = self.depth[v] - self.depth[w];
            for i in 0..self.height {
                if d >> i & 1 == 1 {
                    macros::chmin!(ans, self.min_weight[i][v]);
                    v = self.parent[i][v].unwrap();
                }
            }
        }

        ans
    }

    pub fn get_max_weight(&self, mut u: usize, mut v: usize) -> usize {
        let w = self.get_lca(u, v);
        let mut ans = 0;

        {
            let d = self.depth[u] - self.depth[w];
            for i in 0..self.height {
                if d >> i & 1 == 1 {
                    macros::chmax!(ans, self.max_weight[i][u]);
                    u = self.parent[i][u].unwrap();
                }
            }
        }

        {
            let d = self.depth[v] - self.depth[w];
            for i in 0..self.height {
                if d >> i & 1 == 1 {
                    macros::chmax!(ans, self.max_weight[i][v]);
                    v = self.parent[i][v].unwrap();
                }
            }
        }

        ans
    }
}

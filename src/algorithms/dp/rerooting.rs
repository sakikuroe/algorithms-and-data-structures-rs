//! verified by
//! - AtCoder | [競プロ典型 90 問 003 - Longest Circular Road（★4）](https://atcoder.jp/contests/typical90/tasks/typical90_c), ([submittion](https://atcoder.jp/contests/typical90/submissions/35251033))
//! - AtCoder | [AtCoder Beginner Contest 160 F - Distributing Integers](https://atcoder.jp/contests/abc160/tasks/abc160_f), ([submittion](https://atcoder.jp/contests/abc160/submissions/35252422))

use crate::data_structures::graph::Graph;

pub struct ReRootingSolver<T, Identity, Merge, AddRoot> {
    dp: Vec<Vec<T>>,
    g: Graph<usize>,
    identity: Identity,
    merge: Merge,
    add_root: AddRoot,
}

impl<T, Identity, Merge, AddRoot> ReRootingSolver<T, Identity, Merge, AddRoot>
where
    T: Clone,
    Identity: Fn() -> T,
    Merge: Fn(&T, &T) -> T,
    AddRoot: Fn(&T) -> T,
{
    pub fn new(n: usize, merge: Merge, identity: Identity, add_root: AddRoot) -> Self {
        ReRootingSolver {
            dp: vec![vec![]; n],
            g: Graph::new(n),
            identity,
            merge,
            add_root,
        }
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, weight: usize) {
        self.g.add_edge(src, dst, weight);
    }

    pub fn dfs1(&mut self, v: usize, p: Option<usize>, ans: &mut Vec<T>) -> T {
        let mut sum = (self.identity)();
        let deg = self.g.edges[v].len();
        self.dp[v] = vec![(self.identity)(); deg];
        let iter = self.g.edges[v].clone().into_iter();
        for (i, e) in iter.enumerate() {
            if p == Some(e.dst) {
                continue;
            }
            let t = self.dfs1(e.dst, Some(e.src), ans);
            self.dp[v][i] = t.clone();
            sum = (self.merge)(&sum, &t);
        }
        (self.add_root)(&sum)
    }

    pub fn dfs2(&mut self, v: usize, p: Option<usize>, dp_p: T, ans: &mut Vec<T>) {
        for (i, e) in self.g.edges[v].iter().enumerate() {
            if p == Some(e.dst) {
                self.dp[v][i] = dp_p.clone();
            }
        }

        let deg = self.g.edges[v].len();
        let mut dp_l = vec![(self.identity)(); deg + 1];
        let mut dp_r = vec![(self.identity)(); deg + 1];
        for i in 0..deg {
            dp_l[i + 1] = (self.merge)(&dp_l[i], &self.dp[v][i]);
            dp_r[deg - 1 - i] = (self.merge)(&dp_r[deg - i], &self.dp[v][deg - 1 - i]);
        }

        ans[v] = (self.add_root)(&dp_l[deg]);

        let e = self.g.edges[v].clone();
        for (i, e) in e.into_iter().enumerate() {
            if p == Some(e.dst) {
                continue;
            }
            self.dfs2(
                e.dst,
                Some(e.src),
                (self.add_root)(&(self.merge)(&dp_l[i], &dp_r[i + 1])),
                ans,
            );
        }
    }

    pub fn solve(&mut self) -> Vec<T> {
        let mut ans = vec![(self.identity)(); self.g.len()];
        self.dfs1(0, None, &mut ans);
        self.dfs2(0, None, (self.identity)(), &mut ans);
        ans
    }
}

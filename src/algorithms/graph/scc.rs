//! verified by
//! - AtCoder | [競プロ典型 90 問 021 - Come Back in One Piece（★5）](https://atcoder.jp/contests/typical90/tasks/typical90_u) ([submittion](https://atcoder.jp/contests/typical90/submissions/34970495))

use crate::data_structures::graph::Graph;

impl<T> Graph<T>
where
    T: Clone + Copy,
{
    pub fn scc(&self) -> Vec<Vec<usize>> {
        fn dfs<T>(v: usize, set: &mut Vec<usize>, used: &mut Vec<bool>, g: &Graph<T>) {
            used[v] = true;
            for e in &g.edges[v] {
                if !used[e.dst] {
                    dfs(e.dst, set, used, g);
                }
            }
            set.push(v);
        }

        let mut vs = vec![];
        let mut used = vec![false; self.len()];
        for v in 0..self.len() {
            if !used[v] {
                dfs(v, &mut vs, &mut used, self);
            }
        }

        let mut rev_g = Graph::new(self.len());
        for e in self.edges.iter().flatten() {
            rev_g.add_edge(e.dst, e.src, e.weight);
        }

        let mut scc = vec![];
        let mut scc_len = 0;
        let mut used = vec![false; self.len()];
        for v in vs.iter().rev() {
            if !used[*v] {
                scc.push(vec![]);
                scc_len += 1;
                dfs(*v, &mut scc[scc_len - 1], &mut used, &rev_g);
            }
        }

        scc
    }
}

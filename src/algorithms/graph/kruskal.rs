use crate::data_structures::{
    graph::{Edge, Graph},
    union_find::UnionFind,
};

impl<T> Graph<T>
where
    T: Clone + Copy + Ord,
{
    //! verified by
    //! - Aizu Online Judge | [AOJ GRL_2_A 最小全域木](https://onlinejudge.u-aizu.ac.jp/problems/GRL_2_A) ([submittion](https://onlinejudge.u-aizu.ac.jp/status/users/Kurosaki96/submissions/1/GRL_2_A/judge/6915986/Rust))
    pub fn kruskal(&self) -> Vec<Edge<T>> {
        let mut res = vec![];
        let mut uf = UnionFind::new(self.len());

        let mut all_edges = self
            .edges
            .iter()
            .cloned()
            .flatten()
            .collect::<Vec<Edge<T>>>();

        all_edges.sort_by_key(|e| e.weight);

        for e in all_edges {
            if !uf.is_same(e.src, e.dst) {
                uf.union(e.src, e.dst);
                res.push(e);
            }
        }

        res
    }
}

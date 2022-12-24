use crate::data_structures::graph::Graph;
use std::collections::VecDeque;

/// verified by
/// - AtCoder | [HHKBプログラミングコンテスト2022 Winter(AtCoder Beginner Contest 282) D. Make Bipartite 2](https://atcoder.jp/contests/abc282/tasks/abc282_d), ([submittion](https://atcoder.jp/contests/abc282/submissions/37468042))

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BW {
    Black,
    White,
}

impl BW {
    pub fn is_black(&self) -> bool {
        match *self {
            BW::Black => true,
            BW::White => false,
        }
    }

    pub fn is_white(&self) -> bool {
        match *self {
            BW::White => true,
            BW::Black => false,
        }
    }

    pub fn flip(&self) -> BW {
        match self.clone() {
            BW::Black => BW::White,
            BW::White => BW::Black,
        }
    }
}

impl<T> Graph<T>
where
    T: Clone,
{
    pub fn is_bipartite(&self) -> Option<Vec<BW>> {
        let mut res = vec![None; self.len()];
        // Breadth-first search for each connected component
        for v in 0..self.len() {
            if res[v].is_none() {
                let mut que = VecDeque::new();
                que.push_back(v);
                res[v] = Some(BW::Black);
                while let Some(u) = que.pop_front() {
                    for e in self.edges[u].iter() {
                        if res[e.dst].is_none() {
                            res[e.dst] = Some(res[e.src].unwrap().flip());
                            que.push_back(e.dst);
                        } else if res[e.src] == res[e.dst] {
                            return None;
                        }
                    }
                }
            }
        }

        let res = res
            .into_iter()
            .map(|x| x.unwrap_or(BW::Black))
            .collect::<Vec<_>>();

        Some(res)
    }
}

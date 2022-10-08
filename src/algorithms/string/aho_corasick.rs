use std::collections::{BTreeMap, VecDeque};

/// verified by
/// - AtCoder | [ユニークビジョンプログラミングコンテスト2022 夏(AtCoder Beginner Contest 268) Ex - Taboo](https://atcoder.jp/contests/abc268/tasks/abc268_h), ([submittion](https://atcoder.jp/contests/abc268/submissions/35157272))
/// (解説)<https://youtu.be/BYoRvdgI5EU?t=9283>
pub struct Node<T>
where
    T: Clone + Copy + Ord,
{
    to: BTreeMap<T, usize>,
    end: bool,
    failure: Option<usize>,
}

// https://youtu.be/BYoRvdgI5EU?t=9283
pub struct AhoCorasick<T>
where
    T: Clone + Copy + Ord,
{
    nodes: Vec<Node<T>>,
}

impl<T> AhoCorasick<T>
where
    T: Clone + Copy + Ord,
{
    pub fn new() -> Self {
        AhoCorasick {
            nodes: vec![Node {
                to: BTreeMap::new(),
                end: false,
                failure: None,
            }],
        }
    }

    pub fn add(&mut self, s: &Vec<T>) -> usize {
        let mut idx = 0;
        for &c in s {
            if !self.nodes[idx].to.contains_key(&c) {
                let len = self.nodes.len();
                self.nodes[idx].to.insert(c, len);
                self.nodes.push(Node {
                    to: BTreeMap::new(),
                    end: false,
                    failure: None,
                });
            }
            idx = *self.nodes[idx].to.get(&c).unwrap();
        }
        self.nodes[idx].end = true;
        idx
    }

    pub fn is_end(&self, v: usize) -> bool {
        self.nodes[v].end
    }

    pub fn next(&mut self, mut v: Option<usize>, c: T) -> usize {
        while let Some(u) = v {
            if let Some(w) = self.nodes[u].to.get(&c) {
                return *w;
            }
            v = self.nodes[u].failure;
        }

        0
    }

    pub fn build(&mut self) {
        let mut que = VecDeque::new();
        que.push_back(0);
        while let Some(v) = que.pop_front() {
            let iter = self.nodes[v].to.clone();
            for (c, u) in iter {
                self.nodes[u].failure = Some(self.next(self.nodes[v].failure, c));
                self.nodes[u].end |= self.nodes[self.nodes[u].failure.unwrap()].end;
                que.push_back(u);
            }
        }
    }
}

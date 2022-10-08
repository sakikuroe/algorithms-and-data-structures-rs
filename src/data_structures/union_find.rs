use std::cmp::Ordering;

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 120 D - Decayed Bridges](https://atcoder.jp/contests/abc120/tasks/abc120_d), ([submittion](https://atcoder.jp/contests/abc120/submissions/34307175))
/// - AtCoder | [AtCoder Typical Contest 001 B - Union Find](https://atcoder.jp/contests/atc001/tasks/unionfind_a), ([submittion](https://atcoder.jp/contests/atc001/submissions/34307248))
#[derive(Clone)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect::<Vec<_>>(),
            rank: vec![0; n],
            size: vec![1; n],
        }
    }

    pub fn is_root(&self, x: usize) -> bool {
        self.parent[x] == x
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.is_root(x) {
            x
        } else {
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
            root
        }
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let (root_x, root_y) = (self.find(x), self.find(y));

        if root_x != root_y {
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                Ordering::Greater => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                }
                Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                    self.rank[root_x] += 1;
                }
                Ordering::Less => {
                    self.parent[root_x] = root_y;
                    self.size[root_y] += self.size[root_x];
                }
            }
        }
    }

    pub fn is_same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn get_size(&mut self, x: usize) -> usize {
        if self.is_root(x) {
            self.size[x]
        } else {
            let root = self.find(x);
            self.size[root]
        }
    }
}

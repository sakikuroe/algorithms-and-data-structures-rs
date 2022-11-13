use std::{cmp::Ordering, collections::BTreeMap};

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
                Ordering::Less => {
                    self.parent[root_x] = root_y;
                    self.size[root_y] += self.size[root_x];
                }
                Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                    self.rank[root_x] += 1;
                }
                Ordering::Greater => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
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

/// verified by
/// - AtCoder | [大和証券プログラミングコンテスト2022 Autumn (AtCoder Beginner Contest 277) C - Ladder Takahashi](https://atcoder.jp/contests/abc277/tasks/abc277_c), ([submittion](https://atcoder.jp/contests/abc277/submissions/36467109))
/// - AtCoder | [大和証券プログラミングコンテスト2022 Autumn (AtCoder Beginner Contest 277) D - Takahashi's Solitaire](https://atcoder.jp/contests/abc277/tasks/abc277_d), ([submittion](https://atcoder.jp/contests/abc277/submissions/36466943))
#[derive(Clone)]
pub struct UnionFindSparse {
    parent: BTreeMap<usize, usize>,
    rank: BTreeMap<usize, usize>,
    size: BTreeMap<usize, usize>,
}

impl UnionFindSparse {
    pub fn new() -> Self {
        UnionFindSparse {
            parent: BTreeMap::new(),
            rank: BTreeMap::new(),
            size: BTreeMap::new(),
        }
    }

    pub fn is_root(&mut self, x: usize) -> bool {
        self.parent.get(&x).unwrap_or(&x) == &x
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.is_root(x) {
            x
        } else {
            let root = self.find(*self.parent.get(&x).unwrap_or(&x));
            self.parent.insert(x, root);
            root
        }
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let (root_x, root_y) = (self.find(x), self.find(y));
        if root_x != root_y {
            match (*self.rank.get(&root_x).unwrap_or(&0)).cmp(self.rank.get(&root_y).unwrap_or(&0))
            {
                Ordering::Less => {
                    self.parent.insert(root_x, root_y);
                    self.size.insert(
                        root_y,
                        *self.size.get(&root_y).unwrap_or(&1)
                            + self.size.get(&root_x).unwrap_or(&1),
                    );
                }
                Ordering::Equal => {
                    self.parent.insert(root_y, root_x);
                    self.size.insert(
                        root_x,
                        *self.size.get(&root_x).unwrap_or(&1)
                            + self.size.get(&root_y).unwrap_or(&1),
                    );
                    self.rank
                        .insert(root_x, *self.rank.get(&root_x).unwrap_or(&0) + 1);
                }
                Ordering::Greater => {
                    self.parent.insert(root_y, root_x);
                    self.size.insert(
                        root_x,
                        *self.size.get(&root_x).unwrap_or(&1)
                            + self.size.get(&root_y).unwrap_or(&1),
                    );
                }
            }
        }
    }

    pub fn is_same(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn get_size(&mut self, x: usize) -> usize {
        if self.is_root(x) {
            *self.size.get(&x).unwrap_or(&1)
        } else {
            let root = self.find(x);
            *self.size.get(&root).unwrap_or(&1)
        }
    }
}
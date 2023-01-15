use std::{cmp::Ordering, collections::BTreeMap};

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 120 D - Decayed Bridges](https://atcoder.jp/contests/abc120/tasks/abc120_d), ([submittion](https://atcoder.jp/contests/abc120/submissions/34307175))
/// - AtCoder | [AtCoder Typical Contest 001 B - Union Find](https://atcoder.jp/contests/atc001/tasks/unionfind_a), ([submittion](https://atcoder.jp/contests/atc001/submissions/34307248))
#[derive(Clone)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
    group_next: Vec<usize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect::<Vec<_>>(),
            rank: vec![0; n],
            size: vec![1; n],
            group_next: (0..n).collect::<Vec<_>>(),
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
            self.group_next.swap(root_x, root_y);
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

    /// Returns:
    ///     (0..self.parent.len()).filter(|&y| self.is_same(x, y)).collect::<Vec<_>>()
    pub fn get_group(&mut self, x: usize) -> Vec<usize> {
        let mut res = vec![x];
        let mut v = x;
        while self.group_next[v] != x {
            res.push(self.group_next[v]);
            v = self.group_next[v];
        }
        res.sort();
        res
    }
}

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 285 D - Change Usernames](https://atcoder.jp/contests/abc285/tasks/abc285_d), ([submittion](https://atcoder.jp/contests/abc285/submissions/38070281))
/// - AtCoder | [大和証券プログラミングコンテスト2022 Autumn (AtCoder Beginner Contest 277) C - Ladder Takahashi](https://atcoder.jp/contests/abc277/tasks/abc277_c), ([submittion](https://atcoder.jp/contests/abc277/submissions/38077025))
/// - AtCoder | [大和証券プログラミングコンテスト2022 Autumn (AtCoder Beginner Contest 277) D - Takahashi's Solitaire](https://atcoder.jp/contests/abc277/tasks/abc277_d), ([submittion](https://atcoder.jp/contests/abc277/submissions/38077028))
#[derive(Clone)]
pub struct UnionFindSparse<T>
where
    T: Ord + Clone,
{
    parent: BTreeMap<T, T>,
    rank: BTreeMap<T, usize>,
    size: BTreeMap<T, usize>,
}

impl<T> UnionFindSparse<T>
where
    T: Ord + Clone,
{
    pub fn new() -> Self {
        UnionFindSparse {
            parent: BTreeMap::new(),
            rank: BTreeMap::new(),
            size: BTreeMap::new(),
        }
    }

    pub fn is_root(&mut self, x: T) -> bool {
        self.parent.get(&x).unwrap_or(&x) == &x
    }

    pub fn find(&mut self, x: T) -> T {
        if self.is_root(x.clone()) {
            x
        } else {
            let root = self.find(self.parent.get(&x).unwrap_or(&x).clone());
            self.parent.insert(x, root.clone());
            root
        }
    }

    pub fn union(&mut self, x: T, y: T) {
        let (root_x, root_y) = (self.find(x), self.find(y));
        if root_x != root_y {
            match (*self.rank.get(&root_x).unwrap_or(&0)).cmp(self.rank.get(&root_y).unwrap_or(&0))
            {
                Ordering::Less => {
                    self.parent.insert(root_x.clone(), root_y.clone());
                    self.size.insert(
                        root_y.clone(),
                        *self.size.get(&root_y).unwrap_or(&1)
                            + self.size.get(&root_x).unwrap_or(&1),
                    );
                }
                Ordering::Equal => {
                    self.parent.insert(root_y.clone(), root_x.clone());
                    self.size.insert(
                        root_x.clone(),
                        *self.size.get(&root_x).unwrap_or(&1)
                            + self.size.get(&root_y).unwrap_or(&1),
                    );
                    self.rank
                        .insert(root_x.clone(), *self.rank.get(&root_x).unwrap_or(&0) + 1);
                }
                Ordering::Greater => {
                    self.parent.insert(root_y.clone(), root_x.clone());
                    self.size.insert(
                        root_x.clone(),
                        *self.size.get(&root_x).unwrap_or(&1)
                            + self.size.get(&root_y).unwrap_or(&1),
                    );
                }
            }
        }
    }

    pub fn is_same(&mut self, x: T, y: T) -> bool {
        self.find(x) == self.find(y)
    }

    pub fn get_size(&mut self, x: T) -> usize {
        if self.is_root(x.clone()) {
            *self.size.get(&x).unwrap_or(&1)
        } else {
            let root = self.find(x);
            *self.size.get(&root).unwrap_or(&1)
        }
    }
}

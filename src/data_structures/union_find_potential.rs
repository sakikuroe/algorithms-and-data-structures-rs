use std::cmp::Ordering;
use crate::algebraic_structures::commutative_group::CommutativeGroup;

/// verified by
/// - AtCoder | [AtCoder Beginner Contest 087 D - People on a Line](https://atcoder.jp/contests/abc087/tasks/arc090_b), ([submittion](https://atcoder.jp/contests/abc087/submissions/37278562))
#[derive(Clone)]
pub struct UnionFindPotential<G>
where
    G: CommutativeGroup,
{
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
    potential: Vec<G::S>,
}

impl<G> UnionFindPotential<G>
where
    G: CommutativeGroup,
    G::S: Clone,
{
    pub fn new(n: usize) -> Self {
        UnionFindPotential {
            parent: (0..n).collect::<Vec<_>>(),
            rank: vec![0; n],
            size: vec![1; n],
            potential: vec![G::id(); n],
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
            self.potential[x] = G::op(&self.potential[x], &self.potential[self.parent[x]]);
            self.parent[x] = root;
            root
        }
    }

    pub fn get_potential(&mut self, x: usize) -> G::S {
        self.find(x);
        self.potential[x].clone()
    }

    pub fn union(&mut self, x: usize, y: usize, mut w: G::S) {
        let (root_x, root_y) = (self.find(x), self.find(y));
        w = G::op(&w, &self.get_potential(x));
        w = G::op(&w, &G::inv(&self.get_potential(y)));
        if root_x != root_y {
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                Ordering::Less => {
                    self.parent[root_x] = root_y;
                    self.size[root_y] += self.size[root_x];
                    self.potential[root_x] = G::inv(&w);
                }
                Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                    self.rank[root_x] += 1;
                    self.potential[root_y] = w;
                }
                Ordering::Greater => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                    self.potential[root_y] = w;
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

    pub fn get_diff(&mut self, x: usize, y: usize) -> G::S {
        G::op(&self.get_potential(y), &G::inv(&self.get_potential(x)))
    }
}

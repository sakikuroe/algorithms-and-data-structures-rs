use super::union_find::UnionFind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Edge<T> {
    pub src: usize,
    pub dst: usize,
    pub weight: T,
}

#[derive(Clone, Debug)]
pub struct Graph<T> {
    pub edges: Vec<Vec<Edge<T>>>,
}

impl<T> Graph<T>
where
    T: Clone,
{
    pub fn new(n: usize) -> Self {
        Graph {
            edges: vec![vec![]; n],
        }
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, weight: T) {
        self.edges[src].push(Edge { src, dst, weight });
    }
}

impl<T> Graph<T>
where
    T: Clone,
{
    pub fn connected(&self) -> bool {
        let mut uf = UnionFind::new(self.len());
        for e in self.edges.iter().flatten() {
            uf.union(e.src, e.dst);
        }
        uf.get_size(0) == self.len()
    }
}

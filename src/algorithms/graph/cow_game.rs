use crate::data_structures::graph::Graph;

pub struct CowGameSolver<T> {
    pub n: usize,
    pub g: Graph<T>,
}

impl<T> CowGameSolver<T>
where
    T: Clone,
{
    pub fn new(n: usize) -> Self {
        CowGameSolver {
            n,
            g: Graph::<T>::new(n),
        }
    }

    // add constraint A_{i} - A_{j} <= w
    pub fn add(&mut self, i: usize, j: usize, w: T) {
        self.g.add_edge(j, i, w);
    }
}

impl CowGameSolver<usize> {
    // return \{A_{i}\}_{i=0}^{n - 1} s.t. maximize A_{n - 1} - A_{0}
    pub fn solve(&self) -> Vec<usize> {
        let dijk = self.g.dijkstra(0);
        (0..self.n).map(|i| dijk.get_distance(i).unwrap()).collect()
    }
}

impl CowGameSolver<isize> {
    // return \{A_{i}\}_{i=0}^{n - 1} s.t. maximize A_{n - 1} - A_{0}
    pub fn solve(&self) -> Vec<isize> {
        self.g.bellman_ford(0).unwrap()
    }
}

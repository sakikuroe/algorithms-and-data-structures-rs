use crate::data_structures::graph::Graph;

const INF: usize = std::usize::MAX;

#[allow(unused_macros)]
pub mod macros {
    macro_rules! min { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); std::cmp::min($x, y) } }}
    macro_rules! max { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); std::cmp::max($x, y) } }}
    macro_rules! chmin { ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); if $x > y { $x = y; true } else { false } }}}
    macro_rules! chmax { ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); if $x < y { $x = y; true } else { false } }}}
    macro_rules! multi_vec { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_vec![$element; ($($lens),*)]; $len] ); ($element: expr; ($len: expr)) => ( vec![$element; $len] ); }
    macro_rules! multi_box_array { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_box_array![$element; ($($lens),*)]; $len].into_boxed_slice() ); ($element: expr; ($len: expr)) => ( vec![$element; $len].into_boxed_slice() ); }
    #[allow(unused_imports)]
    pub(super) use {chmax, chmin, max, min, multi_box_array, multi_vec};
}

pub struct FloydWarshall {
    dist: Vec<Vec<Option<usize>>>,
}

impl Graph<usize> {
    pub fn floyd_warshall(&mut self) -> FloydWarshall {
        let mut dist = macros::multi_vec!(None ; (self.len(), self.len()));

        for e in self.edges.iter().flatten() {
            dist[e.src][e.dst] = Some(e.weight);
        }

        for k in 0..self.len() {
            for i in 0..self.len() {
                for j in 0..self.len() {
                    if dist[i][k].is_some() && dist[k][j].is_some() {
                        if dist[i][j].unwrap_or(INF) > dist[i][k].unwrap() + dist[k][j].unwrap() {
                            dist[i][j] = Some(dist[i][k].unwrap() + dist[k][j].unwrap());
                        }
                    }
                }
            }
        }

        FloydWarshall { dist }
    }
}

impl FloydWarshall {
    pub fn get_distance(&self, src: usize, dst: usize) -> Option<usize> {
        self.dist[src][dst]
    }
}

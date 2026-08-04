pub mod algebra {
    pub mod monoid;
    pub mod semi_group;
}

pub mod modulo998244353 {
    pub mod combinatorics;
    pub mod convolution;
    mod convolution_avx2;
    pub mod convolution_mont;
    pub mod fps;
    pub mod modint;
    pub mod modulo;
}

pub mod ds {
    pub mod segment_tree {
        pub mod segment_tree_dense;
    }
    pub mod bit_vector;
    pub mod union_find;
    pub mod wavelet_matrix;
}

pub mod graph {
    pub mod bellman_ford;
    pub mod bfs;
    pub mod bipartite;
    pub mod builder;
    pub mod dfs;
    pub mod dijkstra;
    pub mod eulerian_path;
    pub mod floyd_warshall;
    pub mod graph;
    pub mod johnson;
    pub mod low_link;
    pub mod mst;
    pub mod scc;
    pub mod topological_sort;
    pub mod two_sat;
    pub mod zero_one_bfs;
}

pub mod io {
    pub mod fastio;
}

pub mod math {
    pub mod number_theory;
}

pub mod string {
    pub mod run_length;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

// 既存コードの clippy 警告を一時的に許可する。
// TODO: #58 で順次解消し、不要になった allow を削除する。
#![allow(
    clippy::clone_on_copy,
    clippy::doc_lazy_continuation,
    clippy::doc_overindented_list_items,
    clippy::erasing_op,
    clippy::get_first,
    clippy::identity_op,
    clippy::int_plus_one,
    clippy::large_const_arrays,
    clippy::legacy_numeric_constants,
    clippy::len_without_is_empty,
    clippy::manual_is_multiple_of,
    clippy::manual_memcpy,
    clippy::missing_safety_doc,
    clippy::module_inception,
    clippy::needless_range_loop,
    clippy::needless_return,
    clippy::new_without_default,
    clippy::redundant_closure,
    clippy::suspicious_arithmetic_impl,
    clippy::type_complexity,
    clippy::unnecessary_cast,
    clippy::unnecessary_map_or,
    clippy::unnecessary_mut_passed,
    clippy::useless_vec
)]

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
        pub mod bit_segment_tree;
        pub mod lazy_segment_tree;
        pub mod range_affine_range_sum;
        pub mod range_assign_add_min_max_sum;
        pub mod range_bitwise_xor_and_or;
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
    pub mod bipartite_matching;
    pub mod builder;
    pub mod centroid_decomposition;
    pub mod dfs;
    pub mod dijkstra;
    pub mod eulerian_path;
    pub mod flow_graph;
    pub mod floyd_warshall;
    pub mod graph;
    pub mod hld;
    pub mod hld_path_query;
    pub mod johnson;
    pub mod low_link;
    pub mod max_flow;
    pub mod min_cost_flow;
    pub mod min_cost_flow_graph;
    pub mod mst;
    pub mod project_selection;
    pub mod scc;
    pub mod topological_sort;
    pub mod tree_diameter;
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

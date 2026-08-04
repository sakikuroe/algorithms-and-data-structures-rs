pub mod bin {
    // bin/ 配下の統合テストが共有するヘルパー (run_binary) を定義する。
    pub mod common;

    pub mod atcoder {
        pub mod abc422_g_balls_and_boxes;
        pub mod fps24_a_snack;
        pub mod fps24_b_tuple_of_integers;
        pub mod fps24_c_sequence;
        pub mod fps24_d_sequence2;
        pub mod fps24_e_sequence3;
        pub mod fps24_i_score;
        pub mod fps24_k_permutation;
        pub mod fps24_l_permutation2;
        pub mod fps24_m_connected_graph;
        pub mod fps24_n_coin2;
    }

    pub mod library_checker {
        pub mod convolution_mod;
        pub mod exp_of_formal_power_series;
        pub mod exp_of_formal_power_series_sparse;
        pub mod inv_of_formal_power_series;
        pub mod inv_of_formal_power_series_sparse;
        pub mod kth_term_of_linearly_recurrent_sequence;
        pub mod log_of_formal_power_series;
        pub mod log_of_formal_power_series_sparse;
        pub mod partition_function;
        pub mod pow_of_formal_power_series;
        pub mod pow_of_formal_power_series_sparse;
        pub mod product_of_polynomial_sequence;
    }
}

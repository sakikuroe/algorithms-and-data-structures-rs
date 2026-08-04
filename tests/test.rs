pub mod bin {
    // bin/ 配下の統合テストが共有するヘルパー (run_binary) を定義する。
    pub mod common;

    pub mod atcoder {
        pub mod abc075_c_bridge;
        pub mod abc176_d_wizard_in_maze;
        pub mod abc193_f_zebraness;
        pub mod abc209_d_collision;
        pub mod abc225_g_x;
        pub mod abc326_g_unlock_achievement;
        pub mod arc085_e_mul;
        pub mod typical90_an_get_more_money;
        pub mod typical90_aq_maze_challenge_with_lack_of_sleep;
        pub mod typical90_bs_fuzzy_priority;
        pub mod typical90_m_passing;
        pub mod typical90_u_come_back_in_one_piece;
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
        pub mod practice2_d_maxflow;
    }

    pub mod aoj {
        pub mod alds1_11_b_depth_first_search;
        pub mod alds1_11_c_breadth_first_search;
        pub mod grl_1_b_single_source_shortest_path;
        pub mod grl_1_c_all_pairs_shortest_path;
        pub mod grl_1_c_all_pairs_shortest_path_johnson;
        pub mod grl_4_b_topological_sort;
    }

    pub mod yukicoder {
        pub mod yuki2713_just_solitaire;
    }

    pub mod codeforces {
        pub mod cf1082g_petya_and_graph;
    }

    pub mod poj {
        pub mod poj2987_firing;
    }

    pub mod library_checker {
        pub mod assignment;
        pub mod bipartitematching;
        pub mod convolution_mod;
        pub mod cycle_detection;
        pub mod eulerian_trail_directed;
        pub mod exp_of_formal_power_series;
        pub mod exp_of_formal_power_series_sparse;
        pub mod inv_of_formal_power_series;
        pub mod inv_of_formal_power_series_sparse;
        pub mod kth_term_of_linearly_recurrent_sequence;
        pub mod lca;
        pub mod lca_by_euler_tour;
        pub mod log_of_formal_power_series;
        pub mod log_of_formal_power_series_sparse;
        pub mod minimum_spanning_tree;
        pub mod partition_function;
        pub mod pow_of_formal_power_series;
        pub mod pow_of_formal_power_series_sparse;
        pub mod product_of_polynomial_sequence;
        pub mod scc;
        pub mod shortest_path;
        pub mod two_sat;
    }
}

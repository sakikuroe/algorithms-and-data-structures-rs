# Algorithm and data structures in Rust
[![library-test](https://github.com/sakikuroe/algorithms-and-data-structures-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/sakikuroe/algorithms-and-data-structures-rs/actions/workflows/rust.yml) 

# About (概要)

This is a library of algorithms and data structures for competitive programming players.

Rust 言語で書かれた競技プログラミングのためのアルゴリズムとデータ構造のライブラリです。

# Documentation (ドキュメント)

- [Documentation](https://sakikuroe.github.io/algorithms-and-data-structures-rs/algorithms_and_data_structures_rs/index.html)

# License (ライセンス)

This library is available under the CC0 license.

このライブラリはCC0ライセンスで提供されています。

---

You can view the CC0 license at https://creativecommons.org/publicdomain/zero/1.0 .

CC0ライセンスの内容は https://creativecommons.org/publicdomain/zero/1.0/deed.ja で確認できます。

# Usage (使い方)

```
$ cargo add --git https://github.com/sakikuroe/algorithms-and-data-structures-rs
```

# Detail (詳細)

```
$ tree src
src
├── algebraic_structures
│   ├── commutative_group.rs
│   ├── complex_number.rs
│   ├── matrix.rs
│   ├── monoid.rs
│   └── semi_group.rs
├── algebraic_structures.rs
├── algorithms
│   ├── bit.rs
│   ├── compress.rs
│   ├── dp
│   │   ├── lis_lds.rs
│   │   └── rerooting.rs
│   ├── dp.rs
│   ├── game
│   │   └── grundy.rs
│   ├── game.rs
│   ├── geometry
│   │   ├── circle.rs
│   │   ├── convex_hull.rs
│   │   └── set_of_points.rs
│   ├── geometry.rs
│   ├── graph
│   │   ├── bellman_ford.rs
│   │   ├── bfs.rs
│   │   ├── bipartite.rs
│   │   ├── burn_bury_problem.rs
│   │   ├── cow_game.rs
│   │   ├── dijkstra.rs
│   │   ├── floyd_warshall.rs
│   │   ├── kruskal.rs
│   │   ├── lca.rs
│   │   ├── max_flow.rs
│   │   ├── namori.rs
│   │   ├── scc.rs
│   │   ├── topological_sort.rs
│   │   ├── traveling_salesman_problem.rs
│   │   ├── tree
│   │   │   ├── diameter.rs
│   │   │   └── max_matching_on_tree.rs
│   │   └── tree.rs
│   ├── graph.rs
│   ├── inversion_number.rs
│   ├── number_theory
│   │   ├── baby_step_giant_step.rs
│   │   ├── binomical_coefficient_arbitrary.rs
│   │   ├── binomical_coefficient.rs
│   │   ├── discrete_logarithm.rs
│   │   ├── floor_sum.rs
│   │   ├── gcd_lcm.rs
│   │   ├── gen_divisors.rs
│   │   ├── integer_factorization.rs
│   │   ├── is_prime.rs
│   │   ├── kth_root.rs
│   │   ├── ntt_1000000007.rs
│   │   ├── ntt.rs
│   │   ├── pow.rs
│   │   ├── sieve_of_eratosthenes.rs
│   │   └── stirling2.rs
│   ├── number_theory.rs
│   ├── permutation.rs
│   ├── search
│   │   └── binary_search.rs
│   ├── search.rs
│   ├── string
│   │   ├── aho_corasick.rs
│   │   ├── correct_bracket.rs
│   │   ├── rolling_hash.rs
│   │   ├── run_length.rs
│   │   ├── suffix_array.rs
│   │   └── z_algorithm.rs
│   └── string.rs
├── algorithms.rs
├── data_structures
│   ├── convex_hull_trick.rs
│   ├── counter.rs
│   ├── disjoint_sparse_table.rs
│   ├── double_ended_priority_queue.rs
│   ├── fenwick_tree.rs
│   ├── graph.rs
│   ├── lazy_segment_tree.rs
│   ├── modint_arbitrary.rs
│   ├── modint.rs
│   ├── persistent_stack.rs
│   ├── segment_tree2d.rs
│   ├── segment_tree.rs
│   ├── sliding_window_aggregation.rs
│   ├── splay_bst lazy.rs
│   ├── succinct_bit_vector.rs
│   ├── union_find_potential.rs
│   ├── union_find.rs
│   └── wavelet_matrix.rs
├── data_structures.rs
├── lib.rs
└── macros.rs
```
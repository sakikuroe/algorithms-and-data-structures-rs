//! 競技プログラミング向けのアルゴリズムとデータ構造を提供するライブラリーです。
//!
//! 区間集約や順位検索、グラフの探索・最短路・フロー、整数論、
//! 整数座標の二次元幾何、法 998244353 上の演算を扱います。列の連長圧縮と標準入出力の機能も含みます。

pub mod algebra {
    //! 半群・モノイドを表すトレイトと実装
    //!
    //! 結合則を満たす演算は [`semi_group::SemiGroup`]、単位元も持つ演算は
    //! [`monoid::Monoid`] で表します。演算が可換であることは
    //! [`monoid::CommutativeMonoid`] で表します。
    //!
    //! 加算・最小値・最大値などの実装を提供しており、これらのトレイトを実装して
    //! 独自の集約演算を定義することもできます。

    pub mod monoid;
    pub mod semi_group;
}

pub mod modulo998244353 {
    //! 法 998244353 上の剰余演算、畳み込み、形式的べき級数
    //!
    //! [`modint::ModInt998244353`] は剰余を表す型で、四則演算や累乗に対応します。
    //! [`combinatorics::Combinatorics`] は階乗とその逆元を前計算し、二項係数などを求めます。
    //!
    //! 係数列の畳み込みは [`convolution`]、逆元・対数・指数関数などの
    //! 形式的べき級数の演算は [`fps::FPS`] が提供します。
    //! Montgomery 表現を用いる演算は [`convolution_mont`] にあります。

    pub mod combinatorics;
    pub mod convolution;
    mod convolution_avx2;
    pub mod convolution_mont;
    pub mod fps;
    pub mod modint;
    pub mod modulo;
}

pub mod ds {
    //! 区間集約、順位検索、集合の併合のためのデータ構造
    //!
    //! [`segment_tree`] は一点・区間の更新と、区間・矩形の集約を扱います。
    //! 静的な列の区間内で値の大小に基づく検索を行う
    //! [`wavelet_matrix::WaveletMatrix`] と、値の和も求める
    //! [`wavelet_matrix_with_sum::WaveletMatrixWithSum`] も提供します。
    //!
    //! [`union_find::UnionFind`] は集合の併合と連結判定を行います。
    //! [`bit_vector::BitVector`] はビット列を保持し、指定位置より前にある 1 の個数を求めます。

    pub mod segment_tree {
        //! 一点・区間の更新と、区間・矩形の集約を行うセグメント木です。
        //!
        //! 更新の種類、座標の範囲、更新位置を事前に列挙できるかに応じた実装を提供します。
        //!
        //! # 一点更新と区間集約
        //!
        //! [`segment_tree_dense::SegmentTreeDense`] は配列の各要素を保持します。
        //! 座標の範囲が大きく、更新する位置が少ない場合には、疎な実装を利用できます。
        //! [`segment_tree_sparse_offline::SegmentTreeSparseOffline`] は更新位置を事前登録し、
        //! [`segment_tree_sparse_online::SegmentTreeSparseOnline`] は更新時に必要な節点を生成します。
        //! いずれも集約演算を [`Monoid`](crate::algebra::monoid::Monoid) で指定します。
        //!
        //! # 区間更新と区間集約
        //!
        //! 遅延セグメント木は、区間全体に作用を適用します。
        //! [`lazy_segment_tree::SegmentTreeLazyDense`] は配列全体を保持し、
        //! [`lazy_segment_tree_sparse_offline::SegmentTreeLazySparseOffline`] は
        //! 更新区間の端点を事前登録します。
        //! [`lazy_segment_tree_sparse_online::SegmentTreeLazySparseOnline`] は
        //! 更新時に必要な節点を生成するため、端点の事前登録は不要です。
        //!
        //! 作用と集約を定義済みの型として、区間の代入・加算と最小値・最大値・和を扱う
        //! [`range_assign_add_min_max_sum`]、法 998244353 上の区間アフィン変換と和を扱う
        //! [`range_affine_range_sum`]、整数のビット演算を扱う [`range_bitwise_xor_and_or`]、
        //! 真偽値の設定・反転と個数を扱う [`bit_segment_tree`] があります。
        //!
        //! # 二次元の矩形集約
        //!
        //! [`segment_tree_2d_dense`] は格子全体を保持します。
        //! 疎な点集合には、更新位置を事前登録する [`segment_tree_2d_sparse_offline`] と、
        //! 実行時に追加する [`segment_tree_2d_sparse_online`] があります。
        //! 矩形の集約には [`CommutativeMonoid`](crate::algebra::monoid::CommutativeMonoid) が必要です。
        //!
        //! # Examples
        //!
        //! 加算モノイドを用いた一点更新と区間和です。
        //!
        //! ```rust
        //! use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_dense};
        //!
        //! let mut tree = segment_tree_dense::SegmentTreeDense::<monoid::AddMonoid>::new(3);
        //! tree.update(0, 4);
        //! tree.update(2, 7);
        //! assert_eq!(11, tree.fold(..));
        //! assert_eq!(7, tree.fold(1..3));
        //! ```

        pub mod bit_segment_tree;
        pub mod lazy_segment_tree;
        pub mod lazy_segment_tree_sparse_offline;
        pub mod lazy_segment_tree_sparse_online;
        pub mod range_affine_range_sum;
        pub mod range_assign_add_min_max_sum;
        pub mod range_bitwise_xor_and_or;
        mod range_bounds;
        pub mod segment_tree_2d_dense;
        pub mod segment_tree_2d_sparse_offline;
        pub mod segment_tree_2d_sparse_online;
        pub mod segment_tree_dense;
        pub mod segment_tree_sparse_offline;
        pub mod segment_tree_sparse_online;
    }
    pub mod bit_vector;
    pub mod interval_bit_set;
    pub mod union_find;
    pub mod wavelet_matrix;
    mod wavelet_matrix_range;
    pub mod wavelet_matrix_with_sum;
}

pub mod geometry {
    //! 整数座標の点・方向、偏角順と凸多角形の操作
    //!
    //! [`primitives`] は点とベクトル、[`predicates`] は向きと線分の関係を扱います。
    //! [`direction`] は非ゼロ方向の比較と集約キーを提供します。
    //! [`argument`] は元添字を保つ方向列と円周上の検索、
    //! [`convex_polygon`] は凸包・支持面・内外判定・最遠点対・Minkowski 和を提供します。

    pub mod argument;
    pub mod convex_polygon;
    pub mod direction;
    mod kernel;
    pub mod predicates;
    pub mod primitives;
}

pub mod graph {
    //! グラフの表現と、探索・最短路・連結性・フローのアルゴリズム
    //!
    //! [`graph::Graph`] は、整数の頂点番号と辺に付随する値を隣接リストで保持します。
    //! [`builder::GraphBuilder`] を用いると、文字列やタプルなどのラベルから頂点番号を割り当て、
    //! グラフを構築できます。
    //!
    //! 単一始点の最短路には、重みなしグラフの [`bfs`]、重みが 0 または 1 の [`zero_one_bfs`]、
    //! 非負の重みの [`dijkstra`]、負の重みも扱う [`bellman_ford`] があります。
    //! 全点対の最短路は [`floyd_warshall`] と [`johnson`] が提供します。
    //!
    //! 木のパスの区間分解は [`hld::Hld`]、パス上の集約は [`hld_path_query`]、
    //! 強連結成分分解は [`scc`]、橋・関節点の検出は [`low_link`]、最小全域木は [`mst`] が扱います。
    //! 最大流・最小費用流には [`max_flow`]・[`min_cost_flow`]、
    //! 二部マッチングには [`bipartite_matching`] があります。
    //! また、最適化問題を最小カットへ帰着する [`project_selection`] と、
    //! 2-SAT の充足判定を行う [`two_sat`] を提供します。

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
    // 汎用グラフ構造体を提供するモジュールであり、`graph::graph` という
    // 名称は意図的であるため、clippy の警告を抑制する。
    #[allow(clippy::module_inception)]
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
    //! 標準入力の読み取りと標準出力への書き出し
    //!
    //! [`fastio::Fastio`] が入力の解析とバッファーを用いた出力を提供します。

    pub mod fastio;
}

pub mod math {
    //! 整数論、合同式、数列の和に関するアルゴリズム
    //!
    //! [`number_theory`] は最大公約数・最小公倍数・拡張 Euclid 法、
    //! [`modular_arithmetic`] は剰余演算と中国剰余定理を提供します。
    //! 素数判定・素因数分解・原始根は [`primality`]、上限までの素数列挙は [`sieve`] が扱います。
    //!
    //! 整数の乗根は [`kth_root`]、等差・等比数列に関する和は [`series`] にあります。

    pub mod kth_root;
    pub mod modular_arithmetic;
    pub mod number_theory;
    pub mod primality;
    pub mod series;
    pub mod sieve;
}

pub mod string {
    //! 列の連長圧縮と復元
    //!
    //! [`run_length::RunLengthEncode`] は連続する同じ値を、値と個数の組にまとめます。
    //! [`run_length::RunLengthDecode`] はその組の列を元の列に戻します。

    pub mod run_length;
}

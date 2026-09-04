//! [`Hld`] が返す向き付きの区間 ([`PathDirection`]) を使って、パス上の値を
//! モノイドで畳み込むための補助モジュールである。
//!
//! `Hld` 自体は木の形 (親・深さ・チェーン分解) だけを保持する非ジェネリックな
//! 型であり、値の集約方法 (モノイド) には一切関知しない。これにより、1つの
//! `Hld` を、頂点の和・辺の最小値など、複数の異なるモノイドで同時に使い
//! 回せる (分解をモノイドの数だけ再計算せずに済む)。
//!
//! 値をどこに置くか (頂点のみ・辺のみ・両方を交互に) によって、以下の3つの
//! 型を使い分ける。頂点値と辺値は内部で同じスロットを共有できないため、
//! 「頂点用に構築したのに辺の値を書き込む」といった取り違えを実行時の
//! サイレントな上書きではなく、型の選択そのもので防ぐことを狙っている。
//!
//! - [`HldVertexPathQuery`][]: 頂点にのみ値を持つパスクエリ。
//! - [`HldEdgePathQuery`][]: 辺にのみ値を持つパスクエリ。辺の値は、その子側
//!   (深い側) の頂点のスロットに保持する。
//! - [`HldVertexEdgePathQuery`][]: 頂点と辺の両方に値を持ち、`u` から `v` へ
//!   の経路を「頂点, 辺, 頂点, 辺, ..., 頂点」と交互に畳み込む。内部の
//!   スロット数を2倍にし、頂点 `v` の値を `2*id(v)`、`v` から親への辺の値を
//!   `2*id(v)-1` に置く。区間分解は [`Hld::vertex_edge_path_ranges`] が
//!   担う。
//!
//! いずれの型も、非可換なモノイド (行列積・アフィン関数の合成など、経路の
//! 向きによって結果が変わる演算) に対応するため、「番号の昇順」と「番号を
//! 反転させた降順」の2本のセグメント木を内部で保持する。可換なモノイド
//! (総和など) しか使わない場合でも、この2本立ての構造はそのまま使い回せる。

use super::super::algebra::monoid::Monoid;
use super::super::ds::segment_tree::segment_tree_dense::SegmentTreeDense;
use super::hld::{Hld, PathDirection};

/// `forward`/`reversed` の2本のセグメント木を保持し、向き付き区間列の
/// 畳み込みを行う内部実装である。[`HldVertexPathQuery`]・
/// [`HldEdgePathQuery`]・[`HldVertexEdgePathQuery`] の3つの公開型が、それぞれ
/// スロット番号の割り当て方だけを変えてこの構造体を内部で共有する。
struct PathFoldStore<M: Monoid>
where
    M::S: Clone,
{
    forward: SegmentTreeDense<M>,
    reversed: SegmentTreeDense<M>,
}

impl<M: Monoid> PathFoldStore<M>
where
    M::S: Clone,
{
    /// `len` 個のスロットを持つ、初期値がすべて単位元の状態で用意する。
    fn new(len: usize) -> Self {
        Self {
            forward: SegmentTreeDense::<M>::new(len),
            reversed: SegmentTreeDense::<M>::new(len),
        }
    }

    /// [`build`](Self::build) より前に呼び出し、スロット `pos` の値を `x` に
    /// 設定する。
    fn set_before_build(&mut self, pos: usize, x: M::S) {
        let len = self.forward.len();
        self.forward.set(pos, x.clone());
        self.reversed.set(len - 1 - pos, x);
    }

    /// [`set_before_build`](Self::set_before_build) による設定を反映させる。
    fn build(&mut self) {
        self.forward.build();
        self.reversed.build();
    }

    /// スロット `pos` の値を `x` に変更する。
    fn set(&mut self, pos: usize, x: M::S) {
        let len = self.forward.len();
        self.forward.update(pos, x.clone());
        self.reversed.update(len - 1 - pos, x);
    }

    /// スロット `pos` の現在の値を返す。
    fn get(&self, pos: usize) -> M::S {
        self.forward.get(pos)
    }

    /// 向き付きの区間の列を、その順序を保ったまま畳み込む。
    fn fold(&self, ranges: &[(usize, usize, PathDirection)]) -> M::S {
        let n = self.forward.len();
        // 区間の列を先頭から順に畳み込んでいく。acc がここまでの畳み込み
        // 結果であり、区間ごとの値を M::op で右から結合していく。
        ranges.iter().fold(M::id(), |acc, &(l, r, dir)| {
            // 区間の向きに応じて、参照するセグメント木を使い分ける。番号の
            // 昇順に読みたい区間 (Forward) はそのまま forward から、降順に
            // 読みたい区間 (Reversed) は、番号を反転させて構築してある
            // reversed から、対応する反転後の区間 [n-r, n-l) を取り出す。
            let value = match dir {
                PathDirection::Forward => self.forward.fold(l, r),
                PathDirection::Reversed => self.reversed.fold(n - r, n - l),
            };
            M::op(&acc, &value)
        })
    }
}

/// `u`,`v` のうち、辺の子側 (深い側) の頂点を返す。`u`,`v` が隣接した頂点
/// (辺で直接結ばれている) でなければならない。
fn resolve_edge_child(hld: &Hld, u: usize, v: usize) -> usize {
    let child = if hld.depth(u) > hld.depth(v) { u } else { v };
    let parent = if child == u { v } else { u };
    debug_assert_eq!(
        Some(parent),
        hld.kth_ancestor(child, 1),
        "u と v は隣接した頂点 (辺で直接結ばれている) でなければならない"
    );
    child
}

/// [`Hld`] を借用し、非可換なモノイド `M` による「頂点」のパスクエリを
/// 処理する。
pub struct HldVertexPathQuery<'a, M: Monoid>
where
    M::S: Clone,
{
    hld: &'a Hld,
    store: PathFoldStore<M>,
}

impl<'a, M: Monoid> HldVertexPathQuery<'a, M>
where
    M::S: Clone,
{
    /// `hld` を借用し、各頂点の初期値から `HldVertexPathQuery` を構築する。
    ///
    /// # Args
    /// - `hld`: 対象の木の HLD
    /// - `values`: `values[v]` が頂点 `v` の初期値であり、長さは `hld` の
    ///   頂点数と一致していなければならない。
    ///
    /// # Panics
    /// - `values.len()` が `hld` の頂点数と一致しない場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    pub fn new(hld: &'a Hld, values: &[M::S]) -> Self {
        assert_eq!(
            hld.vertex_count(),
            values.len(),
            "values.len() は Hld の頂点数と一致していなければならない"
        );
        let mut store = PathFoldStore::new(values.len());
        for (v, value) in values.iter().enumerate() {
            store.set_before_build(hld.vertex_id(v), value.clone());
        }
        store.build();
        Self { hld, store }
    }

    /// 頂点 `v` の値を `x` に変更する。
    ///
    /// # Args
    /// - `v`: 対象の頂点
    /// - `x`: 新しい値
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_vertex(&mut self, v: usize, x: M::S) {
        self.store.set(self.hld.vertex_id(v), x);
    }

    /// 頂点 `v` の現在の値を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点
    ///
    /// # Returns
    /// `M::S`: 頂点 `v` の現在の値
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_vertex(&self, v: usize) -> M::S {
        self.store.get(self.hld.vertex_id(v))
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての頂点の値を、`u` から
    /// `v` へ向かう順に畳み込む。
    ///
    /// # Args
    /// - `u`: パスの起点
    /// - `v`: パスの終点
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn fold_vertex_path(&self, u: usize, v: usize) -> M::S {
        self.store.fold(&self.hld.vertex_path_ranges(u, v))
    }
}

/// [`Hld`] を借用し、非可換なモノイド `M` による「辺」のパスクエリを処理
/// する。辺の値は、その子側 (深い側) の頂点のスロットに保持する。
pub struct HldEdgePathQuery<'a, M: Monoid>
where
    M::S: Clone,
{
    hld: &'a Hld,
    store: PathFoldStore<M>,
}

impl<'a, M: Monoid> HldEdgePathQuery<'a, M>
where
    M::S: Clone,
{
    /// `hld` を借用し、すべての辺の値を単位元とした `HldEdgePathQuery` を
    /// 構築する。各辺の実際の初期値は、構築後に [`set_edge`](Self::set_edge)
    /// で設定する。
    ///
    /// # Args
    /// - `hld`: 対象の木の HLD
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    pub fn new(hld: &'a Hld) -> Self {
        let n = hld.vertex_count();
        let mut store = PathFoldStore::new(n);
        for pos in 0..n {
            store.set_before_build(pos, M::id());
        }
        store.build();
        Self { hld, store }
    }

    /// 辺 `{u, v}` の値を `x` に変更する。
    ///
    /// # Args
    /// - `u`/`v`: 辺の両端点であり、隣接した頂点でなければならない。
    /// - `x`: 新しい値
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_edge(&mut self, u: usize, v: usize, x: M::S) {
        let child = resolve_edge_child(self.hld, u, v);
        self.store.set(self.hld.vertex_id(child), x);
    }

    /// 辺 `{u, v}` の現在の値を返す。
    ///
    /// # Args
    /// - `u`/`v`: 辺の両端点であり、隣接した頂点でなければならない。
    ///
    /// # Returns
    /// `M::S`: 辺 `{u, v}` の現在の値
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_edge(&self, u: usize, v: usize) -> M::S {
        let child = resolve_edge_child(self.hld, u, v);
        self.store.get(self.hld.vertex_id(child))
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての辺の値を、`u` から `v`
    /// へ向かう順に畳み込む。
    ///
    /// # Args
    /// - `u`: パスの起点
    /// - `v`: パスの終点
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn fold_edge_path(&self, u: usize, v: usize) -> M::S {
        self.store.fold(&self.hld.edge_path_ranges(u, v))
    }
}

/// [`Hld`] を借用し、非可換なモノイド `M` による「頂点と辺の両方」の
/// パスクエリを処理する。`u` から `v` へのパスを、頂点 `u`、その先の辺、
/// 隣の頂点、その先の辺、……、頂点 `v` という順に、頂点と辺を交互に
/// 畳み込む。
///
/// 内部では、頂点 `v` の値をスロット `2*id(v)`、`v` から親への辺の値を
/// スロット `2*id(v)-1` に置くことで、頂点のみのスロット数を2倍にした
/// 上で [`Hld::vertex_path_ranges`] が返す区間をそのまま2倍に引き伸ばして
/// 使う。これにより、頂点用の区間分解ロジックをそのまま再利用できる。
pub struct HldVertexEdgePathQuery<'a, M: Monoid>
where
    M::S: Clone,
{
    hld: &'a Hld,
    store: PathFoldStore<M>,
}

impl<'a, M: Monoid> HldVertexEdgePathQuery<'a, M>
where
    M::S: Clone,
{
    /// `hld` を借用し、各頂点の初期値から `HldVertexEdgePathQuery` を
    /// 構築する。各辺の値は単位元で初期化され、実際の初期値は構築後に
    /// [`set_edge`](Self::set_edge) で設定する。
    ///
    /// # Args
    /// - `hld`: 対象の木の HLD
    /// - `values`: `values[v]` が頂点 `v` の初期値であり、長さは `hld` の
    ///   頂点数と一致していなければならない。
    ///
    /// # Panics
    /// - `values.len()` が `hld` の頂点数と一致しない場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    pub fn new(hld: &'a Hld, values: &[M::S]) -> Self {
        assert_eq!(
            hld.vertex_count(),
            values.len(),
            "values.len() は Hld の頂点数と一致していなければならない"
        );
        let n = values.len();
        let len = if n == 0 { 0 } else { 2 * n - 1 };
        let mut store = PathFoldStore::new(len);
        for (v, value) in values.iter().enumerate() {
            store.set_before_build(2 * hld.vertex_id(v), value.clone());
        }
        for v in 0..n {
            let id = hld.vertex_id(v);
            if id != 0 {
                // 根以外の頂点は、親への辺のスロットを単位元で初期化しておく
                // (根には対応する辺が存在しない)。
                store.set_before_build(2 * id - 1, M::id());
            }
        }
        store.build();
        Self { hld, store }
    }

    /// 頂点 `v` の値を `x` に変更する。
    ///
    /// # Args
    /// - `v`: 対象の頂点
    /// - `x`: 新しい値
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_vertex(&mut self, v: usize, x: M::S) {
        self.store.set(2 * self.hld.vertex_id(v), x);
    }

    /// 頂点 `v` の現在の値を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点
    ///
    /// # Returns
    /// `M::S`: 頂点 `v` の現在の値
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_vertex(&self, v: usize) -> M::S {
        self.store.get(2 * self.hld.vertex_id(v))
    }

    /// 辺 `{u, v}` の値を `x` に変更する。
    ///
    /// # Args
    /// - `u`/`v`: 辺の両端点であり、隣接した頂点でなければならない。
    /// - `x`: 新しい値
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_edge(&mut self, u: usize, v: usize, x: M::S) {
        let child = resolve_edge_child(self.hld, u, v);
        self.store.set(2 * self.hld.vertex_id(child) - 1, x);
    }

    /// 辺 `{u, v}` の現在の値を返す。
    ///
    /// # Args
    /// - `u`/`v`: 辺の両端点であり、隣接した頂点でなければならない。
    ///
    /// # Returns
    /// `M::S`: 辺 `{u, v}` の現在の値
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_edge(&self, u: usize, v: usize) -> M::S {
        let child = resolve_edge_child(self.hld, u, v);
        self.store.get(2 * self.hld.vertex_id(child) - 1)
    }

    /// 頂点 `u` から `v` へのパス上にある、頂点と辺の値を、`u`,
    /// (`u` の次の辺), (次の頂点), ……, `v` という順に交互に畳み込む。
    ///
    /// # Args
    /// - `u`: パスの起点
    /// - `v`: パスの終点
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn fold_vertex_edge_path(&self, u: usize, v: usize) -> M::S {
        self.store.fold(&self.hld.vertex_edge_path_ranges(u, v))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::algebra::monoid::AffineMonoid;
    use super::super::graph::Graph;
    use super::*;

    type Affine = AffineMonoid<998244353>;

    /// Background: 0-1-2 の単純パス。
    fn create_path() -> Graph<()> {
        let mut g = Graph::new(3);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(1, 2, ());
        g
    }

    /// Background: 0 を根とし、0-1, 0-2, 1-3, 1-4, 3-5 の辺を持つ木。
    ///
    /// ```text
    ///         0
    ///        / \
    ///       1   2
    ///      / \
    ///     3   4
    ///     |
    ///     5
    /// ```
    fn create_tree() -> Graph<()> {
        let mut g = Graph::new(6);
        g.add_undirected_edge(0, 1, ());
        g.add_undirected_edge(0, 2, ());
        g.add_undirected_edge(1, 3, ());
        g.add_undirected_edge(1, 4, ());
        g.add_undirected_edge(3, 5, ());
        g
    }

    // HldVertexPathQuery::new/get_vertex のテスト: 構築直後の状態を検証する。
    mod hld_vertex_path_query_new {
        use super::*;

        /// Scenario: 構築直後は、各頂点の値がそのまま取得できる。
        /// - Given: 3頂点の単純パス (0-1-2) があり、それぞれ異なる
        ///   一次関数 f(x)=2x+1, f(x)=3x+0, f(x)=1x+5 を初期値として持つ。
        /// - When: `HldVertexPathQuery` を構築する。
        /// - Then: 各頂点の値が、初期値のまま `get_vertex` で取得できる。
        #[test]
        fn returns_initial_values() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            // When
            let sut = HldVertexPathQuery::<Affine>::new(&hld, &values);
            // Then
            assert_eq!((2, 1), sut.get_vertex(0));
            assert_eq!((3, 0), sut.get_vertex(1));
            assert_eq!((1, 5), sut.get_vertex(2));
        }

        /// Scenario: 初期値の個数が頂点数と一致しない場合、パニックする
        /// (異常系)。
        /// - Given: 3頂点の単純パスがあり、初期値は2個しかない。
        /// - When: `HldVertexPathQuery` を構築しようとする。
        /// - Then: パニックする。
        #[test]
        #[should_panic]
        fn panics_when_values_len_mismatches_vertex_count() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(1_u64, 0_u64), (1, 0)];
            // When, Then (panic)
            HldVertexPathQuery::<Affine>::new(&hld, &values);
        }
    }

    // HldVertexPathQuery::fold_vertex_path のテスト: 戻り値そのもの (畳み込み結果) を検証する。
    mod hld_vertex_path_query_fold_vertex_path {
        use super::*;

        /// Scenario: 非可換な合成では、パスの向きによって結果が異なる。
        /// - Given: 3頂点の単純パス (0-1-2) があり、それぞれ
        ///   f(x)=2x+1, f(x)=3x+0, f(x)=1x+5 を持つ。
        /// - When: 0から2へのパスと、2から0へのパスをそれぞれ畳み込む。
        /// - Then: 0→2 は f_2(f_1(f_0(x))) = 6x+8、2→0 は
        ///   f_0(f_1(f_2(x))) = 6x+31 に対応する係数になり、両者は一致
        ///   しない。
        #[test]
        fn produces_direction_dependent_result_for_noncommutative_monoid() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let sut = HldVertexPathQuery::<Affine>::new(&hld, &values);
            // When
            let forward = sut.fold_vertex_path(0, 2);
            let backward = sut.fold_vertex_path(2, 0);
            // Then
            assert_eq!((6, 8), forward);
            assert_eq!((6, 31), backward);
        }

        /// Scenario: 頂点の値を更新すると、以降の畳み込み結果に反映される。
        /// - Given: 3頂点の単純パス (0-1-2) があり、それぞれ
        ///   f(x)=2x+1, f(x)=3x+0, f(x)=1x+5 を持つ。
        /// - When: 頂点1の値を f(x)=5x+2 に更新してから、0から2への
        ///   パスを畳み込む。
        /// - Then: f_2(f_1'(f_0(x))) = 10x+12 に対応する係数になる。
        #[test]
        fn reflects_vertex_update_in_later_folds() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let mut sut = HldVertexPathQuery::<Affine>::new(&hld, &values);
            // When
            sut.set_vertex(1, (5, 2));
            let result = sut.fold_vertex_path(0, 2);
            // Then
            assert_eq!((10, 12), result);
        }

        /// Scenario: 起点と終点が同じ場合、その頂点自身の値になる (境界値)。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: 頂点1から頂点1自身へのパスを畳み込む。
        /// - Then: 頂点1の値そのものになる。
        #[test]
        fn returns_single_vertex_value_for_same_endpoint() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let sut = HldVertexPathQuery::<Affine>::new(&hld, &values);
            // When
            let result = sut.fold_vertex_path(1, 1);
            // Then
            assert_eq!((3, 0), result);
        }
    }

    // HldEdgePathQuery::new/set_edge/get_edge のテスト: 構築直後・更新後の状態を検証する。
    mod hld_edge_path_query_set_edge {
        use super::*;

        /// Scenario: 構築直後は、すべての辺の値が単位元になる。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: `HldEdgePathQuery` を構築する。
        /// - Then: 各辺の値が単位元 (恒等関数 f(x)=x) になる。
        #[test]
        fn initializes_all_edges_to_identity() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            // When
            let sut = HldEdgePathQuery::<Affine>::new(&hld);
            // Then
            assert_eq!((1, 0), sut.get_edge(0, 1));
            assert_eq!((1, 0), sut.get_edge(1, 2));
        }

        /// Scenario: 辺の値を設定すると、`get_edge` に反映される。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: 辺 {0,1} の値を f(x)=2x+1 に設定する。
        /// - Then: `get_edge(0,1)`、および端点を入れ替えた `get_edge(1,0)`
        ///   の両方が新しい値を返す。
        #[test]
        fn reflects_set_edge_in_get_edge_regardless_of_endpoint_order() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let mut sut = HldEdgePathQuery::<Affine>::new(&hld);
            // When
            sut.set_edge(0, 1, (2, 1));
            // Then
            assert_eq!((2, 1), sut.get_edge(0, 1));
            assert_eq!((2, 1), sut.get_edge(1, 0));
        }
    }

    // HldEdgePathQuery::fold_edge_path のテスト: 戻り値そのもの (畳み込み結果) を検証する。
    mod hld_edge_path_query_fold_edge_path {
        use super::*;

        /// Scenario: 非可換な合成では、パスの向きによって結果が異なる。
        /// - Given: 3頂点の単純パス (0-1-2) があり、辺 {0,1} は
        ///   f(x)=2x+1、辺 {1,2} は f(x)=3x+5 を持つ。
        /// - When: 0から2へのパスと、2から0へのパスをそれぞれ畳み込む。
        /// - Then: 0→2 は g(f(x)) = 3(2x+1)+5 = 6x+8、2→0 は
        ///   f(g(x)) = 2(3x+5)+1 = 6x+11 に対応する係数になり、両者は
        ///   一致しない。
        #[test]
        fn produces_direction_dependent_result_for_noncommutative_monoid() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let mut sut = HldEdgePathQuery::<Affine>::new(&hld);
            sut.set_edge(0, 1, (2, 1));
            sut.set_edge(1, 2, (3, 5));
            // When
            let forward = sut.fold_edge_path(0, 2);
            let backward = sut.fold_edge_path(2, 0);
            // Then
            assert_eq!((6, 8), forward);
            assert_eq!((6, 11), backward);
        }

        /// Scenario: 起点と終点が同じ場合、辺は1本も含まれないため単位元
        /// になる (境界値)。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: 頂点1から頂点1自身への辺パスを畳み込む。
        /// - Then: 単位元 (恒等関数 f(x)=x) になる。
        #[test]
        fn returns_identity_for_same_endpoint() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let mut sut = HldEdgePathQuery::<Affine>::new(&hld);
            sut.set_edge(0, 1, (2, 1));
            // When
            let result = sut.fold_edge_path(1, 1);
            // Then
            assert_eq!((1, 0), result);
        }
    }

    // HldVertexEdgePathQuery::fold_vertex_edge_path のテスト: 頂点・辺を交互に畳み込む戻り値を検証する。
    mod hld_vertex_edge_path_query_fold_vertex_edge_path {
        use super::*;

        /// Scenario: 単純パス上で、頂点と辺の値が正しい順序で畳み込まれる。
        /// - Given: 3頂点の単純パス (0-1-2) があり、頂点0,1,2はそれぞれ
        ///   f(x)=2x+0, f(x)=3x+0, f(x)=5x+0、辺{0,1}はf(x)=1x+1、
        ///   辺{1,2}はf(x)=1x+2 を持つ。
        /// - When: 0から2へのパスを畳み込む。
        /// - Then: 頂点0,辺{0,1},頂点1,辺{1,2},頂点2 の順に合成した
        ///   g(x) = f_v2(f_e12(f_v1(f_e01(f_v0(x))))) と一致する
        ///   (f_v0(x)=2x, f_e01(x)=x+1 -> 2x+1, f_v1(x)=3x -> 6x+3,
        ///   f_e12(x)=x+2 -> 6x+5, f_v2(x)=5x -> 30x+25 なので (30,25))。
        #[test]
        fn interleaves_vertex_and_edge_values_in_traversal_order() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 0_u64), (3, 0), (5, 0)];
            let mut sut = HldVertexEdgePathQuery::<Affine>::new(&hld, &values);
            sut.set_edge(0, 1, (1, 1));
            sut.set_edge(1, 2, (1, 2));
            // When
            let result = sut.fold_vertex_edge_path(0, 2);
            // Then
            assert_eq!((30, 25), result);
        }

        /// Scenario: LCA を経由する経路でも、頂点と辺の値が正しい順序で
        /// 畳み込まれる (複数のチェーンをまたぐ)。
        /// - Given: 上記の木があり、頂点5から頂点4へのパス (5-3-1-4、
        ///   経路は 5,{5,3},3,{3,1},1,{1,4},4) の各頂点・辺すべてに
        ///   異なる一次関数を設定する。
        /// - When: 5から4へのパスを畳み込む。
        /// - Then: 手計算で合成した結果と一致する。
        #[test]
        fn interleaves_vertex_and_edge_values_across_lca() {
            // Given
            let g = create_tree();
            let hld = g.try_hld(0).unwrap();
            // 頂点0..5の値。畳み込みに関与するのは 5,3,1,4 のみである。
            let values = vec![
                (1_u64, 0_u64),
                (7, 1), // 頂点1: f(x) = 7x+1
                (1, 0),
                (2, 3),  // 頂点3: f(x) = 2x+3
                (11, 5), // 頂点4: f(x) = 11x+5
                (4, 2),  // 頂点5: f(x) = 4x+2
            ];
            let mut sut = HldVertexEdgePathQuery::<Affine>::new(&hld, &values);
            sut.set_edge(5, 3, (1, 6)); // 辺{5,3}: f(x) = x+6
            sut.set_edge(3, 1, (3, 0)); // 辺{3,1}: f(x) = 3x
            sut.set_edge(1, 4, (1, 9)); // 辺{1,4}: f(x) = x+9
            // When
            let result = sut.fold_vertex_edge_path(5, 4);
            // Then
            // f_v5(x) = 4x+2
            // f_e53(f_v5(x)) = (4x+2)+6 = 4x+8
            // f_v3(...) = 2(4x+8)+3 = 8x+19
            // f_e31(...) = 3(8x+19) = 24x+57
            // f_v1(...) = 7(24x+57)+1 = 168x+400
            // f_e14(...) = (168x+400)+9 = 168x+409
            // f_v4(...) = 11(168x+409)+5 = 1848x+4504
            assert_eq!((1848, 4504), result);
        }

        /// Scenario: 起点と終点が同じ場合、その頂点自身の値のみになる
        /// (境界値)。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: 頂点1から頂点1自身へのパスを畳み込む。
        /// - Then: 頂点1の値そのものになり、辺は含まれない。
        #[test]
        fn returns_single_vertex_value_for_same_endpoint() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 0_u64), (3, 1), (5, 0)];
            let mut sut = HldVertexEdgePathQuery::<Affine>::new(&hld, &values);
            sut.set_edge(0, 1, (1, 100));
            // When
            let result = sut.fold_vertex_edge_path(1, 1);
            // Then
            assert_eq!((3, 1), result);
        }

        /// Scenario: 頂点・辺の値を更新すると、以降の畳み込み結果に反映
        /// される。
        /// - Given: 3頂点の単純パス (0-1-2) がある。
        /// - When: 頂点1の値と辺{1,2}の値を更新してから、0から2への
        ///   パスを畳み込む。
        /// - Then: 更新後の値で合成した結果になる。
        #[test]
        fn reflects_updates_in_later_folds() {
            // Given
            let g = create_path();
            let hld = g.try_hld(0).unwrap();
            let values = vec![(1_u64, 0_u64), (1, 0), (1, 0)];
            let mut sut = HldVertexEdgePathQuery::<Affine>::new(&hld, &values);
            // When
            sut.set_vertex(1, (2, 3));
            sut.set_edge(1, 2, (5, 7));
            let result = sut.fold_vertex_edge_path(0, 2);
            // Then
            // f_v0(x)=x, f_e01(x)=x (単位元のまま), f_v1(x)=2x+3,
            // f_e12(x)=5x+7, f_v2(x)=x (単位元のまま)
            // g(f_e12(f_v1(x))) = 5(2x+3)+7 = 10x+22
            assert_eq!((10, 22), result);
        }
    }
}

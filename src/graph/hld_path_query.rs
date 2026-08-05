//! [`Hld`] が返す向き付きの区間 ([`PathDirection`]) を使って、パス上の値を
//! モノイドで畳み込むための補助モジュールである。
//!
//! `Hld` 自体は木の形 (親・深さ・チェーン分解) だけを保持する非ジェネリックな
//! 型であり、値の集約方法 (モノイド) には一切関知しない。これにより、1つの
//! `Hld` を、頂点の和・辺の最小値など、複数の異なるモノイドで同時に使い
//! 回せる (分解をモノイドの数だけ再計算せずに済む)。
//!
//! [`HldPathQuery`] は、非可換なモノイド (行列積・アフィン関数の合成など、
//! 経路の向きによって結果が変わる演算) によるパスクエリで必要になる、
//! 「番号の昇順」と「番号の降順」の2本のセグメント木を、`&Hld` を借用した
//! うえで保持する。可換なモノイド (総和など) しか使わない場合は、この型を
//! 経由せず、`Hld::vertex_path_ranges`/`edge_path_ranges` とセグメント木を
//! 直接組み合わせれば、木1本分のメモリと更新コストで済む。

use super::super::algebra::monoid::Monoid;
use super::super::ds::segment_tree::segment_tree_dense::SegmentTreeDense;
use super::hld::{Hld, PathDirection};

/// [`Hld`] を借用し、非可換なモノイド `M` によるパスクエリを処理する。
pub struct HldPathQuery<'a, M: Monoid>
where
    M::S: Clone,
{
    hld: &'a Hld,
    /// 番号の昇順に対応するセグメント木。
    forward: SegmentTreeDense<M>,
    /// 番号を反転させた (降順に対応する) セグメント木。
    reversed: SegmentTreeDense<M>,
}

impl<'a, M: Monoid> HldPathQuery<'a, M>
where
    M::S: Clone,
{
    /// `hld` を借用し、各頂点の初期値から `HldPathQuery` を構築する。
    ///
    /// # Args
    /// - `hld`: 対象の木の HLD。
    /// - `values`: `values[v]` が頂点 `v` の初期値。長さは `hld` の頂点数と
    ///   一致していなければならない。
    ///
    /// # Panics
    /// - `values.len()` が `hld` の頂点数と一致しない場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(V)
    ///   - V は頂点数である。
    pub fn new(hld: &'a Hld, values: &[M::S]) -> Self {
        let n = values.len();
        let mut forward = SegmentTreeDense::<M>::new(n);
        let mut reversed = SegmentTreeDense::<M>::new(n);
        for (v, value) in values.iter().enumerate() {
            let id = hld.vertex_id(v);
            forward.set(id, value.clone());
            reversed.set(n - 1 - id, value.clone());
        }
        forward.build();
        reversed.build();
        Self {
            hld,
            forward,
            reversed,
        }
    }

    /// 頂点 `v` の値を `x` に変更する。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    /// - `x`: 新しい値。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_vertex(&mut self, v: usize, x: M::S) {
        let n = self.forward.len();
        let id = self.hld.vertex_id(v);
        self.forward.update(id, x.clone());
        self.reversed.update(n - 1 - id, x);
    }

    /// 辺 `{u, v}` の値を `x` に変更する。辺は、深い側 (子側) の頂点の値
    /// として保持される。
    ///
    /// # Args
    /// - `u`/`v`: 辺の両端点。
    /// - `x`: 新しい値。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V)
    ///   - V は頂点数である。
    pub fn set_edge(&mut self, u: usize, v: usize, x: M::S) {
        let child = if self.hld.depth(u) > self.hld.depth(v) {
            u
        } else {
            v
        };
        self.set_vertex(child, x);
    }

    /// 頂点 `v` の現在の値を返す。
    ///
    /// # Args
    /// - `v`: 対象の頂点。
    ///
    /// # Returns
    /// `M::S`: 頂点 `v` の現在の値。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_vertex(&self, v: usize) -> M::S {
        self.forward.get(self.hld.vertex_id(v))
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての頂点の値を、`u` から
    /// `v` へ向かう順に畳み込む。
    ///
    /// # Args
    /// - `u`: パスの起点。
    /// - `v`: パスの終点。
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn fold_vertex_path(&self, u: usize, v: usize) -> M::S {
        self.fold_ranges(self.hld.vertex_path_ranges(u, v))
    }

    /// 頂点 `u` から `v` へのパス上にある、すべての辺の値を、`u` から `v`
    /// へ向かう順に畳み込む。
    ///
    /// # Args
    /// - `u`: パスの起点。
    /// - `v`: パスの終点。
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果。
    ///
    /// # Complexity
    /// - 時間計算量: O(log V) (ならし)
    ///   - V は頂点数である。
    pub fn fold_edge_path(&self, u: usize, v: usize) -> M::S {
        self.fold_ranges(self.hld.edge_path_ranges(u, v))
    }

    /// 向き付きの区間の列を、その順序を保ったまま畳み込む。
    ///
    /// # Args
    /// - `ranges`: [`Hld::vertex_path_ranges`]/[`Hld::edge_path_ranges`] が
    ///   返す、向き付きの半開区間の列。
    ///
    /// # Returns
    /// `M::S`: 畳み込んだ結果。
    fn fold_ranges(&self, ranges: Vec<(usize, usize, PathDirection)>) -> M::S {
        let n = self.forward.len();
        ranges.into_iter().fold(M::id(), |acc, (l, r, dir)| {
            let value = match dir {
                PathDirection::Forward => self.forward.fold(l, r),
                PathDirection::Reversed => self.reversed.fold(n - r, n - l),
            };
            M::op(&acc, &value)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::graph::Graph;
    use super::super::super::algebra::monoid::AffineMonoid;
    use super::*;

    type Affine = AffineMonoid<998244353>;

    // new/get_vertex のテスト: 構築直後の状態を検証する。
    mod new {
        use super::*;

        /// Scenario: 構築直後は、各頂点の値がそのまま取得できる。
        /// - Given: 3頂点の単純パス (0-1-2) があり、それぞれ異なる
        ///   一次関数 f(x)=2x+1, f(x)=3x+0, f(x)=1x+5 を初期値として持つ。
        /// - When: `HldPathQuery` を構築する。
        /// - Then: 各頂点の値が、初期値のまま `get_vertex` で取得できる。
        #[test]
        fn returns_initial_values() {
            // Given
            let mut g = Graph::new(3);
            g.add_undirected_edge(0, 1, ());
            g.add_undirected_edge(1, 2, ());
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            // When
            let sut = HldPathQuery::<Affine>::new(&hld, &values);
            // Then
            assert_eq!((2, 1), sut.get_vertex(0));
            assert_eq!((3, 0), sut.get_vertex(1));
            assert_eq!((1, 5), sut.get_vertex(2));
        }
    }

    // fold_vertex_path のテスト: 戻り値そのもの (畳み込み結果) を検証する。
    mod fold_vertex_path {
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
            let mut g = Graph::new(3);
            g.add_undirected_edge(0, 1, ());
            g.add_undirected_edge(1, 2, ());
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let sut = HldPathQuery::<Affine>::new(&hld, &values);
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
            let mut g = Graph::new(3);
            g.add_undirected_edge(0, 1, ());
            g.add_undirected_edge(1, 2, ());
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let mut sut = HldPathQuery::<Affine>::new(&hld, &values);
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
            let mut g = Graph::new(3);
            g.add_undirected_edge(0, 1, ());
            g.add_undirected_edge(1, 2, ());
            let hld = g.try_hld(0).unwrap();
            let values = vec![(2_u64, 1_u64), (3, 0), (1, 5)];
            let sut = HldPathQuery::<Affine>::new(&hld, &values);
            // When
            let result = sut.fold_vertex_path(1, 1);
            // Then
            assert_eq!((3, 0), result);
        }
    }
}

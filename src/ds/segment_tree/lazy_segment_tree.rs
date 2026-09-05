//! 遅延評価セグメント木を提供するモジュールである。
//!
//! 区間更新と区間クエリを $O(\log n)$ で処理できる遅延伝播
//! セグメント木 (`SegmentTreeLazyDense`) を実装する。モノイド
//! 上の値を管理し、準同型写像 (`Hom` トレイト) を通じて区間に
//! 対する作用を遅延的に適用する。

use super::super::super::algebra::monoid::{self, GeneratedMonoid};

/// モノイドの値に対する準同型写像を表すトレイトである。
///
/// 遅延セグメント木において、区間に対する作用 (更新操作) を
/// 抽象化する。作用の適用 (`f`) と作用同士の合成
/// (`composition`) を定義する必要がある。
///
/// # Examples
/// ```
/// use anmitsu::ds::segment_tree::lazy_segment_tree;
///
/// /// 加算による作用。`f(x) = x + self.0` を表す。
/// #[derive(Clone)]
/// struct AddEffect(i64);
///
/// impl lazy_segment_tree::Hom<i64> for AddEffect {
///     fn f(&self, x: &i64) -> i64 {
///         x + self.0
///     }
///     fn composition(&self, other: &Self) -> Self {
///         AddEffect(self.0 + other.0)
///     }
/// }
/// ```
pub trait Hom<S> {
    /// 値 `x` に作用を適用した結果を返す。
    ///
    /// # Args
    /// - `x` - 作用を適用する対象の値への参照
    ///
    /// # Returns
    /// 作用適用後の値を返す。
    fn f(&self, x: &S) -> S;

    /// `self` を先に適用し `other` を後から適用する合成を返す。
    ///
    /// すなわち、`self.composition(other).f(x)` と
    /// `other.f(self.f(x))` が等価である必要がある。
    ///
    /// # Args
    /// - `other` - `self` の後に適用する作用
    ///
    /// # Returns
    /// 合成された作用を返す。
    fn composition(&self, other: &Self) -> Self;
}

/// 遅延評価つき密セグメント木である。
///
/// 区間更新と区間クエリを $O(\log n)$ で処理する。内部的には
/// 完全二分木を 1-indexed 配列で表現し、各ノードに遅延作用を
/// 保持する。モノイド `M` が管理する値の型と、作用 `F` が
/// `Hom` トレイトを実装している必要がある。
///
/// # Examples
/// ```
/// use anmitsu::{
///     algebra::monoid,
///     ds::segment_tree::lazy_segment_tree,
/// };
///
/// #[derive(Clone)]
/// struct AddEffect(i64);
/// impl lazy_segment_tree::Hom<i64> for AddEffect {
///     fn f(&self, x: &i64) -> i64 { x + self.0 }
///     fn composition(&self, other: &Self) -> Self {
///         AddEffect(self.0 + other.0)
///     }
/// }
///
/// let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
///     monoid::AddMonoid,
///     AddEffect,
/// >::new(5);
/// // 位置 2 の値のみに 10 を加算する。
/// seg.effect(2, 3, AddEffect(10));
/// assert_eq!(10, seg.fold(0, 5));
/// ```
pub struct SegmentTreeLazyDense<M, F>
where
    M: monoid::Monoid,
{
    /// 内部配列のサイズ。要素数を 2 の冪に切り上げた値である。
    size: usize,
    /// 根から葉までの深さ。`size` の底 2 の対数に等しい。
    log: u32,
    /// 完全二分木を 1-indexed 配列で表現したデータ。長さは
    /// `2 * size` であり、添字 1 が根、添字 `size` 以降が
    /// 葉に対応する。添字 0 は未使用である。
    data: Vec<M::S>,
    /// 各ノードに保持される遅延作用。`None` は作用が未適用で
    /// あることを表す。1-indexed で長さは `2 * size` である。
    lazy: Vec<Option<F>>,
}

impl<M, F> SegmentTreeLazyDense<M, F>
where
    M: monoid::Monoid,
    M::S: Clone,
    F: Hom<M::S> + Clone,
{
    /// 指定されたサイズの遅延セグメント木を生成する。
    ///
    /// すべての葉をモノイドの単位元で初期化する。内部サイズは
    /// `size` を 2 の冪に切り上げた値となる。
    ///
    /// # Args
    /// - `size` - 管理する要素数
    ///
    /// # Returns
    /// 全要素が単位元で初期化された
    /// `SegmentTreeLazyDense` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(n)$
    /// - 空間計算量: $O(n)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::new(5);
    /// // 内部サイズは 2 の冪に切り上げた 8 になる。
    /// assert_eq!(8, seg.len());
    /// ```
    pub fn new(size: usize) -> Self {
        // 完全二分木を実現するため、要素数を 2 の冪に切り上げる。
        let size = size.next_power_of_two();
        let log = size.trailing_zeros();

        // すべてのノードを単位元で初期化する。
        // 単位元同士の演算は単位元を返すため、内部ノードの
        // 構築は不要である。
        SegmentTreeLazyDense {
            size,
            log,
            data: vec![M::id(); 2 * size],
            lazy: vec![None; 2 * size],
        }
    }

    /// ベクタから遅延セグメント木を構築する。
    ///
    /// 各要素を葉に配置し、内部ノードをボトムアップで構築する。
    /// ベクタの長さが 2 の冪でない場合、不足分は単位元で
    /// 埋められる。
    ///
    /// # Args
    /// - `v` - 初期値のベクタ
    ///
    /// # Returns
    /// ベクタの内容で初期化された
    /// `SegmentTreeLazyDense` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(n)$
    /// - 空間計算量: $O(n)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::from_vec(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(8, seg.len());
    /// ```
    pub fn from_vec(v: Vec<M::S>) -> Self {
        let n = v.len();

        // 完全二分木を実現するため、要素数を 2 の冪に切り上げる。
        let size = n.next_power_of_two();
        let log = size.trailing_zeros();

        // すべてのノードを単位元で初期化する。
        let mut data = vec![M::id(); 2 * size];

        // ベクタの各要素を対応する葉ノードに配置する。
        // 1-indexed のため、葉ノードの添字は `size` から始まる。
        for i in 0..n {
            data[size + i] = v[i].clone();
        }

        // 内部ノードの値を子ノードの演算結果で構築する。
        // 葉から根へ向かってボトムアップに計算する。
        for i in (1..size).rev() {
            data[i] = M::op(&data[2 * i], &data[2 * i + 1]);
        }

        SegmentTreeLazyDense {
            size,
            log,
            data,
            lazy: vec![None; 2 * size],
        }
    }

    /// 基底値のベクタから遅延セグメント木を構築する。
    ///
    /// `GeneratedMonoid` の `singleton` を用いて各基底値を
    /// モノイド値に変換し、木を構築する。
    ///
    /// # Args
    /// - `v` - 基底値のベクタ
    ///
    /// # Returns
    /// 基底値から構築された `SegmentTreeLazyDense` を返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(n)$
    /// - 空間計算量: $O(n)$
    ///
    /// # Examples
    /// ```
    /// use anmitsu::ds::segment_tree::bit_segment_tree;
    ///
    /// let mut seg = bit_segment_tree::BitSegTree::from_values(
    ///     vec![true, false, true],
    /// );
    /// let data = seg.fold(0, 3);
    /// assert_eq!(2, data.ones);
    /// ```
    pub fn from_values(v: Vec<M::V>) -> Self
    where
        M: GeneratedMonoid,
    {
        Self::from_vec(v.into_iter().map(M::singleton).collect())
    }

    /// 管理する要素数 (2 の冪に切り上げた内部サイズ) を返す。
    ///
    /// # Returns
    /// 内部サイズを返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(1)$
    /// - 空間計算量: $O(1)$
    pub fn len(&self) -> usize {
        self.size
    }

    /// ノード `k` のデータに遅延作用を反映した値を返す。
    ///
    /// 遅延作用が存在すれば適用した結果を、存在しなければ
    /// データをそのまま返す。遅延作用自体は変更しない。
    ///
    /// # Args
    /// - `k` - 評価対象のノード添字
    ///
    /// # Returns
    /// 遅延作用を反映したノードの値を返す。
    fn eval(&self, k: usize) -> M::S {
        match self.lazy[k].as_ref() {
            Some(ef) => ef.f(&self.data[k]),
            None => self.data[k].clone(),
        }
    }

    /// ノード `idx` の遅延作用を子ノードへ伝播し、
    /// 自身のデータを更新する。
    ///
    /// 遅延作用が存在する場合、左右の子に合成して転送し、
    /// 自身のデータに適用した後、遅延作用を `None` に
    /// リセットする。
    ///
    /// # Args
    /// - `idx` - 伝播対象のノード添字
    fn propagate(&mut self, idx: usize) {
        // 遅延作用を取り出す。存在しなければ何もしない。
        if let Some(ef) = self.lazy[idx].take() {
            // 内部ノードであれば、遅延作用を左右の子へ伝播する。
            if idx < self.size {
                // 左の子への伝播。既存の遅延作用と合成する。
                self.lazy[2 * idx] = match self.lazy[2 * idx].take() {
                    Some(old) => Some(old.composition(&ef)),
                    None => Some(ef.clone()),
                };

                // 右の子への伝播。
                self.lazy[2 * idx + 1] = match self.lazy[2 * idx + 1].take() {
                    Some(old) => Some(old.composition(&ef)),
                    None => Some(ef.clone()),
                };
            }

            // 自身のデータに遅延作用を適用する。
            self.data[idx] = ef.f(&self.data[idx]);
        }
    }

    /// ノード `k` の祖先すべての遅延作用を根から葉の方向に
    /// 順次伝播する。
    ///
    /// `effect` や `fold` の前に呼び出すことで、対象区間の
    /// 祖先ノードのデータを最新の状態にする。
    ///
    /// # Args
    /// - `k` - 葉ノードの添字
    fn push_ancestors(&mut self, k: usize) {
        // 根 (深さ log) から葉の親 (深さ 1) まで順に伝播する。
        for i in (1..=self.log).rev() {
            self.propagate(k >> i);
        }
    }

    /// ノード `k` の祖先すべてのデータを葉から根の方向に
    /// 再計算する。
    ///
    /// 子ノードの遅延作用を反映した値をもとに親ノードの
    /// データを更新する。
    ///
    /// # Args
    /// - `k` - 起点となるノードの添字
    fn update_ancestors(&mut self, mut k: usize) {
        // 親ノードへ向かってボトムアップに再計算する。
        k >>= 1;
        while k >= 1 {
            self.data[k] = M::op(&self.eval(2 * k), &self.eval(2 * k + 1));
            k >>= 1;
        }
    }

    /// 区間 `[l, r)` に作用 `effect` を適用する。
    ///
    /// 対象区間をカバーするノードの遅延作用に新しい作用を
    /// 合成し、祖先ノードのデータを再計算する。
    ///
    /// # Args
    /// - `l` - 区間の左端 (0-indexed, inclusive)
    /// - `r` - 区間の右端 (0-indexed, exclusive)
    /// - `effect` - 適用する作用
    ///
    /// # Complexity
    /// - 時間計算量: $O(\log n)$
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::from_vec(vec![1, 2, 3, 4, 5]);
    /// // 位置 2 の値に 10 を加算する。
    /// seg.effect(2, 3, AddEffect(10));
    /// assert_eq!(25, seg.fold(0, 5));
    /// ```
    pub fn effect(&mut self, mut l: usize, mut r: usize, effect: F) {
        // 論理的な添字を 1-indexed の葉ノード添字に変換する。
        l += self.size;
        r += self.size;

        // 対象区間の祖先ノードの遅延作用を上から順に伝播する。
        self.push_ancestors(l);
        self.push_ancestors(r - 1);

        // 再計算のために変換後の端点を保持する。
        let l0 = l;
        let r0 = r;

        // 対象区間をカバーするノードに作用を合成する。
        // 左端と右端から中心へ向かって走査する。
        while l < r {
            // 左端が右の子であれば、そのノードは区間に
            // 完全に含まれる。
            if l & 1 == 1 {
                self.lazy[l] = match self.lazy[l].take() {
                    Some(old) => Some(old.composition(&effect)),
                    None => Some(effect.clone()),
                };
                l += 1;
            }
            // 右端が右の子であれば、その直前のノードが区間に
            // 完全に含まれる。
            if r & 1 == 1 {
                r -= 1;
                self.lazy[r] = match self.lazy[r].take() {
                    Some(old) => Some(old.composition(&effect)),
                    None => Some(effect.clone()),
                };
            }

            // 親ノードの階層へ移動する。
            l >>= 1;
            r >>= 1;
        }

        // 対象区間の祖先ノードのデータを下から順に再計算する。
        self.update_ancestors(l0);
        self.update_ancestors(r0 - 1);
    }

    /// 区間 `[l, r)` のモノイド積 (畳み込み) を返す。
    ///
    /// 対象区間の祖先ノードの遅延作用を伝播した上で、
    /// 区間をカバーするノードの値を集約する。
    ///
    /// # Args
    /// - `l` - 区間の左端 (0-indexed, inclusive)
    /// - `r` - 区間の右端 (0-indexed, exclusive)
    ///
    /// # Returns
    /// 区間 `[l, r)` のモノイド積を返す。
    ///
    /// # Complexity
    /// - 時間計算量: $O(\log n)$
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::from_vec(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(15, seg.fold(0, 5));
    /// assert_eq!(9, seg.fold(1, 4));
    /// ```
    pub fn fold(&mut self, mut l: usize, mut r: usize) -> M::S {
        // 論理的な添字を 1-indexed の葉ノード添字に変換する。
        l += self.size;
        r += self.size;

        // 対象区間の祖先ノードの遅延作用を上から順に伝播する。
        self.push_ancestors(l);
        self.push_ancestors(r - 1);

        // 左端からの集約値と右端からの集約値を別々に保持する。
        // 非可換なモノイドに対応するため、左からと右からの
        // 演算順序を正しく保つ必要がある。
        let mut sum_l = M::id();
        let mut sum_r = M::id();

        while l < r {
            // 左端が右の子であれば、そのノードの値を
            // 左集約に追加する。
            if l & 1 == 1 {
                sum_l = M::op(&sum_l, &self.eval(l));
                l += 1;
            }
            // 右端が右の子であれば、その直前のノードの値を
            // 右集約に追加する。
            if r & 1 == 1 {
                r -= 1;
                sum_r = M::op(&self.eval(r), &sum_r);
            }

            // 親ノードの階層へ移動する。
            l >>= 1;
            r >>= 1;
        }

        // 左集約と右集約を結合して最終結果とする。
        M::op(&sum_l, &sum_r)
    }

    /// 区間 `[l, r)` の `fold` 結果に対して述語 `f` が
    /// `true` を返すような最小の `l` を返す。
    ///
    /// # Args
    /// - `r` - 区間の右端 (0-indexed, exclusive)
    /// - `f` - 判定に用いる述語。単位元に対して `true` を
    ///   返す必要がある。
    ///
    /// # Returns
    /// `f(fold(l, r))` が `true` となる最小の `l` を返す。
    /// 全体が `true` の場合は `0` を返す。
    ///
    /// # Panics
    /// `f(&M::id())` が `false` の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: $O(\log n)$
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::from_vec(vec![1, 2, 3, 4, 5]);
    /// // [1, 4) = 2+3+4 = 9 < 10, [0, 4) = 1+2+3+4 = 10
    /// let l = seg.min_left(4, |&sum| sum < 10);
    /// assert_eq!(1, l);
    /// ```
    pub fn min_left<L>(&mut self, r: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        // 述語は単位元に対して真でなければならない。
        assert!(f(&M::id()));
        assert!(r <= self.len());

        // 空区間の場合は 0 を返す。
        if r == 0 {
            return 0;
        }

        // 1-indexed の葉ノード添字に変換する。
        let mut r = r + self.size;

        // 右端の祖先ノードの遅延作用を伝播する。
        self.push_ancestors(r - 1);

        // 右端からの集約値を保持する。
        let mut sm = M::id();

        loop {
            r -= 1;

            // 右の子 (奇数添字) であれば親へ上がる。
            // 左の子に到達するまで繰り返す。
            while r > 1 && r & 1 == 1 {
                r >>= 1;
            }

            // 現在のノードを含めると述語が偽になる場合、
            // 境界はこのノードの内部にある。
            let t = M::op(&self.eval(r), &sm);
            if !f(&t) {
                // 葉に到達するまで二分探索で絞り込む。
                while r < self.size {
                    // 遅延作用を子へ伝播してから右の子へ降りる。
                    self.propagate(r);
                    r = 2 * r + 1;

                    // 右の子を含めても述語が真なら、
                    // 境界は左の子側にある。
                    let t = M::op(&self.eval(r), &sm);
                    if f(&t) {
                        sm = t;
                        r -= 1;
                    }
                }
                // 1-indexed の葉添字を論理添字に変換して返す。
                return r + 1 - self.size;
            }

            // 述語がまだ真であれば、集約値を更新して
            // 次のノードへ進む。
            sm = t;

            // r が 2 の冪であれば先頭に到達している。
            if r & r.wrapping_neg() == r {
                break;
            }
        }

        0
    }

    /// 区間 `[l, r)` の `fold` 結果に対して述語 `f` が
    /// `true` を返すような最大の `r` を返す。
    ///
    /// # Args
    /// - `l` - 区間の左端 (0-indexed, inclusive)
    /// - `f` - 判定に用いる述語。単位元に対して `true` を
    ///   返す必要がある。
    ///
    /// # Returns
    /// `f(fold(l, r))` が `true` となる最大の `r` を返す。
    /// 全体が `true` の場合は `self.len()` を返す。
    ///
    /// # Panics
    /// `f(&M::id())` が `false` の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: $O(n)$ (愚直実装)
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```
    /// # use anmitsu::{
    /// #     algebra::monoid,
    /// #     ds::segment_tree::lazy_segment_tree,
    /// # };
    /// # #[derive(Clone)]
    /// # struct AddEffect(i64);
    /// # impl lazy_segment_tree::Hom<i64> for AddEffect {
    /// #     fn f(&self, x: &i64) -> i64 { x + self.0 }
    /// #     fn composition(&self, other: &Self) -> Self {
    /// #         AddEffect(self.0 + other.0)
    /// #     }
    /// # }
    /// let mut seg = lazy_segment_tree::SegmentTreeLazyDense::<
    ///     monoid::AddMonoid,
    ///     AddEffect,
    /// >::from_vec(vec![1, 2, 3, 4, 5]);
    /// // [1, 4) = 2+3+4 = 9 < 10, [1, 5) = 2+3+4+5 = 14
    /// let r = seg.max_right(1, |&sum| sum < 10);
    /// assert_eq!(4, r);
    /// ```
    pub fn max_right<L>(&mut self, l: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        // 述語は単位元に対して真でなければならない。
        assert!(f(&M::id()));

        // 全体が条件を満たす場合は末尾を返す。
        if l == self.len() || f(&self.fold(l, self.len())) {
            return self.len();
        }

        // 末尾から左端へ向かって、条件を満たす最大の `r` を
        // 線形探索する。
        (l..self.len())
            .rev()
            .find(|&i| f(&self.fold(l, i)))
            .unwrap()
    }
}

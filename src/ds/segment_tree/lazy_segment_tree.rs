//! 遅延評価セグメント木を提供するモジュールである。
//!
//! 区間更新と区間クエリを $O(\log n)$ で処理できる遅延伝播
//! セグメント木 (`SegmentTreeLazyDense`) を実装する。モノイド
//! 上の値を管理し、準同型写像 (`Hom` トレイト) を通じて区間に
//! 対する作用を遅延的に適用する。

use super::super::super::algebra::monoid;

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
/// 完全二分木を配列で表現し、各ノードに遅延作用を保持する。
/// モノイド `M` が管理する値の型と、作用 `F` が `Hom` トレイト
/// を実装している必要がある。
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
    /// 完全二分木を配列で表現したデータ。長さは
    /// `2 * size - 1` であり、添字 0 が根に対応する。
    data: Vec<M::S>,
    /// 各ノードに保持される遅延作用。`None` は作用が未適用で
    /// あることを表す。
    lazy: Vec<Option<F>>,
}

impl<M, F> SegmentTreeLazyDense<M, F>
where
    M: monoid::Monoid,
    M::S: Clone,
    F: Hom<M::S> + Clone,
    Option<F>: Clone,
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

        // すべてのノードを単位元で初期化する。
        let mut data = vec![M::id(); 2 * size - 1];

        // 内部ノードの値を子ノードから構築する。
        // 初期状態ではすべて単位元であるため実質的に変化しないが、
        // 一般の初期化と同じ構築手順を踏んでおく。
        for i in (0..size - 1).rev() {
            data[i] = M::op(&data[2 * i + 1], &data[2 * i + 2]);
        }

        // 遅延配列はすべて None (作用なし) で初期化する。
        SegmentTreeLazyDense {
            size,
            data,
            lazy: vec![None; 2 * size - 1],
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

        // すべてのノードを単位元で初期化する。
        let mut data = vec![M::id(); 2 * size - 1];

        // ベクタの各要素を対応する葉ノードに配置する。
        // 葉ノードの添字は `size - 1` から始まる。
        for i in 0..n {
            data[size - 1 + i] = v[i].clone();
        }

        // 内部ノードの値を子ノードの演算結果で構築する。
        // 葉から根へ向かってボトムアップに計算する。
        for i in (0..size - 1).rev() {
            data[i] = M::op(&data[2 * i + 1], &data[2 * i + 2]);
        }

        // 遅延配列はすべて None (作用なし) で初期化する。
        SegmentTreeLazyDense {
            size,
            data,
            lazy: vec![None; 2 * size - 1],
        }
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

    /// ノード `idx` の遅延作用を子ノードへ伝播し、
    /// 自身のデータを更新する。
    ///
    /// 内部ノードの場合、遅延作用を左右の子に合成して転送する。
    /// その後、自身の遅延作用をデータに適用し、遅延作用を
    /// `None` にリセットする。
    ///
    /// # Args
    /// - `idx` - 伝播対象のノード添字
    fn propagate(&mut self, idx: usize) {
        // 内部ノードであれば、遅延作用を左右の子へ伝播する。
        if idx < self.size - 1 {
            // 左の子 (添字 `2 * idx + 1`) への伝播。
            // 子がすでに遅延作用を持つ場合は合成し、
            // 持たない場合は親の作用をそのまま引き継ぐ。
            self.lazy[2 * idx + 1] = match (&self.lazy[2 * idx + 1], &self.lazy[idx]) {
                (Some(ef1), Some(ef2)) => Some(ef1.composition(ef2)),
                (Some(ef1), None) => Some(ef1.clone()),
                (None, Some(ef2)) => Some(ef2.clone()),
                (None, None) => None,
            };

            // 右の子 (添字 `2 * idx + 2`) への伝播。
            self.lazy[2 * idx + 2] = match (&self.lazy[2 * idx + 2], &self.lazy[idx]) {
                (Some(ef1), Some(ef2)) => Some(ef1.composition(ef2)),
                (Some(ef1), None) => Some(ef1.clone()),
                (None, Some(ef2)) => Some(ef2.clone()),
                (None, None) => None,
            };
        }

        // 自身の遅延作用をデータに適用する。
        // 適用後、遅延作用を None にリセットして
        // 二重適用を防ぐ。
        if let Some(effect) = self.lazy[idx].clone() {
            self.data[idx] = effect.f(&self.data[idx]);
            self.lazy[idx] = None;
        }
    }

    /// 区間 `[l, r]` の両端から根までのパス上にある
    /// ノード添字を、根から葉の方向に整列して返す。
    ///
    /// `effect` や `fold` の前に、対象区間の祖先ノードの
    /// 遅延作用を上から順に伝播するために使用する。
    ///
    /// # Args
    /// - `l` - 区間の左端 (内部添字)
    /// - `r` - 区間の右端 (内部添字)
    ///
    /// # Returns
    /// 根から葉への順に並べたノード添字のベクタを返す。
    fn get_index(&self, mut l: usize, mut r: usize) -> Vec<usize> {
        // 両端から根までのパスを収集する。
        let mut res = vec![];
        while l > 0 {
            l = (l - 1) / 2;
            res.push(l);
        }
        while r > 0 {
            r = (r - 1) / 2;
            res.push(r);
        }

        // 根から葉の方向に伝播する必要があるため、
        // 逆順にする。
        res.reverse();
        res
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
    /// - 空間計算量: $O(\log n)$
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
        // 論理的な添字を内部配列の添字に変換する。
        // 葉ノードは添字 `size - 1` から始まる。
        l += self.size - 1;
        r += self.size - 1;

        // 対象区間の祖先ノードの遅延作用を
        // 上から順に伝播する。
        for idx in self.get_index(l, r - 1) {
            self.propagate(idx);
        }

        // 対象区間をカバーするノードに作用を合成する。
        // セグメント木の区間分割に従い、左端と右端から
        // 中心へ向かって走査する。
        {
            let mut l = l;
            let mut r = r;
            while l < r {
                // 左端が偶数添字 (左の子) であれば、
                // そのノードは区間に完全に含まれる。
                if l % 2 == 0 {
                    if let Some(old) = self.lazy[l].clone() {
                        self.lazy[l] = Some(old.composition(&effect));
                    } else {
                        self.lazy[l] = Some(effect.clone());
                    }
                }
                // 右端が偶数添字 (左の子) であれば、
                // その直前のノードが区間に完全に含まれる。
                if r % 2 == 0 {
                    if let Some(old) = self.lazy[r - 1].clone() {
                        self.lazy[r - 1] = Some(old.composition(&effect));
                    } else {
                        self.lazy[r - 1] = Some(effect.clone());
                    }
                }

                // 親ノードの階層へ移動する。
                l = l / 2;
                r = (r - 1) / 2;
            }
        }

        // 対象区間の祖先ノードのデータを下から順に再計算する。
        // 子ノードに遅延作用がある場合は、それを考慮して
        // 正しいデータ値を求める。
        for idx in self.get_index(l, r - 1).into_iter().rev() {
            self.data[idx] = match (
                self.lazy[2 * idx + 1].clone(),
                self.lazy[2 * idx + 2].clone(),
            ) {
                (Some(ef1), Some(ef2)) => M::op(
                    &ef1.f(&self.data[2 * idx + 1]),
                    &ef2.f(&self.data[2 * idx + 2]),
                ),
                (Some(ef1), None) => {
                    M::op(&ef1.f(&self.data[2 * idx + 1]), &self.data[2 * idx + 2])
                }
                (None, Some(ef2)) => {
                    M::op(&self.data[2 * idx + 1], &ef2.f(&self.data[2 * idx + 2]))
                }
                (None, None) => M::op(&self.data[2 * idx + 1], &self.data[2 * idx + 2]),
            };
        }
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
    /// - 空間計算量: $O(\log n)$
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
        // 論理的な添字を内部配列の添字に変換する。
        l += self.size - 1;
        r += self.size - 1;

        // 対象区間の祖先ノードの遅延作用を
        // 上から順に伝播する。
        for idx in self.get_index(l, r - 1) {
            self.propagate(idx);
        }

        // 左端からの集約値と右端からの集約値を別々に保持する。
        // 非可換なモノイドに対応するため、左からと右からの
        // 演算順序を正しく保つ必要がある。
        let mut sum_l = M::id();
        let mut sum_r = M::id();

        while l < r {
            // 左端が偶数添字であれば、そのノードの値を
            // 左集約に追加する。遅延作用がある場合は
            // 適用してから加える。
            if l % 2 == 0 {
                sum_l = if let Some(ef) = self.lazy[l].clone() {
                    M::op(&sum_l, &ef.f(&self.data[l]))
                } else {
                    M::op(&sum_l, &self.data[l])
                };
            }
            // 右端が偶数添字であれば、その直前のノードの値を
            // 右集約に追加する。
            if r % 2 == 0 {
                sum_r = if let Some(ef) = self.lazy[r - 1].clone() {
                    M::op(&ef.f(&self.data[r - 1]), &sum_r)
                } else {
                    M::op(&self.data[r - 1], &sum_r)
                };
            }

            // 親ノードの階層へ移動する。
            l = l / 2;
            r = (r - 1) / 2;
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
    /// - 空間計算量: $O(\log n)$
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
    pub fn min_left<L>(&mut self, mut r: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        /// ノード `k` がセグメント木のサイズ `len` に対して
        /// 有効な位置にあるかどうかを判定する補助関数。
        fn is_good_node(k: usize, len: usize) -> bool {
            if k >= len {
                true
            } else {
                let d = k.leading_zeros() - len.leading_zeros();
                len >> d != k || len >> d << d == len
            }
        }

        // 述語は単位元に対して真でなければならない。
        assert!(f(&M::id()));
        assert!(r <= self.len());

        // 全体が条件を満たす場合、または空区間の場合は
        // 0 を返す。
        if r == 0 || f(&self.fold(0, r)) {
            return 0;
        }

        // 右端からの集約値を保持する。
        let mut sum = M::id();

        // 内部添字に変換する。
        r += self.len();

        loop {
            r -= 1;

            // 有効なノード位置になるまで右の子へ降りる。
            while !is_good_node(r, self.len()) {
                r = r * 2 + 1;
            }

            // 可能な限り親ノードへ上がる。
            while r & 1 != 0 && is_good_node(r >> 1, self.len()) {
                r >>= 1;
                self.propagate(r - 1);
            }

            // 現在のノードを含めると述語が偽になる場合、
            // 境界はこのノードの内部にある。
            if !f(&if let Some(ef) = self.lazy[r - 1].clone() {
                M::op(&sum, &ef.f(&self.data[r - 1]))
            } else {
                M::op(&sum, &self.data[r - 1])
            }) {
                // 葉に到達するまで二分探索で絞り込む。
                while r < self.len() {
                    // 右の子へ降りて伝播する。
                    r = r * 2 + 1;
                    self.propagate(r - 1);

                    // 右の子を含めても述語が真なら、
                    // 境界は左の子側にある。
                    let t = if let Some(ef) = self.lazy[r - 1].clone() {
                        M::op(&sum, &ef.f(&self.data[r - 1]))
                    } else {
                        M::op(&sum, &self.data[r - 1])
                    };
                    if f(&t) {
                        sum = t;
                        r -= 1;
                        self.propagate(r - 1);
                    }
                }
                // 内部添字を論理添字に変換して返す。
                return r + 1 - self.len();
            }

            // 述語がまだ真であれば、集約値を更新して
            // 次のノードへ進む。
            sum = if let Some(ef) = self.lazy[r - 1].clone() {
                M::op(&sum, &ef.f(&self.data[r - 1]))
            } else {
                M::op(&sum, &self.data[r - 1])
            };
        }
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
    /// - 空間計算量: $O(\log n)$
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

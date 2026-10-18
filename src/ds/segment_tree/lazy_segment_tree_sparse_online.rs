//! 更新端点で区間を分割するオンライン版の疎な遅延セグメント木。
//!
//! 論理上の区間 `[0, len)` の各点は初めモノイドの単位元を持つ。
//! 一様な区間を葉とする AVL 木を使い、座標幅に依存する長い経路を保持しない。
//! 更新回数を `Q` とすると節点数は `O(Q)`。読み取りでは節点を生成しない。

use super::super::super::algebra::monoid;
use std::{cmp, ops::RangeBounds};

pub use super::lazy_segment_tree::RangeAction;

/// 子がないこと、または空の部分木を表す。
const NONE: u32 = u32::MAX;

/// 区間全体の集約値と、葉の一様な作用または内部節点の未伝播作用。
struct Node<S, F> {
    /// この部分木の区間全体に対する現在の集約値。
    value: S,
    /// 葉では初期値からの作用、内部節点では子へ未伝播の作用。
    lazy: Option<F>,
    /// 元の座標系における区間長。要素数を座標圧縮しない。
    len: usize,
    /// 左右の子。葉ではともに `NONE`。未使用節点では先頭が空きリスト。
    children: [u32; 2],
    /// 葉を 1 とする部分木の高さ。未使用節点では 0。
    height: u8,
}

/// 境界探索中の節点と、その節点が担当する半開区間。
struct Traversal {
    /// 節点配列内の位置。
    node: u32,
    /// 区間の左端。
    lower: usize,
    /// 区間の右端。
    upper: usize,
}

/// 論理範囲に比例する領域を確保しない遅延セグメント木。
///
/// 更新の端点で一様な区間を分割し、AVL 木の高さを維持する。
/// 一様な区間の一部は、その長さに初期値からの作用を適用して集約する。
/// 座標は `usize` のままで、確保できる節点数は `u32::MAX` 個まで。
/// 作用型に `Default` や `Copy` は要求しない。
///
/// # Examples
/// ```
/// use anmitsu::{algebra::monoid, ds::segment_tree::lazy_segment_tree_sparse_online};
///
/// #[derive(Clone)]
/// struct Add(i64);
/// impl lazy_segment_tree_sparse_online::RangeAction<i64> for Add {
///     fn apply(&self, value: &i64, len: usize) -> i64 {
///         value + self.0 * len as i64
///     }
///     fn composition(&self, other: &Self) -> Self {
///         Add(self.0 + other.0)
///     }
/// }
/// let mut seg = lazy_segment_tree_sparse_online::SegmentTreeLazySparseOnline::<
///     monoid::AddMonoid,
///     Add,
/// >::new(1_000_000_000);
/// seg.effect(10..20, Add(3));
/// assert_eq!(30, seg.fold(..));
/// assert_eq!(3, seg.get(15));
/// ```
pub struct SegmentTreeLazySparseOnline<M, F>
where
    M: monoid::Monoid,
{
    /// 論理上の区間長。
    len: usize,
    /// 現在の根。
    root: u32,
    /// 再利用できる節点の先頭。
    free: u32,
    /// 葉と内部節点を同じ配列で保持する。
    nodes: Vec<Node<M::S, F>>,
}

impl<M, F> SegmentTreeLazySparseOnline<M, F>
where
    M: monoid::Monoid,
    M::S: Clone,
    F: RangeAction<M::S>,
{
    /// 全点が単位元である論理上の区間を作成する。
    ///
    /// # Args
    /// - `len` - 論理上の区間 `[0, len)` の長さ。
    ///
    /// # Returns
    /// 根節点だけを持つ木を返す。
    ///
    /// # Complexity
    /// 時間・空間ともに `O(1)`。
    pub fn new(len: usize) -> Self {
        Self {
            len,
            root: 0,
            free: NONE,
            nodes: vec![Node {
                value: M::id(),
                lazy: None,
                len,
                children: [NONE; 2],
                height: 1,
            }],
        }
    }

    /// 論理上の区間長を返す。
    ///
    /// # Returns
    /// 構築時に指定した `len` を返す。
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// 論理上の区間が空であるかを返す。
    ///
    /// # Returns
    /// `len == 0` なら真を返す。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 指定した区間に作用を適用する。
    ///
    /// # Args
    /// - `range` - 更新する区間。`left..right`、`left..=right`、`..` などを指定できる。
    /// - `effect` - 現在の値へ後から適用する作用。
    ///
    /// # Panics
    /// 端点が範囲外か、順序が逆の場合にパニックする。
    /// 新しい節点が必要なとき、節点数が `u32::MAX` 個ならパニックする。
    ///
    /// # Complexity
    /// 時間 `O(log(K + 1))`、追加節点数は高々 4 個。空区間では `O(1)`。
    /// `K` は現在保持している区間数。
    pub fn effect(&mut self, range: impl RangeBounds<usize>, effect: F) {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left < right {
            self.root = self.effect_node(self.root, left, right, &effect);
        }
    }

    /// 指定した区間の集約値を返す。
    ///
    /// # Args
    /// - `range` - 集約する区間。`left..right`、`left..=right`、`..` などを指定できる。
    ///
    /// # Returns
    /// 区間の集約値。空区間では単位元を返す。
    ///
    /// # Panics
    /// 端点が範囲外か、順序が逆の場合にパニックする。
    ///
    /// # Complexity
    /// 時間・再帰スタックともに `O(log(K + 1))`。
    /// `K` は現在保持している区間数。
    pub fn fold(&self, range: impl RangeBounds<usize>) -> M::S {
        let (left, right) = super::range_bounds::normalize(range, self.len, "index");
        if left == right {
            return M::id();
        }
        self.fold_node(self.root, 0, left, right, None)
    }

    /// 点の現在値を返す。
    ///
    /// # Args
    /// - `index` - 取得する点の座標。
    ///
    /// # Returns
    /// 現在値。未更新なら単位元を返す。
    ///
    /// # Panics
    /// `index >= len` の場合にパニックする。
    pub fn get(&self, index: usize) -> M::S {
        assert!(index < self.len, "index out of bounds");
        self.fold(index..index + 1)
    }

    /// `[left, right)` の集約値を述語が受け入れる最大の `right` を返す。
    ///
    /// # Args
    /// - `left` - 探索を開始する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最大の右端。最後まで真なら `len` を返す。
    ///
    /// # Panics
    /// `left > len`、または単位元を述語が拒否するとパニックする。
    pub fn max_right<P>(&self, left: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(left <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.max_right_node(
            Traversal {
                node: self.root,
                lower: 0,
                upper: self.len,
            },
            left,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(self.len)
    }

    /// `[left, right)` の集約値を述語が受け入れる最小の `left` を返す。
    ///
    /// # Args
    /// - `right` - 探索を終了する位置。
    /// - `predicate` - 単位元を受け入れ、区間を広げると真から偽にのみ変わる述語。
    ///
    /// # Returns
    /// 条件を満たす最小の左端。最後まで真なら `0` を返す。
    ///
    /// # Panics
    /// `right > len`、または単位元を述語が拒否するとパニックする。
    pub fn min_left<P>(&self, right: usize, predicate: P) -> usize
    where
        P: Fn(&M::S) -> bool,
    {
        assert!(right <= self.len, "index out of bounds");
        assert!(predicate(&M::id()), "predicate must accept the identity");
        let mut sum = M::id();
        self.min_left_node(
            Traversal {
                node: self.root,
                lower: 0,
                upper: self.len,
            },
            right,
            None,
            &predicate,
            &mut sum,
        )
        .unwrap_or(0)
    }

    /// 空の部分木を 0 として高さを返す。
    #[inline]
    fn height(&self, node: u32) -> u8 {
        if node == NONE {
            0
        } else {
            self.nodes[node as usize].height
        }
    }

    /// 空き節点を再利用し、なければ配列の末尾に追加する。
    ///
    /// # Args
    /// - `entry` - 新しい葉または内部節点。
    ///
    /// # Returns
    /// 格納した節点の添字を返す。
    fn allocate(&mut self, entry: Node<M::S, F>) -> u32 {
        if self.free != NONE {
            let node = self.free;
            self.free = self.nodes[node as usize].children[0];
            self.nodes[node as usize] = entry;
            node
        } else {
            assert!(
                self.nodes.len() < NONE as usize,
                "too many sparse segment tree nodes"
            );
            let node = self.nodes.len() as u32;
            self.nodes.push(entry);
            node
        }
    }

    /// 分割で不要になった内部節点を空きリストへ戻す。
    fn release(&mut self, node: u32) {
        self.nodes[node as usize] = Node {
            value: M::id(),
            lazy: None,
            len: 0,
            children: [self.free, NONE],
            height: 0,
        };
        self.free = node;
    }

    /// 左右の集約値、区間長、高さから内部節点を更新する。
    #[inline]
    fn pull(&mut self, node: u32) {
        let [left, right] = self.nodes[node as usize].children;
        let value = M::op(
            &self.nodes[left as usize].value,
            &self.nodes[right as usize].value,
        );
        let len = self.nodes[left as usize].len + self.nodes[right as usize].len;
        let height = 1 + cmp::max(self.height(left), self.height(right));
        let entry = &mut self.nodes[node as usize];
        entry.value = value;
        entry.len = len;
        entry.height = height;
    }

    /// 部分木全体の集約値と、葉または子への作用を更新する。
    #[inline]
    fn apply_node(&mut self, node: u32, effect: &F) {
        let entry = &mut self.nodes[node as usize];
        entry.value = effect.apply(&entry.value, entry.len);
        let lazy = match entry.lazy.as_ref() {
            Some(old) => old.composition(effect),
            None => effect.clone(),
        };
        // 古い作用の破棄がパニックしても、新しい作用の所有権を木に残す。
        drop(entry.lazy.replace(lazy));
    }

    /// 内部節点の未伝播作用を、すでに存在する二つの子へ渡す。
    #[inline]
    fn push(&mut self, node: u32) {
        if self.nodes[node as usize].children[0] == NONE {
            return;
        }
        if let Some(effect) = self.nodes[node as usize].lazy.take() {
            let children = self.nodes[node as usize].children;
            self.apply_node(children[0], &effect);
            self.apply_node(children[1], &effect);
        }
    }

    /// 左右の部分木をまとめる内部節点を作る。
    fn branch(&mut self, left: u32, right: u32) -> u32 {
        let value = M::op(
            &self.nodes[left as usize].value,
            &self.nodes[right as usize].value,
        );
        let len = self.nodes[left as usize].len + self.nodes[right as usize].len;
        let height = 1 + cmp::max(self.height(left), self.height(right));
        self.allocate(Node {
            value,
            lazy: None,
            len,
            children: [left, right],
            height,
        })
    }

    /// 作用を伝播してから左回転し、新しい根を返す。
    fn rotate_left(&mut self, node: u32) -> u32 {
        self.push(node);
        let pivot = self.nodes[node as usize].children[1];
        self.push(pivot);
        self.nodes[node as usize].children[1] = self.nodes[pivot as usize].children[0];
        self.nodes[pivot as usize].children[0] = node;
        self.pull(node);
        self.pull(pivot);
        pivot
    }

    /// 作用を伝播してから右回転し、新しい根を返す。
    fn rotate_right(&mut self, node: u32) -> u32 {
        self.push(node);
        let pivot = self.nodes[node as usize].children[0];
        self.push(pivot);
        self.nodes[node as usize].children[0] = self.nodes[pivot as usize].children[1];
        self.nodes[pivot as usize].children[1] = node;
        self.pull(node);
        self.pull(pivot);
        pivot
    }

    /// 子の高さの差を高々 1 に戻す。
    fn balance(&mut self, node: u32) -> u32 {
        self.pull(node);
        let [left, right] = self.nodes[node as usize].children;
        if self.height(left) > self.height(right) + 1 {
            self.push(left);
            let [a, b] = self.nodes[left as usize].children;
            if self.height(a) < self.height(b) {
                self.nodes[node as usize].children[0] = self.rotate_left(left);
            }
            self.rotate_right(node)
        } else if self.height(right) > self.height(left) + 1 {
            self.push(right);
            let [a, b] = self.nodes[right as usize].children;
            if self.height(b) < self.height(a) {
                self.nodes[node as usize].children[1] = self.rotate_right(right);
            }
            self.rotate_left(node)
        } else {
            node
        }
    }

    /// 順序を保って二つの AVL 木を連結する。
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == NONE {
            return right;
        }
        if right == NONE {
            return left;
        }
        if self.height(left) > self.height(right) + 1 {
            self.push(left);
            let tail = self.nodes[left as usize].children[1];
            let joined = self.join(tail, right);
            self.nodes[left as usize].children[1] = joined;
            self.balance(left)
        } else if self.height(right) > self.height(left) + 1 {
            self.push(right);
            let head = self.nodes[right as usize].children[0];
            let joined = self.join(left, head);
            self.nodes[right as usize].children[0] = joined;
            self.balance(right)
        } else {
            self.branch(left, right)
        }
    }

    /// 更新端点を含む葉だけを分割し、帰りがけに高さを整える。
    ///
    /// # Args
    /// - `node` - 更新対象の根。
    /// - `left`、`right` - 部分木の先頭を 0 とする更新範囲。
    /// - `effect` - 対象区間へ後から適用する作用。
    ///
    /// # Returns
    /// 更新後の部分木の根を返す。
    fn effect_node(&mut self, node: u32, left: usize, right: usize, effect: &F) -> u32 {
        let len = self.nodes[node as usize].len;
        if left == 0 && right == len {
            self.apply_node(node, effect);
            return node;
        }
        let [mut a, mut b] = self.nodes[node as usize].children;
        if a == NONE {
            // 一様な葉だけを最大 3 区間へ分け、中央には元の節点を使う。
            let lazy = self.nodes[node as usize].lazy.clone();
            let prefix = if left == 0 {
                NONE
            } else {
                self.allocate(Node {
                    value: Self::uniform_value(lazy.as_ref(), left),
                    lazy: lazy.clone(),
                    len: left,
                    children: [NONE; 2],
                    height: 1,
                })
            };
            let suffix = if right == len {
                NONE
            } else {
                self.allocate(Node {
                    value: Self::uniform_value(lazy.as_ref(), len - right),
                    lazy: lazy.clone(),
                    len: len - right,
                    children: [NONE; 2],
                    height: 1,
                })
            };
            self.nodes[node as usize].len = right - left;
            self.nodes[node as usize].value = Self::uniform_value(lazy.as_ref(), right - left);
            self.apply_node(node, effect);
            let head = self.join(prefix, node);
            return self.join(head, suffix);
        }
        self.push(node);
        let middle = self.nodes[a as usize].len;
        if left < middle {
            a = self.effect_node(a, left, cmp::min(right, middle), effect);
        }
        if middle < right {
            b = self.effect_node(b, left.saturating_sub(middle), right - middle, effect);
        }
        if self.height(a).abs_diff(self.height(b)) <= 1 {
            // 高さが保たれる大半の更新では、連結や節点の再確保を省く。
            self.nodes[node as usize].children = [a, b];
            self.pull(node);
            node
        } else {
            self.release(node);
            self.join(a, b)
        }
    }

    /// 一様な区間に初期値からの作用を適用する。
    #[inline]
    fn uniform_value(effect: Option<&F>, len: usize) -> M::S {
        match effect {
            Some(effect) => effect.apply(&M::id(), len),
            None => M::id(),
        }
    }

    /// 節点の古い作用の後に、祖先からの新しい作用を合成する。
    #[inline]
    fn descend_effect(&self, node: u32, ancestor: Option<F>) -> Option<F> {
        match (self.nodes[node as usize].lazy.as_ref(), ancestor) {
            (Some(old), Some(new)) => Some(old.composition(&new)),
            (Some(old), None) => Some(old.clone()),
            (None, inherited) => inherited,
        }
    }

    /// 読み取り専用の祖先作用を反映した区間全体の値を返す。
    #[inline]
    fn inherited_value(&self, node: u32, ancestor: Option<&F>) -> M::S {
        let entry = &self.nodes[node as usize];
        match ancestor {
            Some(effect) => effect.apply(&entry.value, entry.len),
            None => entry.value.clone(),
        }
    }

    /// 一様な葉の一部を含め、節点を作らずに区間を集約する。
    fn fold_node(
        &self,
        node: u32,
        lower: usize,
        left: usize,
        right: usize,
        ancestor: Option<F>,
    ) -> M::S {
        let entry = &self.nodes[node as usize];
        let upper = lower + entry.len;
        if right <= lower || upper <= left {
            return M::id();
        }
        if left <= lower && upper <= right {
            return self.inherited_value(node, ancestor.as_ref());
        }
        let inherited = self.descend_effect(node, ancestor);
        let [a, b] = entry.children;
        if a == NONE {
            let len = cmp::min(upper, right) - cmp::max(lower, left);
            return Self::uniform_value(inherited.as_ref(), len);
        }
        let middle = lower + self.nodes[a as usize].len;
        if right <= middle {
            return self.fold_node(a, lower, left, right, inherited);
        }
        if middle <= left {
            return self.fold_node(b, middle, left, right, inherited);
        }
        let a = self.fold_node(a, lower, left, right, inherited.clone());
        let b = self.fold_node(b, middle, left, right, inherited);
        M::op(&a, &b)
    }

    /// 左から走査し、失敗する一様な葉の中だけを二分探索する。
    fn max_right_node<P>(
        &self,
        range: Traversal,
        left: usize,
        ancestor: Option<F>,
        predicate: &P,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Traversal { node, lower, upper } = range;
        if upper <= left {
            return None;
        }
        if left <= lower {
            let value = self.inherited_value(node, ancestor.as_ref());
            let next = M::op(sum, &value);
            if predicate(&next) {
                *sum = next;
                return None;
            }
        }
        let inherited = self.descend_effect(node, ancestor);
        let [a, b] = self.nodes[node as usize].children;
        if a == NONE {
            let start = cmp::max(lower, left);
            let len = upper - start;
            // 左端が葉の途中なら、その残り全体を受理するかを先に確認する。
            if lower < left {
                let value = Self::uniform_value(inherited.as_ref(), len);
                let next = M::op(sum, &value);
                if predicate(&next) {
                    *sum = next;
                    return None;
                }
            }
            let mut accepted = 0;
            let mut rejected = len;
            while rejected - accepted > 1 {
                let middle = accepted + (rejected - accepted) / 2;
                let value = Self::uniform_value(inherited.as_ref(), middle);
                if predicate(&M::op(sum, &value)) {
                    accepted = middle;
                } else {
                    rejected = middle;
                }
            }
            return Some(start + accepted);
        }
        let middle = lower + self.nodes[a as usize].len;
        self.max_right_node(
            Traversal {
                node: a,
                lower,
                upper: middle,
            },
            left,
            inherited.clone(),
            predicate,
            sum,
        )
        .or_else(|| {
            self.max_right_node(
                Traversal {
                    node: b,
                    lower: middle,
                    upper,
                },
                left,
                inherited,
                predicate,
                sum,
            )
        })
    }

    /// 右から走査し、失敗する一様な葉の中だけを二分探索する。
    fn min_left_node<P>(
        &self,
        range: Traversal,
        right: usize,
        ancestor: Option<F>,
        predicate: &P,
        sum: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Traversal { node, lower, upper } = range;
        if right <= lower {
            return None;
        }
        if upper <= right {
            let value = self.inherited_value(node, ancestor.as_ref());
            let next = M::op(&value, sum);
            if predicate(&next) {
                *sum = next;
                return None;
            }
        }
        let inherited = self.descend_effect(node, ancestor);
        let [a, b] = self.nodes[node as usize].children;
        if a == NONE {
            let end = cmp::min(upper, right);
            let len = end - lower;
            if right < upper {
                let value = Self::uniform_value(inherited.as_ref(), len);
                let next = M::op(&value, sum);
                if predicate(&next) {
                    *sum = next;
                    return None;
                }
            }
            let mut accepted = 0;
            let mut rejected = len;
            while rejected - accepted > 1 {
                let middle = accepted + (rejected - accepted) / 2;
                let value = Self::uniform_value(inherited.as_ref(), middle);
                if predicate(&M::op(&value, sum)) {
                    accepted = middle;
                } else {
                    rejected = middle;
                }
            }
            return Some(end - accepted);
        }
        let middle = lower + self.nodes[a as usize].len;
        self.min_left_node(
            Traversal {
                node: b,
                lower: middle,
                upper,
            },
            right,
            inherited.clone(),
            predicate,
            sum,
        )
        .or_else(|| {
            self.min_left_node(
                Traversal {
                    node: a,
                    lower,
                    upper: middle,
                },
                right,
                inherited,
                predicate,
                sum,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 区間の各値に同じ整数を加える作用。
    #[derive(Clone)]
    struct Add(i64);

    impl RangeAction<i64> for Add {
        /// 和には増分に区間長を掛けて加える。
        fn apply(&self, value: &i64, len: usize) -> i64 {
            value + self.0 * len as i64
        }

        /// 二つの加算をまとめる。
        fn composition(&self, other: &Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    /// 区間の各値を `a * x + b` に置き換える作用。
    #[derive(Clone)]
    struct Affine {
        /// 乗数。
        a: i64,
        /// 加数。
        b: i64,
    }

    impl RangeAction<i64> for Affine {
        /// 区間和に各点のアフィン変換をまとめて適用する。
        fn apply(&self, value: &i64, len: usize) -> i64 {
            self.a * value + self.b * len as i64
        }

        /// 古い作用 `self` の後に `other` を合成する。
        fn composition(&self, other: &Self) -> Self {
            Self {
                a: other.a * self.a,
                b: other.a * self.b + other.b,
            }
        }
    }

    /// 未生成区間への作用と集約を確認する。
    mod effect {
        use super::*;

        /// Scenario: 全域作用の後に一部だけ変更しても、他の点の値を保つ。
        /// - Given: まだ根しかない長さ 8 の木がある。
        /// - When: 全域に加算し、一部の区間へさらに加算する。
        /// - Then: 未生成だった点も区間長に応じた値を持つ。
        #[test]
        fn applies_to_uncreated_ranges() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(8);
            // When
            sut.effect(.., Add(2));
            assert_eq!(1, sut.nodes.len());
            sut.effect(3..6, Add(5));
            // Then
            assert_eq!(31, sut.fold(..));
            assert_eq!(21, sut.fold(3..6));
            assert_eq!(2, sut.get(0));
            assert_eq!(7, sut.get(5));
            assert_eq!(2, sut.get(7));
        }

        /// Scenario: 作用順序が異なるアフィン変換も正しい。
        /// - Given: 値がすべて 0 の木がある。
        /// - When: 全域・一部・全域の順に作用する。
        /// - Then: 古い作用から新しい作用への順番で値が決まる。
        #[test]
        fn composes_noncommutative_actions_in_time_order() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Affine>::new(4);
            // When
            sut.effect(.., Affine { a: 1, b: 2 });
            sut.effect(1..3, Affine { a: 3, b: 1 });
            sut.effect(.., Affine { a: 2, b: 4 });
            // Then
            assert_eq!(8, sut.get(0));
            assert_eq!(18, sut.get(1));
            assert_eq!(18, sut.get(2));
            assert_eq!(8, sut.get(3));
            assert_eq!(52, sut.fold(..));
        }

        /// Scenario: 複数の区間更新後も素朴な配列と一致する。
        /// - Given: 長さ 9 の木と配列がある。
        /// - When: 重なり合う区間への加算を繰り返す。
        /// - Then: すべての半開区間の集約値が一致する。
        #[test]
        fn matches_naive_range_sums() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(9);
            let mut values = [0_i64; 9];
            // When
            for step in 0..24 {
                let left = step * 7 % 10;
                let right = left + step * 5 % (10 - left);
                let delta = step as i64 - 8;
                sut.effect(left..right, Add(delta));
                for value in &mut values[left..right] {
                    *value += delta;
                }

                // Then
                for query_left in 0..=9 {
                    for query_right in query_left..=9 {
                        let expected = values[query_left..query_right].iter().sum::<i64>();
                        assert_eq!(expected, sut.fold(query_left..query_right));
                    }
                }
            }
        }
    }

    /// 境界探索が未生成区間と未伝播作用を扱えることを確認する。
    mod boundary {
        use super::super::super::super::super::algebra::semi_group;
        use super::*;

        /// 文字列を左から順に連結するモノイド。
        struct Concat;

        impl semi_group::SemiGroup for Concat {
            type S = String;

            /// 左右の順序を保って文字列を結合する。
            fn op(left: &Self::S, right: &Self::S) -> Self::S {
                left.clone() + right
            }
        }

        impl monoid::Monoid for Concat {
            /// 空文字列を単位元として返す。
            fn id() -> Self::S {
                String::new()
            }
        }

        /// 区間中の各文字を一つの文字へ置き換える作用。
        #[derive(Clone)]
        struct Fill(char);

        impl RangeAction<String> for Fill {
            /// 区間長と同じ個数の文字を返す。
            fn apply(&self, _value: &String, len: usize) -> String {
                self.0.to_string().repeat(len)
            }

            /// 後から指定された文字を優先する。
            fn composition(&self, other: &Self) -> Self {
                other.clone()
            }
        }

        /// Scenario: 非可換な連結でも集約順序と左右の境界が保たれる。
        /// - Given: 文字列連結を用いる長さ 5 の木がある。
        /// - When: 全域と部分区間へ順に文字を設定する。
        /// - Then: 集約と境界探索は文字の並びに一致する。
        #[test]
        fn preserves_order_for_noncommutative_monoid() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<Concat, Fill>::new(5);
            // When
            sut.effect(.., Fill('x'));
            sut.effect(1..4, Fill('a'));
            sut.effect(2..=2, Fill('b'));
            // Then
            assert_eq!("xabax", sut.fold(..));
            assert_eq!("aba", sut.fold(1..4));
            assert_eq!(2, sut.max_right(0, |value| !value.contains("ab")));
            assert_eq!(3, sut.min_left(5, |value| !value.contains("ba")));
        }

        /// Scenario: 両側の探索は配列で求めた境界に一致する。
        /// - Given: 長さ 9 の木と非負値の配列がある。
        /// - When: 区間加算後、各位置から合計の閾値を探す。
        /// - Then: 左右の境界が素朴な走査と一致する。
        #[test]
        fn searches_both_directions() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(9);
            let mut values = [0_i64; 9];
            // When
            for (left, right, delta) in [(0, 9, 2), (2, 7, 3), (4, 5, 6)] {
                sut.effect(left..right, Add(delta));
                for value in &mut values[left..right] {
                    *value += delta;
                }
            }
            let before = sut.nodes.len();

            // Then
            for limit in [1, 3, 8, 20, 100] {
                for left in 0..=9 {
                    let mut expected = left;
                    let mut sum = 0;
                    while expected < 9 && sum + values[expected] < limit {
                        sum += values[expected];
                        expected += 1;
                    }
                    assert_eq!(expected, sut.max_right(left, |&value| value < limit));
                }
                for right in 0..=9 {
                    let mut expected = right;
                    let mut sum = 0;
                    while expected > 0 && sum + values[expected - 1] < limit {
                        expected -= 1;
                        sum += values[expected];
                    }
                    assert_eq!(expected, sut.min_left(right, |&value| value < limit));
                }
            }
            assert_eq!(before, sut.nodes.len());
        }

        /// Scenario: 広い論理範囲でも未生成部分の位置を探せる。
        /// - Given: 長さ 10 億の木がある。
        /// - When: 末尾の一点へ作用する。
        /// - Then: 境界探索が末尾の座標を返す。
        #[test]
        fn searches_large_sparse_range() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(1_000_000_000);
            // When
            sut.effect(999_999_999..1_000_000_000, Add(7));
            // Then
            assert_eq!(999_999_999, sut.max_right(0, |&sum| sum < 7));
            assert_eq!(1_000_000_000, sut.min_left(1_000_000_000, |&sum| sum < 7));
            assert_eq!(0, sut.fold(..999_999_999));
        }
    }

    /// 空区間と範囲外を確認する。
    mod bounds {
        use super::*;

        /// Scenario: 包含端点と無制限端点で作用と集約を指定できる。
        /// - Given: 長さ 4 の空の木がある。
        /// - When: 先頭と末尾を異なる範囲表記で更新する。
        /// - Then: 集約は指定された端点を正しく含む。
        #[test]
        fn accepts_inclusive_and_unbounded_ranges() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.effect(..=1, Add(3));
            sut.effect(2.., Add(2));
            // Then
            assert_eq!(10, sut.fold(..));
            assert_eq!(5, sut.fold(1..=2));
            assert_eq!(4, sut.fold(2..));
        }

        /// Scenario: 長さ 0 の木は空区間の作用と集約を受け付ける。
        /// - Given: 長さ 0 の木がある。
        /// - When: 空区間に作用して集約・境界探索する。
        /// - Then: 集約は単位元、境界は 0 を返す。
        #[test]
        fn handles_empty_tree() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(0);
            // When
            sut.effect(.., Add(3));
            // Then
            assert!(sut.is_empty());
            assert_eq!(0, sut.len());
            assert_eq!(0, sut.fold(..));
            assert_eq!(0, sut.max_right(0, |&sum| sum < 1));
            assert_eq!(0, sut.min_left(0, |&sum| sum < 1));
            assert_eq!(1, sut.nodes.len());
        }

        /// Scenario: 範囲外の右端を拒否する。
        /// - Given: 長さ 4 の木がある。
        /// - When: 右端 5 まで作用する。
        /// - Then: 範囲外としてパニックする。
        #[test]
        #[should_panic(expected = "range out of bounds")]
        fn rejects_out_of_bounds_range() {
            // Given
            let mut sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.effect(0..5, Add(1));
        }

        /// Scenario: 単位元を拒否する述語は境界探索に使用できない。
        /// - Given: 長さ 4 の木がある。
        /// - When: 偽の述語で境界探索する。
        /// - Then: 述語条件違反としてパニックする。
        #[test]
        #[should_panic(expected = "predicate must accept the identity")]
        fn rejects_predicate_on_identity() {
            // Given
            let sut = SegmentTreeLazySparseOnline::<monoid::AddMonoid, Add>::new(4);
            // When
            sut.max_right(0, |_| false);
        }
    }
}

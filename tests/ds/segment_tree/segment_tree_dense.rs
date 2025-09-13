// tests/ds/segment_tree/segment_tree_dense.rs
use anmitsu::{
    algebra::{
        monoid::{self, AddMonoid, Monoid},
        semi_group,
    },
    ds::segment_tree::segment_tree_dense::{self, SegmentTreeDense},
};
use rand::{self, Rng};

// A 2x2 matrix for testing non-commutative operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Matrix2x2 {
    mat: [[u64; 2]; 2],
}

// A monoid for matrix multiplication.
#[derive(Clone)]
struct MatrixMulMonoid;

impl semi_group::SemiGroup for MatrixMulMonoid {
    type S = Matrix2x2;

    fn op(a: &Self::S, b: &Self::S) -> Self::S {
        let mut res = Matrix2x2 { mat: [[0; 2]; 2] };
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    res.mat[i][j] =
                        res.mat[i][j].saturating_add(a.mat[i][k].saturating_mul(b.mat[k][j]));
                }
            }
        }
        res
    }
}

impl monoid::Monoid for MatrixMulMonoid {
    fn id() -> Self::S {
        Matrix2x2 {
            mat: [[1, 0], [0, 1]],
        }
    }
}

/// A naive implementation of a segment tree for testing purposes.
/// This implementation uses a simple vector and performs operations with loops,
/// serving as a baseline for verifying the correctness of the optimized `SegmentTreeDense`.
/// test 用の segment tree の愚直な実装である.
/// この実装は単純な vector を用い, loop を使って操作を実行する.
/// これにより, 最適化された `SegmentTreeDense` の正当性を検証するための基準として機能する.
#[derive(Clone)]
pub struct NaiveSegmentTree<M>
where
    M: monoid::Monoid,
{
    data: Vec<M::S>,
}

impl<M> NaiveSegmentTree<M>
where
    M: monoid::Monoid,
    M::S: Clone,
{
    /// Creates a new `NaiveSegmentTree` with `n` elements.
    /// `n` 個の要素を持つ `NaiveSegmentTree` を生成する.
    ///
    /// # Args
    /// - `n`: The size of the data structure.
    ///        データ構造のサイズ.
    ///
    /// # Returns
    /// `NaiveSegmentTree<M>`: A new instance initialized with the monoid's identity element.
    ///                        モノイドの単位元で初期化された新しいインスタンス.
    ///
    /// # Complexity
    /// - Time complexity: O(n), where `n` is the size.
    ///                          ここで `n` はサイズである.
    /// - Space complexity: O(n).
    pub fn new(n: usize) -> Self {
        Self {
            data: vec![M::id(); n],
        }
    }

    /// Returns the size of the data structure.
    /// データ構造のサイズを返す.
    ///
    /// # Returns
    /// `usize`: The number of elements.
    ///          要素数.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Sets the value at index `idx` to `x`.
    /// In this naive implementation, this operation is not lazy.
    /// インデックス `idx` の値を `x` にセットする.
    /// この愚直な実装では, この操作は遅延実行されない.
    ///
    /// # Args
    /// - `idx`: The index to set.
    ///          セット対象のインデックス.
    /// - `x`: The new value.
    ///        新しい値.
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds.
    /// `idx` が範囲外の場合にパニックする.
    pub fn set(&mut self, idx: usize, x: M::S) {
        self.data[idx] = x;
    }

    /// This method is for interface compatibility with `SegmentTreeDense`.
    /// In this naive implementation, it does nothing as data is always up-to-date.
    /// このメソッドは `SegmentTreeDense` とのインターフェース互換性のために存在する.
    /// この愚直な実装では, データは常に最新であるため何もしない.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    pub fn build(&mut self) {
        // This implementation is always up-to-date, so build does nothing.
    }

    /// Updates the value at index `idx` to `x`.
    /// インデックス `idx` の値を `x` に更新する.
    ///
    /// # Args
    /// - `idx`: The index to update.
    ///          更新対象のインデックス.
    /// - `x`: The new value.
    ///        新しい値.
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds.
    /// `idx` が範囲外の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    pub fn update(&mut self, idx: usize, x: M::S) {
        self.data[idx] = x;
    }

    /// Gets the value at index `idx`.
    /// インデックス `idx` の値を取得する.
    ///
    /// # Args
    /// - `idx`: The index to retrieve.
    ///          値を取得するインデックス.
    ///
    /// # Returns
    /// `M::S`: The value at `idx`.
    ///         `idx` の値.
    ///
    /// # Panics
    /// Panics if `idx` is out of bounds.
    /// `idx` が範囲外の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(1).
    /// - Space complexity: O(1).
    pub fn get(&self, idx: usize) -> M::S {
        self.data[idx].clone()
    }

    /// Performs a range fold (query) on the interval `[l, r)`.
    /// 区間 `[l, r)` 上の値に対して `fold` (畳み込み) を行う `query` を実行する.
    ///
    /// # Args
    /// - `l`: The start index of the range (inclusive).
    ///        `query` 区間の開始インデックス (含む).
    /// - `r`: The end index of the range (exclusive).
    ///        `query` 区間の終了インデックス (含まない).
    ///
    /// # Returns
    /// `M::S`: The folded result. Returns the identity element for an empty range.
    ///         畳み込み結果. 区間が空の場合は単位元を返す.
    ///
    /// # Panics
    /// Panics if `r > self.len()`.
    /// `r > self.len()` の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(r - l).
    /// - Space complexity: O(1).
    pub fn fold(&self, l: usize, r: usize) -> M::S {
        assert!(r <= self.len());
        if l >= r {
            return M::id();
        }

        let mut res = M::id();
        for i in l..r {
            res = M::op(&res, &self.data[i]);
        }
        res
    }

    /// Finds the maximum `r` in `[l, self.len()]` such that a predicate on `fold(l, r)` holds.
    /// `fold(l, r)` に対する述語が `true` となるような, `[l, self.len()]` 内の最大の `r` を探索する.
    ///
    /// # Args
    /// - `l`: The start index of the range.
    ///        範囲の開始インデックス.
    /// - `f`: The predicate function.
    ///        述語関数.
    ///
    /// # Returns
    /// `usize`: The maximum `r` satisfying the predicate.
    ///          述語を満たす最大の `r`.
    ///
    /// # Complexity
    /// - Time complexity: O(N), where N is the number of elements.
    ///                          ここで N は要素数である.
    /// - Space complexity: O(1).
    pub fn max_right<F>(&self, l: usize, f: F) -> usize
    where
        F: Fn(&M::S) -> bool,
    {
        assert!(
            f(&M::id()),
            "predicate must be true for the identity element"
        );
        assert!(l <= self.len());

        let mut sum = M::id();
        for r in l..self.len() {
            let next_sum = M::op(&sum, &self.data[r]);
            if !f(&next_sum) {
                return r;
            }
            sum = next_sum;
        }
        self.len()
    }

    /// Finds the minimum `l` in `[0, r]` such that a predicate on `fold(l, r)` holds.
    /// `fold(l, r)` に対する述語が `true` となるような, `[0, r]` 内の最小の `l` を探索する.
    ///
    /// # Args
    /// - `r`: The end index of the range.
    ///        範囲の終了インデックス.
    /// - `f`: The predicate function.
    ///        述語関数.
    ///
    /// # Returns
    /// `usize`: The minimum `l` satisfying the predicate.
    ///          述語を満たす最小の `l`.
    ///
    /// # Complexity
    /// - Time complexity: O(N), where N is the number of elements.
    ///                          ここで N は要素数である.
    /// - Space complexity: O(1).
    pub fn min_left<L>(&self, r: usize, f: L) -> usize
    where
        L: Fn(&M::S) -> bool,
    {
        assert!(
            f(&M::id()),
            "predicate must be true for the identity element"
        );
        assert!(r <= self.len());

        let mut sum = M::id();
        for l in (0..r).rev() {
            let next_sum = M::op(&self.data[l], &sum);
            if !f(&next_sum) {
                return l + 1;
            }
            sum = next_sum;
        }
        0
    }
}

// Helper function to create a NaiveSegmentTree and a SegmentTreeDense from the same data.
// 同じデータから NaiveSegmentTree と SegmentTreeDense を作成するヘルパー関数.
fn setup_trees<M>(initial_data: &[M::S]) -> (NaiveSegmentTree<M>, SegmentTreeDense<M>)
where
    M: monoid::Monoid,
    M::S: Clone,
{
    let n = initial_data.len();
    let mut naive = NaiveSegmentTree::<M>::new(n);
    let mut dense = segment_tree_dense::SegmentTreeDense::<M>::new(n);

    for (i, val) in initial_data.iter().enumerate() {
        naive.set(i, val.clone());
        dense.set(i, val.clone());
    }
    dense.build();

    (naive, dense)
}

#[test]
fn test_new_and_len() {
    // Test with size 0.
    let seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(0);
    assert_eq!(seg.len(), 0);

    // Test with a non-zero size.
    let seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(10);
    assert_eq!(seg.len(), 10);
}

#[test]
fn test_set_build_get_fold_add() {
    let initial_data = vec![1, 10, 100, 1000, 10000];
    let n = initial_data.len();
    let (naive, dense) = setup_trees::<AddMonoid>(&initial_data);

    // Test get.
    for i in 0..n {
        assert_eq!(naive.get(i), dense.get(i), "get({}) failed", i);
    }

    // Test fold over various ranges.
    for i in 0..=n {
        for j in i..=n {
            assert_eq!(
                naive.fold(i, j),
                dense.fold(i, j),
                "fold({}, {}) failed",
                i,
                j
            );
        }
    }
}

#[test]
fn test_update_add() {
    let initial_data = vec![1, 2, 3, 4, 5];
    let n = initial_data.len();
    let (mut naive, mut dense) = setup_trees::<AddMonoid>(&initial_data);

    // Update a value.
    dense.update(2, 10);
    naive.update(2, 10);

    // Verify the updated value and folds.
    assert_eq!(10, dense.get(2));
    assert_eq!(naive.fold(0, n), dense.fold(0, n));
    assert_eq!(naive.fold(1, 4), dense.fold(1, 4));
}

#[test]
fn test_fold_edge_cases() {
    let seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(5);
    // Test empty range.
    assert_eq!(AddMonoid::id(), seg.fold(0, 0));
    assert_eq!(AddMonoid::id(), seg.fold(3, 3));
    assert_eq!(AddMonoid::id(), seg.fold(5, 5));
}

#[test]
#[should_panic]
fn test_get_out_of_bounds() {
    let seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(5);
    seg.get(5);
}

#[test]
#[should_panic]
fn test_set_out_of_bounds() {
    let mut seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(5);
    seg.set(5, 1);
}

#[test]
#[should_panic]
fn test_update_out_of_bounds() {
    let mut seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(5);
    seg.update(5, 1);
}

#[test]
#[should_panic]
fn test_fold_out_of_bounds() {
    let seg = segment_tree_dense::SegmentTreeDense::<AddMonoid>::new(5);
    seg.fold(0, 6);
}

#[test]
fn test_max_right_add() {
    let initial_data = vec![1, 2, 3, 4, 5];
    let n = initial_data.len();
    let (_, dense) = setup_trees::<AddMonoid>(&initial_data);

    // from l=1, sum < 10 -> [1,4) = 2+3+4=9, [1,5)=14
    assert_eq!(4, dense.max_right(1, |&sum| sum < 10));
    // from l=0, sum <= 6 -> [0,3) = 1+2+3=6, [0,4)=10
    assert_eq!(3, dense.max_right(0, |&sum| sum <= 6));
    // Predicate is always true.
    assert_eq!(n, dense.max_right(0, |&_sum| true));
    // Predicate is true only for id.
    assert_eq!(0, dense.max_right(0, |&sum| sum == 0));
    // Start from the end.
    assert_eq!(n, dense.max_right(n, |&_sum| true));
}

#[test]
fn test_min_left_add() {
    let initial_data = vec![1, 2, 3, 4, 5];
    let n = initial_data.len();
    let (naive, mut dense) = setup_trees::<AddMonoid>(&initial_data);

    // until r=4, sum < 10 -> [1,4) = 2+3+4=9, [0,4)=10
    assert_eq!(1, dense.min_left(4, |&sum| sum < 10));
    // until r=5, sum <= 15 -> [0,5) = 15
    assert_eq!(0, dense.min_left(5, |&sum| sum <= 15));
    // Predicate is always true.
    assert_eq!(0, dense.min_left(n, |&_sum| true));
    // Predicate is true only for id.
    assert_eq!(n, dense.min_left(n, |&sum| sum == 0));
    // Start from the beginning.
    assert_eq!(0, dense.min_left(0, |&_sum| true));
    // Compare with naive implementation
    assert_eq!(
        naive.min_left(4, |&s| s < 10),
        dense.min_left(4, |&s| s < 10)
    );
}

// Function to generate a random 2x2 matrix.
fn rand_matrix() -> Matrix2x2 {
    let mut rng = rand::rng();
    Matrix2x2 {
        mat: [
            [rng.random_range(1..=5), rng.random_range(1..=5)],
            [rng.random_range(1..=5), rng.random_range(1..=5)],
        ],
    }
}

#[test]
fn test_randomized_comparison_matrix() {
    const N: usize = 10000;
    const Q: usize = 10000;

    let initial_data: Vec<Matrix2x2> = (0..N).map(|_| rand_matrix()).collect();
    let (mut naive, mut dense) = setup_trees::<MatrixMulMonoid>(&initial_data);

    let mut rng = rand::rng();

    for _ in 0..Q {
        let op_type = rng.random_range(0..4);

        match op_type {
            0 => {
                // Test update
                let idx = rng.random_range(0..N);
                let val = rand_matrix();
                dense.update(idx, val);
                naive.update(idx, val);
                assert_eq!(naive.get(idx), dense.get(idx));
            }
            1 => {
                // Test fold
                let mut l = rng.random_range(0..=N);
                let mut r = rng.random_range(0..=N);
                if l > r {
                    std::mem::swap(&mut l, &mut r);
                }
                assert_eq!(naive.fold(l, r), dense.fold(l, r));
            }
            2 => {
                // Test max_right
                let l = rng.random_range(0..=N);
                // A simple predicate for testing: check if the top-left element is below a threshold.
                let threshold = 1_000_000;
                let f = |m: &Matrix2x2| m.mat[0][0] < threshold;
                assert_eq!(naive.max_right(l, f), dense.max_right(l, f));
            }
            3 => {
                // Test min_left
                let r = rng.random_range(0..=N);
                let threshold = 1_000_000;
                let f = |m: &Matrix2x2| m.mat[0][0] < threshold;
                assert_eq!(naive.min_left(r, f), dense.min_left(r, f));
            }
            _ => unreachable!(),
        }
    }
}

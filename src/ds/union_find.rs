use std::cmp;

/// A data structure for Disjoint Set Union (DSU), also known as Union-Find.
/// It efficiently manages a collection of disjoint sets.
///
/// Union-Find (Disjoint Set Union) データ構造を実装する.
/// 互いに素な集合のコレクションを効率的に管理する.
#[derive(Clone)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
    group_next: Vec<usize>,
}

impl UnionFind {
    /// Creates a new `UnionFind` instance with `n` elements.
    /// Initially, each element is in its own set.
    ///
    /// `n` 個の要素を持つ新しい `UnionFind` インスタンスを生成する.
    /// 初期状態では, 各要素がそれぞれ独立した集合に属する.
    ///
    /// # Args
    /// * `n`: The number of elements.
    ///        要素数.
    ///
    /// # Returns
    /// A new `UnionFind` instance.
    /// 新しい `UnionFind` インスタンスを返す.
    ///
    /// # Complexity
    /// - Time complexity: O(N), where N is the number of elements.
    ///                    ここで N は要素数である.
    /// - Space complexity: O(N), where N is the number of elements.
    ///                     ここで N は要素数である.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::union_find::UnionFind; // NOTE: Assuming this path
    /// let uf = UnionFind::new(5);
    /// ```
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect::<Vec<_>>(),
            rank: vec![0; n],
            size: vec![1; n],
            group_next: (0..n).collect::<Vec<_>>(),
        }
    }

    /// Returns the number of elements managed by this `UnionFind`.
    /// `UnionFind` が管理する要素数を返す.
    ///
    /// # Returns
    /// The total number of elements.
    /// 全要素数.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// Checks if an element `x` is the root of its set.
    /// 要素 `x` がその集合の根であるかどうかを判定する.
    ///
    /// # Args
    /// * `x`: The element to check.
    ///        判定対象の要素.
    ///
    /// # Returns
    /// `true` if `x` is a root, otherwise `false`.
    /// `x` が根であれば `true` を, そうでなければ `false` を返す.
    ///
    /// # Panics
    /// Panics if `x` is out of bounds.
    /// `x` が範囲外の場合にパニックする.
    pub fn is_root(&self, x: usize) -> bool {
        if x >= self.len() {
            panic!(
                "Index {} is out of bounds for UnionFind with size {}",
                x,
                self.len()
            );
        }

        self.parent[x] == x
    }

    /// Finds the root of the set containing element `x`.
    /// This method also performs path compression.
    ///
    /// 要素 `x` を含む集合の根を見つける.
    /// このメソッドは経路圧縮も同時に行う.
    ///
    /// # Args
    /// * `x`: The element to find the root of.
    ///        根を探す対象の要素.
    ///
    /// # Returns
    /// The root of the set.
    /// 属する集合の根を返す.
    ///
    /// # Panics
    /// Panics if `x` is out of bounds.
    /// `x` が範囲外の場合にパニックする.
    ///
    /// # Complexity
    /// - Time complexity: O(α(N)), where α is the inverse Ackermann function.
    ///                    ここで α は逆アッカーマン関数である.
    pub fn find(&mut self, x: usize) -> usize {
        if x >= self.parent.len() {
            panic!(
                "Index {} is out of bounds for UnionFind with size {}",
                x,
                self.parent.len()
            );
        }

        // First pass: walk up the tree (without recursion, to avoid both the
        // function-call overhead and the risk of stack overflow on a deep,
        // not-yet-compressed tree) until the root is found.
        // 第1パス: (関数呼び出しのオーバーヘッドと、経路圧縮前の深い木での
        // スタックオーバーフローの両方を避けるため) 再帰を使わずに根まで
        // たどる.
        let mut root = x;
        while self.parent[root] != root {
            root = self.parent[root];
        }

        // Second pass: path compression. Point every node on the path
        // directly to the root.
        // 第2パス: 経路圧縮. 経路上の全ノードを根へ直接つなぎ変える.
        let mut current = x;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }

        root
    }

    /// Merges the sets containing elements `x` and `y`.
    /// This method uses union by rank to keep the tree structure balanced.
    ///
    /// 要素 `x` と要素 `y` を含む集合をマージする.
    /// このメソッドはランクによる統合 (union by rank) を用いて木構造のバランスを保つ.
    ///
    /// # Args
    /// * `x`: An element in the first set.
    ///        最初の集合に含まれる要素.
    /// * `y`: An element in the second set.
    ///        二番目の集合に含まれる要素.
    ///
    /// # Panics
    /// Panics if `x` or `y` are out of bounds.
    /// `x` または `y` が範囲外の場合にパニックする.
    pub fn union(&mut self, x: usize, y: usize) {
        let len = self.len();
        if x >= len || y >= len {
            panic!(
                "Index out of bounds for union: x={}, y={}, len={}",
                x, y, len
            );
        }

        let (root_x, root_y) = (self.find(x), self.find(y));

        if root_x != root_y {
            // Link the `group_next` pointers to enable group traversal.
            // This is a specific implementation detail for `get_group`.
            self.group_next.swap(root_x, root_y);

            // Union by rank: Attach the shorter tree to the root of the taller tree.
            // This helps to keep the trees from becoming too deep.
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                cmp::Ordering::Less => {
                    self.parent[root_x] = root_y;
                    self.size[root_y] += self.size[root_x];
                }
                cmp::Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                    self.rank[root_x] += 1;
                }
                cmp::Ordering::Greater => {
                    self.parent[root_y] = root_x;
                    self.size[root_x] += self.size[root_y];
                }
            }
        }
    }

    /// Checks if elements `x` and `y` are in the same set.
    /// 要素 `x` と `y` が同じ集合に属するかどうかを判定する.
    ///
    /// # Args
    /// * `x`: The first element.
    ///        最初の要素.
    /// * `y`: The second element.
    ///        二番目の要素.
    ///
    /// # Returns
    /// `true` if `x` and `y` are in the same set, otherwise `false`.
    /// `x` と `y` が同じ集合に属する場合は `true` を, そうでなければ `false` を返す.
    ///
    /// # Panics
    /// Panics if `x` or `y` are out of bounds.
    /// `x` または `y` が範囲外の場合にパニックする.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::union_find::UnionFind; // NOTE: Assuming this path
    /// let mut uf = UnionFind::new(3);
    /// uf.union(0, 1);
    /// assert_eq!(uf.is_same(0, 1), true);
    /// assert_eq!(uf.is_same(0, 2), false);
    /// ```
    pub fn is_same(&mut self, x: usize, y: usize) -> bool {
        let len = self.len();
        if x >= len || y >= len {
            panic!(
                "Index out of bounds for is_same: x={}, y={}, len={}",
                x, y, len
            );
        }

        self.find(x) == self.find(y)
    }

    /// Returns the size of the set containing element `x`.
    /// 要素 `x` を含む集合のサイズを返す.
    ///
    /// # Args
    /// * `x`: The element.
    ///        対象の要素.
    ///
    /// # Returns
    /// The size of the set.
    /// 集合のサイズ.
    ///
    /// # Panics
    /// Panics if `x` is out of bounds.
    /// `x` が範囲外の場合にパニックする.
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::ds::union_find::UnionFind; // NOTE: Assuming this path
    /// let mut uf = UnionFind::new(5);
    /// uf.union(0, 1);
    /// uf.union(0, 2);
    /// assert_eq!(uf.get_size(0), 3);
    /// assert_eq!(uf.get_size(3), 1);
    /// ```
    pub fn get_size(&mut self, x: usize) -> usize {
        if x >= self.len() {
            panic!(
                "Index {} is out of bounds for UnionFind with size {}",
                x,
                self.len()
            );
        }

        if self.is_root(x) {
            self.size[x]
        } else {
            let root = self.find(x);
            self.size[root]
        }
    }

    /// Returns all elements in the set containing element `x`.
    /// Note: This implementation may be inefficient and is not a standard part of Union-Find.
    ///
    /// 要素 `x` が属する集合の全要素を取得する.
    /// 注意: この実装は非効率な場合があり, Union-Find の標準的な機能ではない.
    ///
    /// # Args
    /// * `x`: The element.
    ///        対象の要素.
    ///
    /// # Returns
    /// A sorted vector of all elements in the set.
    /// 集合に含まれる全要素からなるソート済みのベクター.
    ///
    /// # Complexity
    /// - Time complexity: O(S log S), where S is the size of the group.
    ///                    ここで S はグループのサイズである.
    ///
    /// # Panics
    /// Panics if `x` is out of bounds.
    /// `x` が範囲外の場合にパニックする.
    pub fn get_group(&mut self, x: usize) -> Vec<usize> {
        if x >= self.len() {
            panic!(
                "Index {} is out of bounds for UnionFind with size {}",
                x,
                self.len()
            );
        }

        let mut res = vec![x];
        let mut v = x;

        // Traverse the cyclic list created by `group_next` to collect all members of the group.
        // This is a custom extension to the standard Union-Find structure.
        while self.group_next[v] != x {
            res.push(self.group_next[v]);
            v = self.group_next[v];
        }

        // Sort for a consistent and predictable output order.
        res.sort();
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // new のテスト: 生成直後の状態を検証する。
    mod new {
        use super::*;

        /// Scenario: 新規に生成した `UnionFind` は、各要素が独立した集合をなす。
        /// - Given: 要素数 4 を指定する。
        /// - When: `UnionFind::new(4)` を呼ぶ。
        /// - Then: すべての要素が自身を根とし、サイズは 1 であり、
        ///   互いに異なる集合に属する。
        #[test]
        fn creates_singleton_sets_for_each_element() {
            // Given, When
            let mut sut = UnionFind::new(4);
            // Then
            assert_eq!(4, sut.len());
            for i in 0..4 {
                assert!(sut.is_root(i));
                assert_eq!(1, sut.get_size(i));
            }
            assert!(!sut.is_same(0, 1));
            assert!(!sut.is_same(2, 3));
        }

        /// Scenario: 要素数 0 で生成すると、空の `UnionFind` になる (境界値)。
        /// - Given: 要素数 0 を指定する。
        /// - When: `UnionFind::new(0)` を呼ぶ。
        /// - Then: `len()` が `0` を返す。
        #[test]
        fn creates_empty_structure_for_zero_elements() {
            // Given, When
            let sut = UnionFind::new(0);
            // Then
            assert_eq!(0, sut.len());
        }
    }

    // find のテスト: 異常系 (範囲外アクセス) を検証する。
    mod find {
        use super::*;

        /// Scenario: 範囲外の要素を指定すると、パニックする (異常系)。
        /// - Given: 要素数 5 の `UnionFind` がある。
        /// - When: 範囲外の要素 `5` に対して `find` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 5 is out of bounds for UnionFind with size 5")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = UnionFind::new(5);
            // When, Then (panic)
            sut.find(5);
        }

        /// Scenario: 空の `UnionFind` に対して呼ぶと、パニックする (境界値・異常系)。
        /// - Given: 要素数 0 の `UnionFind` がある。
        /// - When: 要素 `0` に対して `find` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
        fn panics_on_empty_structure() {
            // Given
            let mut sut = UnionFind::new(0);
            // When, Then (panic)
            sut.find(0);
        }
    }

    // union のテスト: 呼び出し後の状態変化を検証する。
    mod union {
        use super::*;

        /// Scenario: 2 つの集合を結合すると、それらは同じ集合になる。
        /// - Given: 要素数 4 の `UnionFind` がある。
        /// - When: `(0, 1)` と `(2, 3)` をそれぞれ結合する。
        /// - Then: 各ペアは同じ集合に属し、異なるペア同士はまだ別集合である。
        #[test]
        fn merges_two_sets() {
            // Given
            let mut sut = UnionFind::new(4);
            // When
            sut.union(0, 1);
            sut.union(2, 3);
            // Then
            assert!(sut.is_same(0, 1));
            assert!(sut.is_same(2, 3));
            assert!(!sut.is_same(1, 2));
        }

        /// Scenario: 結合済みの集合同士をさらに結合すると、推移的に同じ集合になる。
        /// - Given: `(0, 1)` と `(2, 3)` がそれぞれ結合済みの `UnionFind` がある。
        /// - When: `(1, 2)` を結合する。
        /// - Then: `0`, `1`, `2`, `3` のすべてが同じ集合に属する。
        #[test]
        fn merges_transitively_across_existing_groups() {
            // Given
            let mut sut = UnionFind::new(4);
            sut.union(0, 1);
            sut.union(2, 3);
            // When
            sut.union(1, 2);
            // Then
            assert!(sut.is_same(0, 2));
            assert!(sut.is_same(0, 3));
        }

        /// Scenario: 同じ要素同士の結合は何も変化させない。
        /// - Given: 要素数 1 の `UnionFind` がある。
        /// - When: 要素 `0` を自分自身と結合する。
        /// - Then: 同じ集合のままであり、サイズは `1`、グループは自分自身のみを含む。
        #[test]
        fn self_union_is_no_op() {
            // Given
            let mut sut = UnionFind::new(1);
            // When
            sut.union(0, 0);
            // Then
            assert!(sut.is_same(0, 0));
            assert_eq!(1, sut.get_size(0));
            assert_eq!(vec![0], sut.get_group(0));
        }

        /// Scenario: 同じペアを繰り返し結合しても、サイズとグループは変化しない。
        /// - Given: 要素数 3 の `UnionFind` がある。
        /// - When: `(0, 1)` を 2 回結合する。
        /// - Then: サイズは `2` のままであり、グループは `[0, 1]` のままである。
        #[test]
        fn redundant_union_does_not_change_size() {
            // Given
            let mut sut = UnionFind::new(3);
            // When
            sut.union(0, 1);
            sut.union(0, 1);
            // Then
            assert!(sut.is_same(0, 1));
            assert_eq!(2, sut.get_size(0));
            assert_eq!(vec![0, 1], sut.get_group(1));
        }

        /// Scenario: 範囲外の要素を指定すると、パニックする (異常系)。
        /// - Given: 要素数 5 の `UnionFind` がある。
        /// - When: 範囲外の要素を含む `(4, 5)` を結合しようとする。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index out of bounds for union: x=4, y=5, len=5")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = UnionFind::new(5);
            // When, Then (panic)
            sut.union(4, 5);
        }

        /// Scenario: 空の `UnionFind` に対して呼ぶと、パニックする (境界値・異常系)。
        /// - Given: 要素数 0 の `UnionFind` がある。
        /// - When: 要素 `0` 同士を結合しようとする。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index out of bounds for union: x=0, y=0, len=0")]
        fn panics_on_empty_structure() {
            // Given
            let mut sut = UnionFind::new(0);
            // When, Then (panic)
            sut.union(0, 0);
        }
    }

    // is_same のテスト: 異常系 (範囲外アクセス) を検証する。
    // 正常系は union のテストで間接的に検証済みである。
    mod is_same {
        use super::*;

        /// Scenario: 範囲外の要素を指定すると、パニックする (異常系)。
        /// - Given: 要素数 3 の `UnionFind` がある。
        /// - When: 範囲外の要素を含む `(0, 3)` で `is_same` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index out of bounds for is_same: x=0, y=3, len=3")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = UnionFind::new(3);
            // When, Then (panic)
            sut.is_same(0, 3);
        }

        /// Scenario: 空の `UnionFind` に対して呼ぶと、パニックする (境界値・異常系)。
        /// - Given: 要素数 0 の `UnionFind` がある。
        /// - When: 要素 `0` 同士で `is_same` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index out of bounds for is_same: x=0, y=0, len=0")]
        fn panics_on_empty_structure() {
            // Given
            let mut sut = UnionFind::new(0);
            // When, Then (panic)
            sut.is_same(0, 0);
        }
    }

    // get_size のテスト: 戻り値、および異常系を検証する。
    mod get_size {
        use super::*;

        /// Scenario: 複数回の結合後も、集合のサイズは正しく更新される。
        /// - Given: 要素数 5 の `UnionFind` に `(0, 1)`, `(0, 2)`, `(3, 4)` を結合する。
        /// - When: 根および根以外の要素から `get_size` を呼ぶ。
        /// - Then: それぞれの集合のサイズが正しく返る。
        #[test]
        fn returns_updated_size_after_unions() {
            // Given
            let mut sut = UnionFind::new(5);
            sut.union(0, 1);
            sut.union(0, 2);
            sut.union(3, 4);
            // When
            let size_0 = sut.get_size(0);
            let size_2 = sut.get_size(2); // 根以外からのクエリー
            let size_3 = sut.get_size(3);
            let size_4 = sut.get_size(4);
            // Then
            assert_eq!(3, size_0);
            assert_eq!(3, size_2);
            assert_eq!(2, size_3);
            assert_eq!(2, size_4);
        }

        /// Scenario: 範囲外の要素を指定すると、パニックする (異常系)。
        /// - Given: 要素数 3 の `UnionFind` がある。
        /// - When: 範囲外の要素 `3` に対して `get_size` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 3 is out of bounds for UnionFind with size 3")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = UnionFind::new(3);
            // When, Then (panic)
            sut.get_size(3);
        }

        /// Scenario: 空の `UnionFind` に対して呼ぶと、パニックする (境界値・異常系)。
        /// - Given: 要素数 0 の `UnionFind` がある。
        /// - When: 要素 `0` に対して `get_size` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
        fn panics_on_empty_structure() {
            // Given
            let mut sut = UnionFind::new(0);
            // When, Then (panic)
            sut.get_size(0);
        }
    }

    // get_group のテスト: 戻り値、および異常系を検証する。
    mod get_group {
        use super::*;

        /// Scenario: 結合後、グループはソート済みの全メンバーを返す。
        /// - Given: 要素数 6 の `UnionFind` に `(0, 1)`, `(2, 3)`, `(1, 2)`, `(4, 5)` を結合する。
        /// - When: 根および根以外の要素から `get_group` を呼ぶ。
        /// - Then: 各グループのメンバーがソート済みで返る。
        #[test]
        fn returns_sorted_members_after_merges() {
            // Given
            let mut sut = UnionFind::new(6);
            sut.union(0, 1);
            sut.union(2, 3);
            sut.union(1, 2); // {0,1} と {2,3} を統合
            sut.union(4, 5);
            // When
            let group_0 = sut.get_group(0);
            let group_3 = sut.get_group(3); // 根以外から
            let group_5 = sut.get_group(5);
            // Then
            assert_eq!(vec![0, 1, 2, 3], group_0);
            assert_eq!(vec![0, 1, 2, 3], group_3);
            assert_eq!(vec![4, 5], group_5);
        }

        /// Scenario: 範囲外の要素を指定すると、パニックする (異常系)。
        /// - Given: 要素数 2 の `UnionFind` がある。
        /// - When: 範囲外の要素 `2` に対して `get_group` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 2 is out of bounds for UnionFind with size 2")]
        fn panics_when_index_out_of_bounds() {
            // Given
            let mut sut = UnionFind::new(2);
            // When, Then (panic)
            sut.get_group(2);
        }

        /// Scenario: 空の `UnionFind` に対して呼ぶと、パニックする (境界値・異常系)。
        /// - Given: 要素数 0 の `UnionFind` がある。
        /// - When: 要素 `0` に対して `get_group` を呼ぶ。
        /// - Then: 範囲外である旨のメッセージでパニックする。
        #[test]
        #[should_panic(expected = "Index 0 is out of bounds for UnionFind with size 0")]
        fn panics_on_empty_structure() {
            // Given
            let mut sut = UnionFind::new(0);
            // When, Then (panic)
            sut.get_group(0);
        }
    }
}

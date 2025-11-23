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

        if self.is_root(x) {
            x
        } else {
            // Path compression: Set the parent of `x` directly to the root.
            // This flattens the tree structure, speeding up future `find` operations.
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
            root
        }
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

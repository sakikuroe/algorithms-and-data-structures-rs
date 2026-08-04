//! 最大流問題・最小費用流問題で共通して使う、残余グラフの土台を提供する
//! モジュールである。
//!
//! [`Graph<T>`](super::graph::Graph) とは別の専用構造を用いる。増加パス探索
//! (Dinic 法・Bellman-Ford 法など) では、ある辺の残余容量を増減させた際に、
//! その逆辺の残余容量も同時に更新する必要がある。そこで、各辺に逆辺への
//! インデックスを直接持たせることで、逆辺を O(1) で参照・更新できるようにする。

use std::ops;

/// フロー問題における容量として利用可能な数値型が満たすべき制約をまとめた
/// trait である。
///
/// 残余容量の初期化に単位元 (`ZERO`) が、増加パスの探索の初期値に十分大きな
/// 上界 (`MAX`) が必要となるため、通常の数値演算に加えてこれらを定数として
/// 要求する。
pub trait FlowCapacity: Copy + Ord + ops::Add<Output = Self> + ops::Sub<Output = Self> {
    /// 加法の単位元 (0)。
    const ZERO: Self;
    /// この型が表現できる最大値。増加パス探索の初期上界として使う。
    const MAX: Self;
}

macro_rules! impl_flow_capacity {
    ($($t:ty),*) => {
        $(
            impl FlowCapacity for $t {
                const ZERO: Self = 0;
                const MAX: Self = <$t>::MAX;
            }
        )*
    };
}
impl_flow_capacity!(i32, i64, u32, u64);

/// 残余グラフ上の1本の有向辺を表す。
pub(super) struct ResidualEdge<Cap> {
    /// 辺の終点。
    pub(super) to: usize,
    /// 残余容量。
    pub(super) cap: Cap,
    /// 逆辺 (終点側の隣接リストに張られている、この辺に対応する辺) の、
    /// 隣接リスト内でのインデックス。
    pub(super) rev: usize,
}

/// 最大流問題・最小費用流問題を表現する有向グラフ。各辺には非負の容量を持たせる。
pub struct FlowGraph<Cap> {
    /// `graph[u]` は、頂点 `u` から出る残余辺の並びである。各辺を追加した際、
    /// 対応する逆辺 (残余容量0で初期化) も終点側に同時に追加される。
    pub(super) graph: Vec<Vec<ResidualEdge<Cap>>>,
    /// `pos[i]` は、`i` 番目に追加した辺の `(始点, 始点の隣接リスト内での添字)`
    /// の組である。追加後の残余容量や流量を、辺番号から引けるようにするために
    /// 使う。
    pos: Vec<(usize, usize)>,
}

impl<Cap> FlowGraph<Cap> {
    /// `n` 頂点、辺を持たないフローグラフを生成する。
    ///
    /// # Args
    /// - `n`: 頂点数。
    ///
    /// # Returns
    /// `Self`: 頂点数 `n`、辺を持たない `FlowGraph<Cap>`。
    ///
    /// # Complexity
    /// - 時間計算量: O(n)
    /// - 空間計算量: O(n)
    #[must_use]
    pub fn new(n: usize) -> Self {
        FlowGraph {
            graph: (0..n).map(|_| Vec::new()).collect::<Vec<_>>(),
            pos: Vec::new(),
        }
    }

    /// グラフの頂点数を返す。
    ///
    /// # Returns
    /// `usize`: 頂点数。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }

    /// これまでに追加した辺の本数を返す。
    ///
    /// # Returns
    /// `usize`: 追加した辺の本数。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn edge_count(&self) -> usize {
        self.pos.len()
    }
}

impl<Cap: FlowCapacity> FlowGraph<Cap> {
    /// 容量 `cap` の有向辺 `from -> to` を追加し、辺番号を返す。
    ///
    /// 内部では、残余容量 `cap` を持つ順辺に加え、残余容量0の逆辺を同時に
    /// 張る。逆辺の残余容量は、流量が押し出されるたびに増加していく。
    ///
    /// # Args
    /// - `from`: 辺の始点。`0..vertex_count()` の範囲でなければならない。
    /// - `to`: 辺の終点。`0..vertex_count()` の範囲でなければならない。
    /// - `cap`: 辺の容量。非負でなければならない。
    ///
    /// # Returns
    /// `usize`: 追加した辺の番号。[`get_edge`](Self::get_edge) で参照する際に
    /// 使う。
    ///
    /// # Panics
    /// - `from`/`to` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::flow_graph::FlowGraph;
    ///
    /// let mut g = FlowGraph::<i64>::new(2);
    /// let id = g.add_edge(0, 1, 5);
    /// assert_eq!((0, 1, 5, 0), g.get_edge(id));
    /// ```
    pub fn add_edge(&mut self, from: usize, to: usize, cap: Cap) -> usize {
        let id = self.edge_count();

        // 自己ループ (from == to) の場合、順辺と逆辺が同じ頂点の隣接リストへ
        // 追加されるため、逆辺のインデックスは「追加前の長さ + 1」になる。
        // 自己ループでない場合は、それぞれ別の隣接リストへの追加になるため、
        // 単純に「追加前の長さ」がそのまま自分の (相手から見た) インデックスになる。
        let from_index = self.graph[from].len();
        let to_index = self.graph[to].len() + usize::from(from == to);

        self.pos.push((from, from_index));
        self.graph[from].push(ResidualEdge {
            to,
            cap,
            rev: to_index,
        });
        self.graph[to].push(ResidualEdge {
            to: from,
            cap: Cap::ZERO,
            rev: from_index,
        });

        id
    }

    /// `i` 番目に追加した辺の `(始点, 終点, 元の容量, 現在の流量)` を返す。
    ///
    /// 元の容量は、順辺の残余容量と逆辺の残余容量の和として求まる。両者の和は
    /// 流量の増減によらず常に元の容量に保たれるからである。また、逆辺の残余
    /// 容量は0から始まり流量が押し出されるたびに増加するため、そのままこの辺の
    /// 現在の流量に等しい。
    ///
    /// # Args
    /// - `i`: 辺番号。[`add_edge`](Self::add_edge) の戻り値。
    ///
    /// # Returns
    /// `(usize, usize, Cap, Cap)`: `(始点, 終点, 元の容量, 現在の流量)`。
    ///
    /// # Panics
    /// - `i` が `0..edge_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_edge(&self, i: usize) -> (usize, usize, Cap, Cap) {
        let (from, index) = self.pos[i];
        let edge = &self.graph[from][index];
        let reverse = &self.graph[edge.to][edge.rev];
        (from, edge.to, edge.cap + reverse.cap, reverse.cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // new のテスト: 生成直後の状態を検証する。
    mod new {
        use super::*;

        /// Scenario: 生成直後のフローグラフは、指定した頂点数を持ち、辺を
        /// 持たない。
        /// - Given: 頂点数 3 を指定する。
        /// - When: `FlowGraph::new(3)` を呼ぶ。
        /// - Then: `vertex_count()` が 3 を返し、`edge_count()` が 0 を返す。
        #[test]
        fn creates_graph_with_given_vertex_count_and_no_edges() {
            // Given, When
            let sut = FlowGraph::<i64>::new(3);
            // Then
            assert_eq!(3, sut.vertex_count());
            assert_eq!(0, sut.edge_count());
        }
    }

    // add_edge のテスト: 呼び出し後の状態変化 (get_edge で観測できる内容) を
    // 検証する。
    mod add_edge {
        use super::*;

        /// Scenario: 辺を追加すると、流量0の状態で観測できる。
        /// - Given: 2頂点、辺を持たないフローグラフがある。
        /// - When: `0 -> 1` に容量5の辺を追加する。
        /// - Then: `get_edge` で `(0, 1, 5, 0)` が観測できる。
        #[test]
        fn records_edge_with_zero_flow() {
            // Given
            let mut sut = FlowGraph::<i64>::new(2);
            // When
            let id = sut.add_edge(0, 1, 5);
            // Then
            assert_eq!((0, 1, 5, 0), sut.get_edge(id));
        }

        /// Scenario: 辺番号は追加した順に0から振られる。
        /// - Given: 3頂点、辺を持たないフローグラフがある。
        /// - When: 辺を2本追加する。
        /// - Then: 辺番号はそれぞれ0, 1になり、`edge_count()` は2になる。
        #[test]
        fn assigns_sequential_ids_in_insertion_order() {
            // Given
            let mut sut = FlowGraph::<i64>::new(3);
            // When
            let id0 = sut.add_edge(0, 1, 1);
            let id1 = sut.add_edge(1, 2, 1);
            // Then
            assert_eq!(0, id0);
            assert_eq!(1, id1);
            assert_eq!(2, sut.edge_count());
        }

        /// Scenario: 自己ループを追加しても、逆辺の対応関係が壊れない
        /// (境界値)。
        /// - Given: 1頂点、辺を持たないフローグラフがある。
        /// - When: `0 -> 0` に容量3の自己ループを追加する。
        /// - Then: `get_edge` で `(0, 0, 3, 0)` が観測できる。
        #[test]
        fn keeps_reverse_edge_consistent_for_self_loop() {
            // Given
            let mut sut = FlowGraph::<i64>::new(1);
            // When
            let id = sut.add_edge(0, 0, 3);
            // Then
            assert_eq!((0, 0, 3, 0), sut.get_edge(id));
        }

        /// Scenario: 同じ2頂点間に複数の辺 (多重辺) を追加しても、それぞれが
        /// 独立に観測できる。
        /// - Given: 2頂点、辺を持たないフローグラフがある。
        /// - When: `0 -> 1` に容量の異なる辺を2本追加する。
        /// - Then: それぞれの辺を個別の辺番号で観測できる。
        #[test]
        fn allows_multi_edge() {
            // Given
            let mut sut = FlowGraph::<i64>::new(2);
            // When
            let id0 = sut.add_edge(0, 1, 2);
            let id1 = sut.add_edge(0, 1, 7);
            // Then
            assert_eq!((0, 1, 2, 0), sut.get_edge(id0));
            assert_eq!((0, 1, 7, 0), sut.get_edge(id1));
        }
    }
}

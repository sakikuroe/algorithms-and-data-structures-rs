//! 最小費用流問題を扱うためのグラフ構造 `MinCostFlowGraph<T>` を提供する
//! モジュールである。
//!
//! [`FlowGraph<Cap>`](super::flow_graph::FlowGraph) と同様に、各辺に逆辺への
//! インデックスを直接持たせることで、逆辺の残余容量を O(1) で参照・更新できる
//! ようにする。最小費用流ではさらに各辺がコストを持ち、逆辺には元の辺の
//! 符号を反転したコストを持たせる。容量とコストは、負の値を取りうるコストの
//! 都合上、同じ数値型 `T` として扱う (競技プログラミングでは両者を `i64` など
//! 単一の整数型で表すことがほとんどであり、型を分けて相互の乗算・変換を
//! 扱う複雑さを負うだけの利点に乏しいための選択である)。

use std::ops;

/// フロー問題における容量・コストとして利用可能な数値型が満たすべき制約を
/// まとめた trait である。
///
/// コストが負の値を取りうるため `Neg` を要求する。また、流量とコストの積を
/// 計算するために `Mul` を要求する。
pub trait FlowValue:
    Copy
    + Ord
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Mul<Output = Self>
    + ops::Neg<Output = Self>
{
    /// 加法の単位元 (0)。
    const ZERO: Self;
    /// この型が表現できる最大値。増加パス探索の初期上界として使う。
    const MAX: Self;
}

macro_rules! impl_flow_value {
    ($($t:ty),*) => {
        $(
            impl FlowValue for $t {
                const ZERO: Self = 0;
                const MAX: Self = <$t>::MAX;
            }
        )*
    };
}
impl_flow_value!(i32, i64);

/// 残余グラフ上の1本の有向辺を表す。
pub(super) struct ResidualEdge<T> {
    /// 辺の終点。
    pub(super) to: usize,
    /// 残余容量。
    pub(super) cap: T,
    /// 単位流量あたりのコスト。逆辺では、元の辺のコストの符号を反転した値に
    /// なる。
    pub(super) cost: T,
    /// 逆辺 (終点側の隣接リストに張られている、この辺に対応する辺) の、
    /// 隣接リスト内でのインデックス。
    pub(super) rev: usize,
}

/// 最小費用流問題を表現する有向グラフ。各辺には非負の容量と、正負を問わない
/// コストを持たせる。
pub struct MinCostFlowGraph<T> {
    /// `graph[u]` は、頂点 `u` から出る残余辺の並びである。各辺を追加した際、
    /// コストの符号を反転した逆辺 (残余容量0で初期化) も終点側に同時に
    /// 追加される。
    pub(super) graph: Vec<Vec<ResidualEdge<T>>>,
    /// `pos[i]` は、`i` 番目に追加した辺の `(始点, 始点の隣接リスト内での添字)`
    /// の組である。追加後の残余容量や流量を、辺番号から引けるようにするために
    /// 使う。
    pos: Vec<(usize, usize)>,
}

impl<T> MinCostFlowGraph<T> {
    /// `n` 頂点、辺を持たない最小費用流グラフを生成する。
    ///
    /// # Args
    /// - `n`: 頂点数
    ///
    /// # Returns
    /// `Self`: 頂点数 `n`、辺を持たない `MinCostFlowGraph<T>`
    ///
    /// # Complexity
    /// - 時間計算量: O(n)
    /// - 空間計算量: O(n)
    #[must_use]
    pub fn new(n: usize) -> Self {
        MinCostFlowGraph {
            graph: (0..n).map(|_| Vec::new()).collect::<Vec<_>>(),
            pos: Vec::new(),
        }
    }

    /// グラフの頂点数を返す。
    ///
    /// # Returns
    /// `usize`: 頂点数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }

    /// これまでに追加した辺の本数を返す。
    ///
    /// # Returns
    /// `usize`: 追加した辺の本数
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn edge_count(&self) -> usize {
        self.pos.len()
    }
}

impl<T: FlowValue> MinCostFlowGraph<T> {
    /// 容量 `cap`、単位流量あたりのコスト `cost` の有向辺 `from -> to` を
    /// 追加し、辺番号を返す。
    ///
    /// # Args
    /// - `from`: 辺の始点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `to`: 辺の終点であり、`0..vertex_count()` の範囲でなければならない。
    /// - `cap`: 辺の容量であり、非負でなければならない。
    /// - `cost`: 辺の単位流量あたりのコストであり、正負は問わない。
    ///
    /// # Returns
    /// `usize`: 追加した辺の番号であり、[`get_edge`](Self::get_edge) で参照する
    /// 際に使う。
    ///
    /// # Panics
    /// - `from`/`to` が `0..vertex_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 O(1)
    ///
    /// # Examples
    /// ```
    /// use anmitsu::graph::min_cost_flow_graph::MinCostFlowGraph;
    ///
    /// let mut g = MinCostFlowGraph::<i64>::new(2);
    /// let id = g.add_edge(0, 1, 5, 3);
    /// assert_eq!((0, 1, 5, 3, 0), g.get_edge(id));
    /// ```
    pub fn add_edge(&mut self, from: usize, to: usize, cap: T, cost: T) -> usize {
        let id = self.edge_count();

        // 自己ループ (from == to) の場合、順辺と逆辺が同じ頂点の隣接リストへ
        // 追加されるため、逆辺のインデックスは「追加前の長さ + 1」になる。
        let from_index = self.graph[from].len();
        let to_index = self.graph[to].len() + usize::from(from == to);

        self.pos.push((from, from_index));
        self.graph[from].push(ResidualEdge {
            to,
            cap,
            cost,
            rev: to_index,
        });
        self.graph[to].push(ResidualEdge {
            to: from,
            cap: T::ZERO,
            cost: -cost,
            rev: from_index,
        });

        id
    }

    /// `i` 番目に追加した辺の `(始点, 終点, 元の容量, コスト, 現在の流量)` を
    /// 返す。
    ///
    /// 元の容量は、順辺の残余容量と逆辺の残余容量の和として求まる。両者の和は
    /// 流量の増減によらず常に元の容量に保たれるからである。また、逆辺の残余
    /// 容量は0から始まり流量が押し出されるたびに増加するため、そのままこの辺の
    /// 現在の流量に等しい。
    ///
    /// # Args
    /// - `i`: 辺番号であり、[`add_edge`](Self::add_edge) の戻り値である。
    ///
    /// # Returns
    /// `(usize, usize, T, T, T)`: `(始点, 終点, 元の容量, コスト, 現在の流量)`
    ///
    /// # Panics
    /// - `i` が `0..edge_count()` の範囲外の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    pub fn get_edge(&self, i: usize) -> (usize, usize, T, T, T) {
        let (from, index) = self.pos[i];
        let edge = &self.graph[from][index];
        let reverse = &self.graph[edge.to][edge.rev];
        (
            from,
            edge.to,
            edge.cap + reverse.cap,
            edge.cost,
            reverse.cap,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // new のテスト: 生成直後の状態を検証する。
    mod new {
        use super::*;

        /// Scenario: 生成直後の最小費用流グラフは、指定した頂点数を持ち、
        /// 辺を持たない。
        /// - Given: 頂点数 3 を指定する。
        /// - When: `MinCostFlowGraph::new(3)` を呼ぶ。
        /// - Then: `vertex_count()` が 3 を返し、`edge_count()` が 0 を返す。
        #[test]
        fn creates_graph_with_given_vertex_count_and_no_edges() {
            // Given, When
            let sut = MinCostFlowGraph::<i64>::new(3);
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
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `0 -> 1` に容量5、コスト3の辺を追加する。
        /// - Then: `get_edge` で `(0, 1, 5, 3, 0)` が観測できる。
        #[test]
        fn records_edge_with_zero_flow() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            // When
            let id = sut.add_edge(0, 1, 5, 3);
            // Then
            assert_eq!((0, 1, 5, 3, 0), sut.get_edge(id));
        }

        /// Scenario: 負のコストを持つ辺も、そのまま観測できる (境界値)。
        /// - Given: 2頂点、辺を持たないグラフがある。
        /// - When: `0 -> 1` に容量2、コスト-4の辺を追加する。
        /// - Then: `get_edge` で `(0, 1, 2, -4, 0)` が観測できる。
        #[test]
        fn allows_negative_cost() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(2);
            // When
            let id = sut.add_edge(0, 1, 2, -4);
            // Then
            assert_eq!((0, 1, 2, -4, 0), sut.get_edge(id));
        }

        /// Scenario: 自己ループを追加しても、逆辺の対応関係が壊れない
        /// (境界値)。
        /// - Given: 1頂点、辺を持たないグラフがある。
        /// - When: `0 -> 0` に容量3、コスト2の自己ループを追加する。
        /// - Then: `get_edge` で `(0, 0, 3, 2, 0)` が観測できる。
        #[test]
        fn keeps_reverse_edge_consistent_for_self_loop() {
            // Given
            let mut sut = MinCostFlowGraph::<i64>::new(1);
            // When
            let id = sut.add_edge(0, 0, 3, 2);
            // Then
            assert_eq!((0, 0, 3, 2, 0), sut.get_edge(id));
        }
    }
}

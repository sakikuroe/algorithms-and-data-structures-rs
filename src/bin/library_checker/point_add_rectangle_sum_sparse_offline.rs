// Library Checker: Point Add Rectangle Sum
// https://judge.yosupo.jp/problem/point_add_rectangle_sum

use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_offline, io::fastio};

/// 点・矩形クエリの読み取りと座標圧縮を行う。
mod input {
    use super::fastio;

    /// 初期点または点加算で扱う座標と重み。
    #[derive(Clone, Copy)]
    pub struct Point {
        /// 行方向に対応する元の座標。
        x: usize,
        /// 列方向に対応する元の座標。
        y: usize,
        /// 点へ加える重み。
        weight: i64,
    }

    /// `point_add_rectangle_sum` のクエリ。
    #[derive(Clone, Copy)]
    pub enum Query {
        /// 点 `(x, y)` へ `weight` を加えるクエリ。
        Add(Point),
        /// 半開矩形 `[left, right) × [bottom, top)` の総和を求めるクエリ。
        Fold {
            /// 矩形の左端。
            left: usize,
            /// 矩形の下端。
            bottom: usize,
            /// 矩形の右端。
            right: usize,
            /// 矩形の上端。
            top: usize,
        },
    }

    /// 入力を座標圧縮した 2 次元セグメント木へ渡すためのデータ。
    pub struct Input {
        /// 入力に現れる x 座標を昇順に重複なく保持する。
        x_coordinates: Vec<usize>,
        /// 入力に現れる y 座標を昇順に重複なく保持する。
        y_coordinates: Vec<usize>,
        /// 入力先頭にある初期点を保持する。
        initial_points: Vec<Point>,
        /// 入力順のクエリを保持する。
        queries: Vec<Query>,
    }

    impl Input {
        /// 元の x 座標を圧縮後の順位へ変換する。
        ///
        /// # Args
        /// - `x` - 圧縮前の x 座標。
        ///
        /// # Returns
        /// `x` 以上の最初の登録座標の順位を返す。
        pub fn x_rank(&self, x: usize) -> usize {
            self.x_coordinates.partition_point(|&value| value < x)
        }

        /// 元の y 座標を圧縮後の順位へ変換する。
        ///
        /// # Args
        /// - `y` - 圧縮前の y 座標。
        ///
        /// # Returns
        /// `y` 以上の最初の登録座標の順位を返す。
        pub fn y_rank(&self, y: usize) -> usize {
            self.y_coordinates.partition_point(|&value| value < y)
        }

        /// 矩形クエリを圧縮後の半開範囲へ変換する。
        ///
        /// # Args
        /// - `query` - 変換対象の矩形クエリ。
        ///
        /// # Returns
        /// 行方向と列方向の圧縮後の半開範囲を返す。
        pub fn rectangle_ranges(&self, query: Query) -> ((usize, usize), (usize, usize)) {
            let Query::Fold {
                left,
                bottom,
                right,
                top,
            } = query
            else {
                unreachable!("rectangle ranges require a fold query")
            };
            (
                (self.x_rank(left), self.x_rank(right)),
                (self.y_rank(bottom), self.y_rank(top)),
            )
        }
    }

    /// `point_add_rectangle_sum` の入力を読み、点と矩形端点を座標圧縮する。
    ///
    /// # Args
    /// - `io` - 問題入力を読む高速入出力オブジェクト。
    ///
    /// # Returns
    /// 初期点、クエリ、全座標の圧縮候補を保持する入力データを返す。
    pub fn read(io: &mut fastio::Fastio) -> Input {
        let n = io.u32() as usize;
        let q = io.u32() as usize;
        let mut x_coordinates = Vec::with_capacity(n + q * 2);
        let mut y_coordinates = Vec::with_capacity(n + q * 2);
        let mut initial_points = Vec::with_capacity(n);
        for _ in 0..n {
            let point = Point {
                x: io.u32() as usize,
                y: io.u32() as usize,
                weight: io.i64(),
            };
            x_coordinates.push(point.x);
            y_coordinates.push(point.y);
            initial_points.push(point);
        }

        let mut queries = Vec::with_capacity(q);
        for _ in 0..q {
            if io.u32() == 0 {
                let point = Point {
                    x: io.u32() as usize,
                    y: io.u32() as usize,
                    weight: io.i64(),
                };
                x_coordinates.push(point.x);
                y_coordinates.push(point.y);
                queries.push(Query::Add(point));
            } else {
                let query = Query::Fold {
                    left: io.u32() as usize,
                    bottom: io.u32() as usize,
                    right: io.u32() as usize,
                    top: io.u32() as usize,
                };
                if let Query::Fold {
                    left,
                    bottom,
                    right,
                    top,
                } = query
                {
                    x_coordinates.extend([left, right]);
                    y_coordinates.extend([bottom, top]);
                }
                queries.push(query);
            }
        }

        x_coordinates.sort_unstable();
        x_coordinates.dedup();
        y_coordinates.sort_unstable();
        y_coordinates.dedup();
        Input {
            x_coordinates,
            y_coordinates,
            initial_points,
            queries,
        }
    }

    /// 圧縮後の x 方向の木の長さを返す。
    ///
    /// # Returns
    /// 入力に現れる x 座標の異なる個数を返す。
    pub fn x_len(input: &Input) -> usize {
        input.x_coordinates.len()
    }

    /// 圧縮後の y 方向の木の長さを返す。
    ///
    /// # Returns
    /// 入力に現れる y 座標の異なる個数を返す。
    pub fn y_len(input: &Input) -> usize {
        input.y_coordinates.len()
    }

    /// 初期点の列を返す。
    ///
    /// # Returns
    /// 入力先頭にある初期点を返す。
    pub fn initial_points(input: &Input) -> &[Point] {
        &input.initial_points
    }

    /// クエリの列を返す。
    ///
    /// # Returns
    /// 入力に現れた順序のクエリを返す。
    pub fn queries(input: &Input) -> &[Query] {
        &input.queries
    }

    /// 点の座標と重みを返す。
    ///
    /// # Args
    /// - `point` - 圧縮前の点。
    ///
    /// # Returns
    /// 点の圧縮前の x、y、重みを返す。
    pub fn point_parts(point: Point) -> (usize, usize, i64) {
        (point.x, point.y, point.weight)
    }
}

/// 疎な offline 2 次元セグメント木で点加算と矩形総和を処理する。
fn main() {
    let mut io = fastio::Fastio::new();
    let input = input::read(&mut io);
    let points = input::initial_points(&input)
        .iter()
        .chain(
            input::queries(&input)
                .iter()
                .filter_map(|query| match query {
                    input::Query::Add(point) => Some(point),
                    input::Query::Fold { .. } => None,
                }),
        )
        .map(|point| {
            let (x, y, _) = input::point_parts(*point);
            (input.x_rank(x), input.y_rank(y))
        });
    let mut seg =
        segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<monoid::AddMonoid>::new(
            input::x_len(&input),
            input::y_len(&input),
            points,
        );
    for &point in input::initial_points(&input) {
        let (x, y, weight) = input::point_parts(point);
        let coordinate = (input.x_rank(x), input.y_rank(y));
        seg.set(coordinate, seg.get(coordinate) + weight);
    }
    seg.build();
    for &query in input::queries(&input) {
        match query {
            input::Query::Add(point) => {
                let (x, y, weight) = input::point_parts(point);
                let coordinate = (input.x_rank(x), input.y_rank(y));
                seg.update(coordinate, seg.get(coordinate) + weight);
            }
            input::Query::Fold { .. } => {
                let ((left, right), (bottom, top)) = input.rectangle_ranges(query);
                io.writeln(seg.fold(left..right, bottom..top));
            }
        }
    }
    io.flush();
}

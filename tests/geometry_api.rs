//! 想定する利用場面から、幾何の公開 API の接続と境界の仕様を検証します。

use anmitsu::geometry::{argument, convex_polygon, direction, predicates, primitives};
use std::{cmp, collections, f64, ops};

/// 幾何ライブラリーから独立した、小規模な参照計算です。
#[path = "bin/geometry_common.rs"]
pub mod reference;

/// 整数座標の点を、公開フィールドだけで構築します。
fn point(x: i64, y: i64) -> primitives::Point2<i64> {
    primitives::Point2 { x, y }
}

/// 整数ベクトルを、公開フィールドだけで構築します。
fn vector(x: i64, y: i64) -> primitives::Vector2<i64> {
    primitives::Vector2 { x, y }
}

/// このテストで非ゼロと分かっているベクトルから方向を構築します。
fn ray(x: i64, y: i64) -> direction::Direction {
    direction::Direction::from_vector(vector(x, y)).expect("fixture direction must be nonzero")
}

/// Background: 重複方向とゼロを含む重み付きの方向列と、その累積和です。
fn create_weighted_order() -> (argument::PolarOrder, Vec<i128>) {
    let vectors = [
        vector(1, 0),
        vector(2, 0),
        vector(0, 1),
        vector(-1, 0),
        vector(0, -1),
        vector(0, 0),
    ];
    let weights = [2_i128, 3, 5, 7, 11, 1000];
    let order =
        argument::PolarOrder::new(vectors.into_iter().map(direction::Direction::from_vector));
    let mut prefix = vec![0];
    for &index in order.indices() {
        prefix.push(prefix.last().unwrap() + weights[index]);
    }
    (order, prefix)
}

/// Scenario: ソートでの原点と、存在しない方向を区別できます。
/// - Given: 原点、正の x 軸、異なる長さの同方向があります。
/// - When: 偏角比較と方向の構築を行います。
/// - Then: 原点は整列上だけ角度 0 となり、方向としては None です。
#[test]
fn distinguishes_zero_sorting_from_direction_construction() {
    // Given
    let zero = vector(0, 0);
    // When
    let comparison = argument::argument_cmp(&zero, &vector(1, 0));
    let missing = direction::Direction::from_vector(zero);
    // Then
    assert_eq!(cmp::Ordering::Equal, comparison);
    assert!(missing.is_none());
    assert_eq!(ray(1, 2), ray(3, 6));
    assert_ne!(ray(1, 2), ray(-1, -2));
}

/// Scenario: 任意の始線からの順序でも、ID と重みを保てます。
/// - Given: 四方向にメタデータを持つ要素があります。
/// - When: 北から反時計回りに整列します。
/// - Then: 北・西・南・東の順で元のメタデータが残ります。
#[test]
fn sorts_payloads_from_an_arbitrary_start() {
    // Given
    let start = ray(0, 1);
    let mut items = [
        (10, ray(1, 0), 2),
        (20, ray(0, -1), 3),
        (30, ray(-1, 0), 5),
        (40, start, 7),
    ];
    // When
    items.sort_by(|a, b| argument::cmp_ccw_from(start, a.1, b.1));
    // Then
    assert_eq!([40, 30, 20, 10], items.map(|item| item.0));
    assert_eq!([7, 5, 3, 2], items.map(|item| item.2));
}

/// Scenario: 折り返す角度区間を、既存の累積和へそのまま渡せます。
/// - Given: 原点に大きな重みを持つ列と、同方向の二要素があります。
/// - When: 北から東までの反時計回りの区間を取得します。
/// - Then: 北・西・南を二つの範囲で集約し、原点の重みを混ぜません。
#[test]
fn connects_wrapped_ranges_to_weighted_prefix_sums() {
    // Given
    let (sut, prefix) = create_weighted_order();
    // When
    let range = sut.ccw_range(ray(0, 1)..ray(1, 0));
    let sum = range
        .split()
        .iter()
        .map(|r| prefix[r.end] - prefix[r.start])
        .sum::<i128>();
    // Then
    assert_eq!(3, range.len());
    assert_eq!(23, sum);
    assert!(range.split().iter().all(|r| !r.is_empty()));
    assert_eq!([5], sut.zero_indices());
    assert_eq!([4, 0, 1, 2, 3], sut.indices());
    assert_eq!(
        [ray(0, -1), ray(1, 0), ray(1, 0), ray(0, 1), ray(-1, 0)],
        sut.directions()
    );
    let mut indices = sut.indices().to_vec();
    indices.sort_unstable();
    assert_eq!([0, 1, 2, 3, 4], indices.as_slice());
}

/// Scenario: 同じ端方向の空・一方向・一周を区別できます。
/// - Given: 東向きが二要素ある方向列があります。
/// - When: 開区間、閉区間、全体、補集合を問い合わせます。
/// - Then: 重複要素を保ち、空と一周を取り違えません。
#[test]
fn distinguishes_empty_ray_and_full_turn() {
    // Given
    let (sut, _) = create_weighted_order();
    let east = ray(1, 0);
    // When
    let empty = sut.ccw_range(east..east);
    let same_ray = sut.ccw_range(east..=east);
    let full = sut.ccw_range(..);
    // Then
    assert!(empty.is_empty());
    assert_eq!(2, same_ray.len());
    assert_eq!(5, full.len());
    assert_eq!(3, same_ray.complement().len());
    assert_eq!(5, empty.complement().len());
    assert!(full.complement().is_empty());
    assert_eq!(5, full.split().iter().map(|r| r.len()).sum::<usize>());
}

/// Scenario: 開始方向と終了方向の包含をそれぞれ指定できます。
/// - Given: 東・北・西の方向に要素があります。
/// - When: 東を除き西を含む区間を取得します。
/// - Then: 北と西だけが残ります。
#[test]
fn selects_endpoint_inclusion_independently() {
    // Given
    let (sut, prefix) = create_weighted_order();
    // When
    let range = sut.ccw_range((
        ops::Bound::Excluded(ray(1, 0)),
        ops::Bound::Included(ray(-1, 0)),
    ));
    let sum = range
        .split()
        .iter()
        .map(|r| prefix[r.end] - prefix[r.start])
        .sum::<i128>();
    // Then
    assert_eq!(2, range.len());
    assert_eq!(12, sum);
}

/// Scenario: ゼロだけの入力にも同じ範囲 API を利用できます。
/// - Given: 幾何学的な方向がない三要素があります。
/// - When: 整列と全体範囲の取得を行います。
/// - Then: ゼロの添字を保持し、非ゼロの範囲は空になります。
#[test]
fn preserves_zero_only_input_indices() {
    // Given
    let sut = argument::PolarOrder::new([None, None, None]);
    // When
    let range = sut.ccw_range(..);
    // Then
    assert!(sut.indices().is_empty());
    assert_eq!([0, 1, 2], sut.zero_indices());
    assert!(range.is_empty());
    assert_eq!(0, sut.lower_bound(ray(1, 0)));
    assert_eq!(0, sut.upper_bound(ray(1, 0)));
    assert!(range.complement().is_empty());
}

/// Scenario: 二点差が i64 を超えても方向と向きを正確に判定できます。
/// - Given: 座標の最小値と最大値を使う三点があります。
/// - When: 二点間の方向、直角、反転、三点の向きを調べます。
/// - Then: 狭い差や積のオーバーフローに依存しません。
#[test]
fn classifies_extreme_coordinates_without_narrow_differences() {
    // Given
    let origin = point(i64::MIN, i64::MIN);
    let east_point = point(i64::MAX, i64::MIN);
    let north_point = point(i64::MIN, i64::MAX);
    // When
    let east = direction::Direction::between(origin, east_point).unwrap();
    let north = direction::Direction::between(origin, north_point).unwrap();
    // Then
    assert_eq!(cmp::Ordering::Equal, east.dot_sign(north));
    assert_eq!(cmp::Ordering::Greater, east.cross_sign(north));
    assert_eq!(cmp::Ordering::Less, east.dot_sign(east.opposite()));
    assert_eq!(
        predicates::Orientation::CounterClockwise,
        predicates::orientation(origin, east_point, north_point)
    );
    assert_eq!(
        predicates::Orientation::Clockwise,
        predicates::orientation(origin, north_point, east_point)
    );
    assert!((east.angle_to(north) - f64::consts::FRAC_PI_2).abs() < 1e-14);
}

/// Scenario: 有向方向を、逆向きも同一とする集約キーへ変換できます。
/// - Given: 正負の整数倍と、別の直線方向があります。
/// - When: 無向方向をハッシュキーとして集約します。
/// - Then: 同じ直線方向の重複を一つにまとめます。
#[test]
fn hashes_unoriented_directions() {
    // Given
    let directions = [ray(2, 4), ray(-1, -2), ray(1, -1)];
    // When
    let keys = directions
        .map(|d| d.unoriented())
        .into_iter()
        .collect::<collections::HashSet<_>>();
    // Then
    assert_eq!(2, keys.len());
    assert_eq!(ray(2, 4).unoriented(), ray(-1, -2).unoriented());
}

/// Scenario: 線分の横断・接触・重なり・非交差を区別できます。
/// - Given: 長さゼロも含む線分対があります。
/// - When: 公開 API で関係を調べます。
/// - Then: 交点の座標を構成せず、必要な関係を取得できます。
#[test]
fn distinguishes_segment_relations_and_points() {
    // Given
    let a = point(0, 0);
    let b = point(4, 0);
    // When / Then
    assert_eq!(
        predicates::SegmentRelation::Crossing,
        predicates::segment_relation(a, b, point(2, -1), point(2, 1))
    );
    assert_eq!(
        predicates::SegmentRelation::Touching,
        predicates::segment_relation(a, b, b, point(4, 3))
    );
    assert_eq!(
        predicates::SegmentRelation::Overlapping,
        predicates::segment_relation(a, b, point(1, 0), point(5, 0))
    );
    assert_eq!(
        predicates::SegmentRelation::Disjoint,
        predicates::segment_relation(a, b, point(5, 0), point(6, 0))
    );
    assert_eq!(
        predicates::SegmentRelation::Touching,
        predicates::segment_relation(a, b, point(2, 0), point(2, 0))
    );
    assert!(predicates::on_segment(point(2, 0), a, b));
    assert!(!predicates::on_segment(point(5, 0), a, b));
}

/// Scenario: 凸包の元添字と辺上の共線点の方針を選べます。
/// - Given: 同一座標と辺上の点を含む点群があります。
/// - When: 両端だけの凸包と、境界を全て残す凸包を求めます。
/// - Then: 重複の代表添字と境界の選択が一貫します。
#[test]
fn preserves_hull_identity_and_collinear_policy() {
    // Given
    let points = [
        point(0, 0),
        point(1, 0),
        point(2, 0),
        point(2, 2),
        point(0, 2),
        point(0, 0),
    ];
    // When
    let endpoints = convex_polygon::convex_hull_indices(
        &points,
        convex_polygon::CollinearPoints::EndpointsOnly,
    );
    let boundary =
        convex_polygon::convex_hull_indices(&points, convex_polygon::CollinearPoints::AllBoundary);
    // Then
    assert_eq!([0, 2, 3, 4], endpoints.as_slice());
    assert_eq!([0, 1, 2, 3, 4], boundary.as_slice());
}

/// Scenario: 凸な境界の正規化で、呼び出し側の向きや重複処理を不要にできます。
/// - Given: 時計回り、閉じる頂点の再掲、辺上の点を含む境界があります。
/// - When: 境界から凸多角形を構築します。
/// - Then: 狭義凸で反時計回りの頂点列と面積が得られます。
#[test]
fn normalizes_clockwise_closed_boundaries() {
    // Given
    let boundary = [
        point(0, 0),
        point(0, 2),
        point(2, 2),
        point(2, 0),
        point(1, 0),
        point(0, 0),
    ];
    // When
    let sut = convex_polygon::ConvexPolygon::try_from_boundary(&boundary).unwrap();
    // Then
    assert_eq!(
        [point(0, 0), point(2, 0), point(2, 2), point(0, 2)],
        sut.vertices()
    );
    assert_eq!(8, sut.area2());
    assert_eq!(predicates::Location::Inside, sut.locate(point(1, 1)));
    assert_eq!(predicates::Location::Boundary, sut.locate(point(1, 0)));
    assert_eq!(predicates::Location::Outside, sut.locate(point(3, 0)));
}

/// Scenario: 局所的な向きだけでは検出できない自己交差を拒否します。
/// - Given: 五芒星の順で並べた頂点があります。
/// - When: 凸な境界として構築しようとします。
/// - Then: 誤って凸多角形として受理しません。
#[test]
fn rejects_self_crossing_boundaries() {
    // Given
    let boundary = [
        point(0, 3),
        point(2, -3),
        point(-3, 1),
        point(3, 1),
        point(-2, -3),
    ];
    // When
    let result = convex_polygon::ConvexPolygon::try_from_boundary(&boundary);
    // Then
    assert!(result.is_err());
}

/// Scenario: 支持面を頂点または辺として取得し、最小側も逆方向で求められます。
/// - Given: 正方形と、辺または頂点を支える方向があります。
/// - When: 最大と最小の支持面を問い合わせます。
/// - Then: 同値な頂点を走査し直さず、支持する面を取得できます。
#[test]
fn exposes_support_edges_and_vertices() {
    // Given
    let sut = convex_polygon::ConvexPolygon::from_points(&[
        point(0, 0),
        point(2, 0),
        point(2, 2),
        point(0, 2),
    ]);
    // When / Then
    for (normal, expected_x) in [(ray(1, 0), 2), (ray(-1, 0), 0)] {
        match sut.support(normal).unwrap() {
            convex_polygon::SupportFace::Edge { start, end } => {
                assert_eq!(expected_x, sut.vertices()[start].x);
                assert_eq!(expected_x, sut.vertices()[end].x);
            }
            convex_polygon::SupportFace::Vertex(_) => panic!("support must be an edge"),
        }
    }
    match sut.support(ray(1, 1)).unwrap() {
        convex_polygon::SupportFace::Vertex(index) => {
            assert_eq!(point(2, 2), sut.vertices()[index])
        }
        convex_polygon::SupportFace::Edge { .. } => panic!("support must be a vertex"),
    }
}

/// Scenario: Minkowski 和を平行移動による衝突領域に接続できます。
/// - Given: 二つの長方形があります。
/// - When: 一方を反転した図形との和を構築し、移動量を内外判定します。
/// - Then: 重なり・接触・非交差が内部・境界・外部に対応します。
#[test]
fn composes_minkowski_sum_with_collision_queries() {
    // Given
    let a = convex_polygon::ConvexPolygon::from_points(&[
        point(0, 0),
        point(3, 0),
        point(3, 2),
        point(0, 2),
    ]);
    let negative_b = convex_polygon::ConvexPolygon::from_points(&[
        point(0, 0),
        point(-1, 0),
        point(-1, -1),
        point(0, -1),
    ]);
    // When
    let collision = a.minkowski_sum(&negative_b);
    // Then
    assert_eq!(predicates::Location::Inside, collision.locate(point(1, 1)));
    assert_eq!(
        predicates::Location::Boundary,
        collision.locate(point(3, 0))
    );
    assert_eq!(predicates::Location::Outside, collision.locate(point(4, 0)));
    assert_eq!(24, collision.area2());
}

/// Scenario: 和の構築を、全頂点対の和の凸包と照合できます。
/// - Given: 小さな点群から作った二つの凸多角形があります。
/// - When: 公開 API で Minkowski 和を構築します。
/// - Then: 独立した Gift wrapping の頂点集合と一致します。
#[test]
fn compares_minkowski_sums_with_pairwise_point_sums() {
    // Given
    for seed in 0..20 {
        let first = reference::create_points(7, seed);
        let second = reference::create_points(6, seed + 100);
        let a = convex_polygon::ConvexPolygon::from_points(
            &first.iter().map(|&(x, y)| point(x, y)).collect::<Vec<_>>(),
        );
        let b = convex_polygon::ConvexPolygon::from_points(
            &second.iter().map(|&(x, y)| point(x, y)).collect::<Vec<_>>(),
        );
        let sums = first
            .iter()
            .flat_map(|&(x, y)| second.iter().map(move |&(u, v)| (x + u, y + v)))
            .collect::<Vec<_>>();
        let mut expected = reference::reference_hull(&sums);
        expected.sort_unstable();
        // When
        let sum = a.minkowski_sum(&b);
        let mut result = sum
            .vertices()
            .iter()
            .map(|p| (p.x, p.y))
            .collect::<Vec<_>>();
        result.sort_unstable();
        // Then
        assert_eq!(expected, result);
    }
}

/// Scenario: 空・一点・線分でも同じ凸多角形 API を利用できます。
/// - Given: 退化した凸包があります。
/// - When: 内外判定、直径、支持面、Minkowski 和を問い合わせます。
/// - Then: 不要な例外処理を呼び出し側に要求しません。
#[test]
fn handles_degenerate_convex_polygons_consistently() {
    // Given
    let empty = convex_polygon::ConvexPolygon::from_points(&[]);
    let singleton = convex_polygon::ConvexPolygon::from_points(&[point(2, 3), point(2, 3)]);
    let segment = convex_polygon::ConvexPolygon::from_points(&[point(0, 0), point(4, 0)]);
    // When / Then
    assert_eq!(predicates::Location::Outside, empty.locate(point(0, 0)));
    assert!(empty.diameter().is_none());
    assert!(empty.support(ray(1, 0)).is_none());
    assert!(empty.minkowski_sum(&segment).vertices().is_empty());
    assert_eq!(Some((0, 0)), singleton.diameter());
    assert_eq!(
        predicates::Location::Boundary,
        singleton.locate(point(2, 3))
    );
    assert_eq!(Some((0, 1)), segment.diameter());
    assert_eq!(predicates::Location::Boundary, segment.locate(point(2, 0)));
    assert_eq!(predicates::Location::Outside, segment.locate(point(5, 0)));
    assert_eq!(0, segment.area2());
    assert!(convex_polygon::furthest_pair(&[point(2, 3)]).is_none());
    let pair = convex_polygon::furthest_pair(&[point(2, 3), point(2, 3)]).unwrap();
    assert_ne!(pair.0, pair.1);
}

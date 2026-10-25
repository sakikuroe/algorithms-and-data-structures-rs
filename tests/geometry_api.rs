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
    assert_eq!([1..5, 0..1], empty.complement().split());
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
    assert!(sut.bracketing_positions(ray(1, 0)).is_none());
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
    assert_eq!(0.0, east.angle_to(east));
    assert_eq!(f64::consts::PI, east.angle_to(east.opposite()));
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
    assert_eq!(
        2,
        [ray(1, 2), ray(3, 6), ray(-1, -2)]
            .into_iter()
            .collect::<collections::HashSet<_>>()
            .len()
    );
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
    assert!(predicates::on_segment(a, a, b));
    assert!(predicates::on_segment(a, a, a));
    assert!(!predicates::on_segment(b, a, a));
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
                assert_eq!((start + 1) % sut.vertices().len(), end);
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
/// - When: 内外判定、最遠点対、支持面、Minkowski 和を問い合わせます。
/// - Then: 不要な例外処理を呼び出し側に要求しません。
#[test]
fn handles_degenerate_convex_polygons_consistently() {
    // Given
    let empty = convex_polygon::ConvexPolygon::from_points(&[]);
    let singleton = convex_polygon::ConvexPolygon::from_points(&[point(2, 3), point(2, 3)]);
    let segment = convex_polygon::ConvexPolygon::from_points(&[point(0, 0), point(4, 0)]);
    // When / Then
    assert_eq!(predicates::Location::Outside, empty.locate(point(0, 0)));
    assert!(empty.furthest_pair().is_none());
    assert!(empty.support(ray(1, 0)).is_none());
    assert!(empty.minkowski_sum(&segment).vertices().is_empty());
    assert!(singleton.furthest_pair().is_none());
    assert_eq!(
        predicates::Location::Boundary,
        singleton.locate(point(2, 3))
    );
    assert_eq!(Some((0, 1)), segment.furthest_pair());
    assert_eq!(predicates::Location::Boundary, segment.locate(point(2, 0)));
    assert_eq!(predicates::Location::Outside, segment.locate(point(5, 0)));
    assert_eq!(0, segment.area2());
    assert_eq!(
        [point(2, 3), point(6, 3)],
        singleton.minkowski_sum(&segment).vertices()
    );
    assert!(convex_polygon::furthest_pair(&[]).is_none());
    assert!(convex_polygon::furthest_pair(&[point(2, 3)]).is_none());
    let pair = convex_polygon::furthest_pair(&[point(2, 3), point(2, 3)]).unwrap();
    assert_eq!((0, 1), pair);
}

// 点・ベクトルの値と演算子を、利用側で組み合わせるテストです。
mod primitive_operations {
    use super::*;

    /// Scenario: 平行移動・差・内積・外積をそのまま組み合わせられます。
    /// - Given: 二点と、加算・整数倍を行うベクトルがあります。
    /// - When: 値の演算と代入演算を組み合わせます。
    /// - Then: 点とベクトルの結果型が保たれ、長さと距離を取得できます。
    #[test]
    fn composes_translation_vector_arithmetic_and_products() {
        // Given
        let origin = point(3, 5);
        let step = vector(2, -1);
        let v = vector(2, -3);
        let w = vector(-1, 5);
        // When
        let moved = origin + step;
        let mut cursor = origin;
        cursor += step;
        cursor -= step;
        let mut accumulated = v;
        accumulated += w;
        accumulated -= w;
        accumulated *= 3;
        // Then
        assert_eq!(point(5, 4), moved);
        assert_eq!(origin, moved - step);
        assert_eq!(step, moved - origin);
        assert_eq!(origin, cursor);
        assert_eq!(vector(1, 2), v + w);
        assert_eq!(vector(3, -8), v - w);
        assert_eq!(vector(-2, 3), -v);
        assert_eq!(vector(6, -9), v * 3);
        assert_eq!(v * 3, accumulated);
        assert_eq!(-17, v.dot(w));
        assert_eq!(7, v.cross(w));
        assert_eq!(cmp::Ordering::Less, v.dot_sign(w));
        assert_eq!(cmp::Ordering::Greater, v.cross_sign(w));
        assert_eq!(13, v.norm_squared());
        assert_eq!(5, origin.distance_squared(moved));
    }

    /// Scenario: 正負と大きな回転数にも同じ向きの規則を使えます。
    /// - Given: 非ゼロのベクトルと回転数があります。
    /// - When: ベクトルと方向を 90 度単位で回転します。
    /// - Then: 反時計回りを正とし、4 を法として結果が一致します。
    #[rstest::rstest]
    #[case::zero(0, (2, -3))]
    #[case::counterclockwise(1, (3, 2))]
    #[case::clockwise(-1, (-3, -2))]
    #[case::half_turn(2, (-2, 3))]
    #[case::more_than_one_turn(5, (3, 2))]
    #[case::minimum_turns(i32::MIN, (2, -3))]
    #[case::maximum_turns(i32::MAX, (-3, -2))]
    fn rotates_signed_quarters(#[case] turns: i32, #[case] expected: (i64, i64)) {
        // Given
        let sut = vector(2, -3);
        // When
        let result = sut.rotate_quarter_turns(turns);
        // Then
        assert_eq!(vector(expected.0, expected.1), result);
        assert_eq!(
            ray(expected.0, expected.1),
            ray(2, -3).rotate_quarter_turns(turns)
        );
    }

    /// Scenario: ゼロや広い座標を、狭い差や方向への変換なしで扱えます。
    /// - Given: ゼロベクトルと最小座標のベクトル、両端の点があります。
    /// - When: 符号・長さ・二点間距離を取得します。
    /// - Then: 値を表現できる演算は成功し、符号は値取得の上限に依存しません。
    #[test]
    fn separates_exact_signs_and_wide_distances_from_narrow_arithmetic() {
        // Given
        let zero = vector(0, 0);
        let extreme = vector(i64::MIN, i64::MIN);
        let left = point(i64::MIN, 0);
        let right = point(i64::MAX, 0);
        // When / Then
        assert_eq!(0, zero.dot(extreme));
        assert_eq!(0, zero.cross(extreme));
        assert_eq!(cmp::Ordering::Equal, zero.dot_sign(extreme));
        assert_eq!(cmp::Ordering::Equal, zero.cross_sign(extreme));
        assert_eq!(cmp::Ordering::Greater, extreme.dot_sign(extreme));
        assert_eq!(0, extreme.dot(vector(1, -1)));
        assert_eq!(1_u128 << 127, extreme.norm_squared());
        assert_eq!(u128::from(u64::MAX).pow(2), left.distance_squared(right));
        assert_eq!(extreme, extreme.rotate_quarter_turns(0));
        assert_eq!(zero, zero.rotate_quarter_turns(-1));
    }

    /// Scenario: 座標の同一性を標準の集合のキーに使えます。
    /// - Given: 同じ座標と正の整数倍の座標があります。
    /// - When: 点とベクトルをそれぞれハッシュ集合へ入れます。
    /// - Then: 同じ座標だけをまとめ、同方向でも別の座標は残します。
    #[test]
    fn hashes_points_and_vectors_by_coordinates() {
        // Given
        let points = [point(1, 2), point(1, 2), point(2, 4)];
        let vectors = [vector(1, 2), vector(1, 2), vector(2, 4)];
        // When
        let point_keys = points.into_iter().collect::<collections::HashSet<_>>();
        let vector_keys = vectors.into_iter().collect::<collections::HashSet<_>>();
        // Then
        assert_eq!(2, point_keys.len());
        assert_eq!(2, vector_keys.len());
    }

    /// Scenario: 値を表現できない演算を丸めずに検出します。
    /// - Given: 座標または戻り値の範囲を超える演算があります。
    /// - When: その値を取得します。
    /// - Then: debug・release のいずれでもパニックします。
    #[rstest::rstest]
    #[case::point_translation(|| { let _ = point(i64::MAX, 0) + vector(1, 0); })]
    #[case::point_difference(|| { let _ = point(i64::MAX, 0) - point(i64::MIN, 0); })]
    #[case::vector_sum(|| { let _ = vector(i64::MAX, 0) + vector(1, 0); })]
    #[case::vector_negation(|| { let _ = -vector(i64::MIN, 0); })]
    #[case::scalar_multiplication(|| { let _ = vector(i64::MAX, 0) * 2; })]
    #[case::quarter_turn(|| { let _ = vector(0, i64::MIN).rotate_quarter_turns(1); })]
    #[case::dot_value(|| { let _ = vector(i64::MIN, i64::MIN).dot(vector(i64::MIN, i64::MIN)); })]
    #[case::distance_value(|| { let _ = point(i64::MIN, i64::MIN).distance_squared(point(i64::MAX, i64::MAX)); })]
    #[should_panic]
    fn rejects_unrepresentable_values(#[case] operation: fn()) {
        // Given
        // When
        operation();
        // Then: パニックを属性で検証します。
    }
}

// 円周上の検索と、通常の列への接続を検証するテストです。
mod circular_queries {
    use super::*;

    /// Scenario: 同方向の重複と切れ目を含め、円周の前後を取得できます。
    /// - Given: 東向きが二つあり、原点も含む方向列があります。
    /// - When: 指定方向から時計回り側・反時計回り側の位置を取得します。
    /// - Then: 同方向も含め、列の端を越える場合は折り返します。
    #[rstest::rstest]
    #[case::duplicate_hit((1, 0), [2, 1])]
    #[case::between_east_and_north((1, 1), [2, 3])]
    #[case::wraps_cut((-1, -1), [4, 0])]
    #[case::single_hit((0, 1), [3, 3])]
    #[case::between_north_and_west((-1, 1), [3, 4])]
    #[case::between_south_and_east((1, -1), [0, 1])]
    fn brackets_positions_inclusively(#[case] query: (i64, i64), #[case] expected: [usize; 2]) {
        // Given
        let (sut, _) = create_weighted_order();
        // When
        let result = sut.bracketing_positions(ray(query.0, query.1));
        // Then
        assert_eq!(Some(expected), result);
    }

    /// Scenario: 一方向しかない列では同じ位置を前後の候補にできます。
    /// - Given: 方向のある一要素とゼロがあります。
    /// - When: 一致する方向と別の方向から前後を検索します。
    /// - Then: どちらも非ゼロの整列位置 0 を二回返します。
    #[test]
    fn brackets_singletons_without_client_side_wrapping() {
        // Given
        let sut = argument::PolarOrder::new([None, Some(ray(3, 4))]);
        // When / Then
        assert_eq!([1], sut.indices());
        assert_eq!(Some([0, 0]), sut.bracketing_positions(ray(3, 4)));
        assert_eq!(Some([0, 0]), sut.bracketing_positions(ray(-3, -4)));
    }

    /// Scenario: 全要素を含む区間も指定した始方向から走査できます。
    /// - Given: 二つの端方向の間の狭い角度に要素がありません。
    /// - When: そこから反時計回りに全要素を取得します。
    /// - Then: 絶対偏角の先頭に戻さず、補集合を二回取っても順序を保ちます。
    #[test]
    fn preserves_start_position_when_a_query_covers_all_elements() {
        // Given
        let (sut, _) = create_weighted_order();
        // When
        let range = sut.ccw_range(ray(1, 1)..ray(2, 1));
        let ids = range
            .split()
            .into_iter()
            .flat_map(|r| r.map(|i| sut.indices()[i]))
            .collect::<Vec<_>>();
        // Then
        assert_eq!(5, range.len());
        assert_eq!([2, 3, 4, 0, 1], ids.as_slice());
        assert_eq!([3..5, 0..3], range.split());
        assert!(range.complement().is_empty());
        assert_eq!(range.split(), range.complement().complement().split());
        assert_eq!([0..5, 0..0], sut.ccw_range(..).split());
    }

    /// Scenario: 角度区間の全端点指定を、独立した所属判定と照合できます。
    /// - Given: 八方向、同方向の重複、ゼロを含む列があります。
    /// - When: 全ての始方向・終方向と開閉端点を組み合わせます。
    /// - Then: 元添字の列挙順・個数・補集合が直接判定と一致します。
    #[test]
    fn compares_all_endpoint_combinations_and_complements() {
        // Given
        let angles = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
        ];
        let rays = angles.map(|(x, y)| ray(x, y));
        let sut =
            argument::PolarOrder::new(rays.into_iter().map(Some).chain([Some(ray(2, 0)), None]));
        let entries = [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (8, 3),
            (4, 4),
            (5, 5),
            (6, 6),
            (7, 7),
        ];
        for start in 0..8 {
            for end in 0..8 {
                for include_start in [false, true] {
                    for include_end in [false, true] {
                        let mut expected = entries
                            .into_iter()
                            .filter(|&(_, angle)| {
                                if start == end {
                                    return include_start && include_end && angle == start;
                                }
                                if angle == start {
                                    return include_start;
                                }
                                if angle == end {
                                    return include_end;
                                }
                                if start < end {
                                    start < angle && angle < end
                                } else {
                                    start < angle || angle < end
                                }
                            })
                            .collect::<Vec<_>>();
                        expected.sort_by_key(|&(id, angle)| ((angle + 8 - start) % 8, id));
                        let expected = expected.into_iter().map(|(id, _)| id).collect::<Vec<_>>();
                        let lower = if include_start {
                            ops::Bound::Included(rays[start])
                        } else {
                            ops::Bound::Excluded(rays[start])
                        };
                        let upper = if include_end {
                            ops::Bound::Included(rays[end])
                        } else {
                            ops::Bound::Excluded(rays[end])
                        };
                        // When
                        let range = sut.ccw_range((lower, upper));
                        let result = range
                            .split()
                            .into_iter()
                            .flat_map(|r| r.map(|i| sut.indices()[i]))
                            .collect::<Vec<_>>();
                        let complement = range.complement();
                        let mut combined = result.clone();
                        combined.extend(
                            complement
                                .split()
                                .into_iter()
                                .flat_map(|r| r.map(|i| sut.indices()[i])),
                        );
                        combined.sort_unstable();
                        // Then
                        assert_eq!(expected, result);
                        assert_eq!(expected.len(), range.len());
                        assert_eq!(9 - expected.len(), complement.len());
                        assert_eq!((0..9).collect::<Vec<_>>(), combined);
                        assert_eq!(range.split(), complement.complement().split());
                    }
                }
            }
        }
        // When / Then
        let east = ray(1, 0);
        assert_eq!(3, sut.ccw_range(..east).len());
        assert_eq!(5, sut.ccw_range(..=east).len());
        assert_eq!(6, sut.ccw_range(east..).len());
        assert_eq!(
            4,
            sut.ccw_range((ops::Bound::Excluded(east), ops::Bound::Unbounded))
                .len()
        );
        assert_eq!(3, sut.directions().partition_point(|&d| d < east));
        assert_eq!(5, sut.directions().partition_point(|&d| d <= east));
    }
}

// 比較と凸多角形の、別の利用場面でも必要になる契約を検証します。
mod ordering_and_polygons {
    use super::*;

    /// Scenario: 原点を含む偏角比較にも一貫した順序が定義されています。
    /// - Given: 小さい格子上の原点と各方向のベクトルがあります。
    /// - When: 全組合せの逆順比較と推移関係を確認します。
    /// - Then: 原点を正の x 軸と同順位に置いても比較の規則を満たします。
    #[test]
    fn preserves_comparator_equivalence_and_transitivity_with_zero() {
        // Given
        let vectors = (-2..=2)
            .flat_map(|x| (-2..=2).map(move |y| vector(x, y)))
            .collect::<Vec<_>>();
        for a in &vectors {
            for b in &vectors {
                // When
                let ab = argument::argument_cmp(a, b);
                let ba = argument::argument_cmp(b, a);
                // Then
                assert_eq!(ab, ba.reverse());
                for c in &vectors {
                    let bc = argument::argument_cmp(b, c);
                    if ab != cmp::Ordering::Greater && bc != cmp::Ordering::Greater {
                        assert_ne!(cmp::Ordering::Greater, argument::argument_cmp(a, c));
                    }
                }
            }
        }
    }

    /// Scenario: 二次元の境界では逆走・重複・内点・複数周を拒否します。
    /// - Given: 局所的な判定だけでは不足する境界候補があります。
    /// - When: 境界から凸多角形を構築します。
    /// - Then: 境界全体の巡回順に反する列を受け付けません。
    #[rstest::rstest]
    #[case::interior_vertex(&[(0, 0), (3, 0), (1, 1), (3, 3), (0, 3)])]
    #[case::backtracks_on_edge(&[(0, 0), (2, 0), (1, 0), (3, 0), (3, 3), (0, 3)])]
    #[case::repeated_nonadjacent_vertex(&[(0, 0), (2, 0), (2, 2), (0, 0), (0, 2)])]
    #[case::multiple_turns(&[(0, 0), (2, 0), (2, 2), (0, 2), (0, 0), (2, 0), (2, 2), (0, 2)])]
    fn rejects_invalid_boundary_traversals(#[case] coordinates: &[(i64, i64)]) {
        // Given
        let boundary = coordinates
            .iter()
            .map(|&(x, y)| point(x, y))
            .collect::<Vec<_>>();
        // When
        let result = convex_polygon::ConvexPolygon::try_from_boundary(&boundary);
        // Then
        assert!(result.is_err());
    }

    /// Scenario: 一次元に退化した境界は順序によらず両端へ正規化できます。
    /// - Given: 同一直線上に戻りや重複を含む点列があります。
    /// - When: 境界として構築します。
    /// - Then: 二つの端点と、全体を支える面が得られます。
    #[test]
    fn normalizes_collinear_boundaries_and_support_faces() {
        // Given
        let boundary = [
            point(2, 0),
            point(0, 0),
            point(1, 0),
            point(2, 0),
            point(0, 0),
        ];
        // When
        let sut = convex_polygon::ConvexPolygon::try_from_boundary(&boundary).unwrap();
        // Then
        assert_eq!([point(0, 0), point(2, 0)], sut.vertices());
        assert!(
            convex_polygon::ConvexPolygon::try_from_boundary(&[])
                .unwrap()
                .vertices()
                .is_empty()
        );
        assert_eq!(
            [point(4, 5)],
            convex_polygon::ConvexPolygon::try_from_boundary(&[point(4, 5); 3])
                .unwrap()
                .vertices()
        );
        assert_eq!(
            Some(convex_polygon::SupportFace::Edge { start: 0, end: 1 }),
            sut.support(ray(0, 1))
        );
        assert_eq!(
            Some(convex_polygon::SupportFace::Vertex(0)),
            sut.support(ray(-1, 0))
        );
        assert_eq!(
            Some(convex_polygon::SupportFace::Vertex(1)),
            sut.support(ray(1, 0))
        );
    }

    /// Scenario: 正規化した多角形の最遠点対を、距離の取得へ接続できます。
    /// - Given: 小さい点群から構築した凸多角形があります。
    /// - When: 頂点の最遠点対とその二乗距離を取得します。
    /// - Then: 異なる二添字を小さい順で返し、全点対の最大距離と一致します。
    #[test]
    fn connects_polygon_furthest_pairs_to_squared_distances() {
        // Given
        for seed in 0..24 {
            let points = reference::create_points(9, seed);
            let sut = convex_polygon::ConvexPolygon::from_points(
                &points.iter().map(|&(x, y)| point(x, y)).collect::<Vec<_>>(),
            );
            let expected = points
                .iter()
                .flat_map(|&a| {
                    points
                        .iter()
                        .map(move |&b| reference::distance_squared(a, b))
                })
                .max()
                .unwrap() as u128;
            // When
            let (first, second) = sut.furthest_pair().unwrap();
            let distance = sut.vertices()[first].distance_squared(sut.vertices()[second]);
            // Then
            assert!(first < second);
            assert_eq!(expected, distance);
        }
    }
}

// 値の取得で表現できる範囲を超えても、判定できることを確認します。
mod full_coordinate_range {
    use super::*;

    /// Scenario: 巨大な積が打ち消し合う二方向も厳密に区別できます。
    /// - Given: 差が u64::MAX 付近で、外積が -1 となる三点があります。
    /// - When: 三点の向きと方向の符号を判定します。
    /// - Then: 個別の積が i128 を超えても丸めず、ゼロとも誤認しません。
    #[test]
    fn compares_nearly_parallel_wide_directions_exactly() {
        // Given
        let origin = point(i64::MIN, i64::MIN);
        let a = point(i64::MAX, i64::MAX - 1);
        let b = point(i64::MAX - 1, i64::MAX - 2);
        // When
        let first = direction::Direction::between(origin, a).unwrap();
        let second = direction::Direction::between(origin, b).unwrap();
        // Then
        assert_eq!(cmp::Ordering::Less, first.cross_sign(second));
        assert_eq!(cmp::Ordering::Greater, first.dot_sign(second));
        assert_eq!(cmp::Ordering::Greater, first.dot_sign(first));
        assert_eq!(
            predicates::Orientation::Clockwise,
            predicates::orientation(origin, a, b)
        );
        assert!(direction::Direction::between(origin, origin).is_none());
    }

    /// Scenario: 面積や距離の値を取得できない大きな図形も問い合わせられます。
    /// - Given: 全座標域に広がる正方形があります。
    /// - When: 構築後に内外判定と、点群・多角形の最遠点対を求めます。
    /// - Then: 面積や二乗距離を必須の中間値にせず、正しい頂点を選びます。
    #[test]
    fn queries_giant_polygons_without_requesting_overflowing_values() {
        // Given
        let points = [
            point(i64::MIN, i64::MIN),
            point(i64::MAX, i64::MAX),
            point(i64::MIN, i64::MAX),
            point(i64::MAX, i64::MIN),
        ];
        // When
        let sut = convex_polygon::ConvexPolygon::from_points(&points);
        let original_pair = convex_polygon::furthest_pair(&points).unwrap();
        let vertex_pair = sut.furthest_pair().unwrap();
        // Then
        assert_eq!(predicates::Location::Inside, sut.locate(point(0, 0)));
        assert_eq!(predicates::Location::Boundary, sut.locate(points[0]));
        assert!([(0, 1), (2, 3)].contains(&original_pair));
        assert!([(0, 2), (1, 3)].contains(&vertex_pair));
        let wide_normal =
            direction::Direction::between(points[0], point(i64::MAX, i64::MAX - 1)).unwrap();
        assert_eq!(
            Some(convex_polygon::SupportFace::Vertex(2)),
            sut.support(wide_normal)
        );
        let inset = convex_polygon::ConvexPolygon::from_points(&[
            point(i64::MIN + 1, i64::MIN + 1),
            point(i64::MAX, i64::MIN + 1),
            point(i64::MAX, i64::MAX),
            point(i64::MIN + 1, i64::MAX),
        ]);
        let negative_unit = convex_polygon::ConvexPolygon::from_points(&[
            point(-1, -1),
            point(0, -1),
            point(0, 0),
            point(-1, 0),
        ]);
        assert_eq!(
            sut.vertices(),
            inset.minkowski_sum(&negative_unit).vertices()
        );
    }

    /// Scenario: 図形の構成や値取得の上限は判定操作と分けて検出します。
    /// - Given: 面積または生成する頂点が返却型の範囲を超えます。
    /// - When: その値を要求します。
    /// - Then: debug・release のいずれでもパニックします。
    #[rstest::rstest]
    #[case::area_value(|| {
        let polygon = convex_polygon::ConvexPolygon::from_points(&[
            point(i64::MIN, i64::MIN), point(i64::MAX, i64::MIN),
            point(i64::MAX, i64::MAX), point(i64::MIN, i64::MAX),
        ]);
        let _ = polygon.area2();
    })]
    #[case::minkowski_coordinates(|| {
        let a = convex_polygon::ConvexPolygon::from_points(&[point(i64::MAX, 0)]);
        let b = convex_polygon::ConvexPolygon::from_points(&[point(1, 0)]);
        let _ = a.minkowski_sum(&b);
    })]
    #[should_panic]
    fn rejects_unrepresentable_polygon_values(#[case] operation: fn()) {
        // Given
        // When
        operation();
        // Then: パニックを属性で検証します。
    }
}

/// Scenario: 空・全点重複・共線でも元添字と境界点の方針が一貫します。
/// - Given: 二次元の面積を持たない点群があります。
/// - When: 両端だけの凸包と、全ての境界点を残す凸包を求めます。
/// - Then: 座標の辞書順と最小の元添字を保ち、同一点を重ねません。
#[rstest::rstest]
#[case::empty(&[], &[], &[])]
#[case::identical(&[(4, 4), (4, 4)], &[0], &[0])]
#[case::collinear(&[(2, 0), (-1, 0), (1, 0), (-1, 0), (0, 0)], &[1, 0], &[1, 4, 2, 0])]
fn preserves_degenerate_hull_indices(
    #[case] coordinates: &[(i64, i64)],
    #[case] endpoints: &[usize],
    #[case] boundary: &[usize],
) {
    // Given
    let points = coordinates
        .iter()
        .map(|&(x, y)| point(x, y))
        .collect::<Vec<_>>();
    // When
    let endpoints_result = convex_polygon::convex_hull_indices(
        &points,
        convex_polygon::CollinearPoints::EndpointsOnly,
    );
    let boundary_result =
        convex_polygon::convex_hull_indices(&points, convex_polygon::CollinearPoints::AllBoundary);
    // Then
    assert_eq!(endpoints, endpoints_result);
    assert_eq!(boundary, boundary_result);
}

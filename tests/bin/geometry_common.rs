//! 幾何の解答を検証する、ライブラリーから独立した小規模な参照計算です。

use std::{cmp, collections};

/// テスト用の二次元整数ベクトルの外積を計算します。
/// 座標は絶対値 10^9 以下なので、参照計算の値は i128 に収まります。
pub fn det(a: (i64, i64), b: (i64, i64)) -> i128 {
    i128::from(a.0) * i128::from(b.1) - i128::from(a.1) * i128::from(b.0)
}

/// テスト用の二点の差を返します。
pub fn difference(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    (a.0 - b.0, a.1 - b.1)
}

/// テスト用の二点間の距離の二乗を返します。
pub fn distance_squared(a: (i64, i64), b: (i64, i64)) -> i128 {
    let (x, y) = difference(a, b);
    i128::from(x) * i128::from(x) + i128::from(y) * i128::from(y)
}

/// 異なる座標の短い点列を、再現可能な整数列から生成します。
pub fn create_points(count: usize, seed: u64) -> Vec<(i64, i64)> {
    let mut state = seed + 1;
    let mut points = Vec::new();
    while points.len() < count {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = ((state >> 32) % 41) as i64 - 20;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let y = ((state >> 32) % 41) as i64 - 20;
        if !points.contains(&(x, y)) {
            points.push((x, y));
        }
    }
    points
}

/// 点数と座標列だけの入力を組み立てます。
pub fn format_points(points: &[(i64, i64)]) -> String {
    let mut input = format!("{}\n", points.len());
    for &(x, y) in points {
        input.push_str(&format!("{x} {y}\n"));
    }
    input
}

/// Library Checker の複数ケース形式の入力を組み立てます。
pub fn format_cases(cases: &[Vec<(i64, i64)>]) -> String {
    let mut input = format!("{}\n", cases.len());
    for points in cases {
        input.push_str(&format_points(points));
    }
    input
}

/// 実数の出力を、問題の絶対誤差または相対誤差の条件で確認します。
pub fn assert_float(expected: f64, output: &str, tolerance: f64) {
    let tokens = output.split_whitespace().collect::<Vec<_>>();
    assert_eq!(1, tokens.len());
    let actual = tokens[0].parse::<f64>().expect("output must be a number");
    assert!(actual.is_finite());
    assert!(
        (expected - actual).abs() <= tolerance * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}

/// 原点を角度 0 とする偏角順を、三区分と整数の外積で確認します。
pub fn assert_argument_sort(input: &str, output: &str) {
    let mut input = input.split_whitespace();
    let n = input.next().unwrap().parse::<usize>().unwrap();
    let expected = (0..n)
        .map(|_| {
            (
                input.next().unwrap().parse::<i64>().unwrap(),
                input.next().unwrap().parse::<i64>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let mut output = output.split_whitespace();
    let actual = (0..n)
        .map(|_| {
            (
                output.next().expect("missing x").parse::<i64>().unwrap(),
                output.next().expect("missing y").parse::<i64>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert!(output.next().is_none());
    let mut expected_multiset = expected.clone();
    let mut actual_multiset = actual.clone();
    expected_multiset.sort_unstable();
    actual_multiset.sort_unstable();
    assert_eq!(expected_multiset, actual_multiset);
    for pair in actual.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let category = |(x, y): (i64, i64)| {
            if y < 0 {
                0
            } else if y == 0 && x >= 0 {
                1
            } else {
                2
            }
        };
        let first = category(a);
        let second = category(b);
        assert!(
            first < second || first == second && det(a, b) >= 0,
            "arguments are reversed: {a:?}, {b:?}"
        );
    }
}

/// Gift wrapping により、境界の途中の点を除いた凸包を求めます。
pub fn reference_hull(points: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut points = points.to_vec();
    points.sort_unstable();
    points.dedup();
    if points.len() <= 1 {
        return points;
    }
    let mut result = Vec::new();
    let mut current = points[0];
    loop {
        result.push(current);
        let mut next = *points.iter().find(|&&p| p != current).unwrap();
        for &point in &points {
            let cross = det(difference(next, current), difference(point, current));
            if cross < 0
                || cross == 0 && distance_squared(current, point) > distance_squared(current, next)
            {
                next = point;
            }
        }
        current = next;
        if current == result[0] {
            break;
        }
        assert!(
            result.len() <= points.len(),
            "reference hull failed to close"
        );
    }
    result
}

/// Library Checker の複数ケース入力から点列を読み取ります。
fn parse_cases(input: &str) -> Vec<Vec<(i64, i64)>> {
    let mut tokens = input.split_whitespace();
    let t = tokens.next().unwrap().parse::<usize>().unwrap();
    (0..t)
        .map(|_| {
            let n = tokens.next().unwrap().parse::<usize>().unwrap();
            (0..n)
                .map(|_| {
                    (
                        tokens.next().unwrap().parse::<i64>().unwrap(),
                        tokens.next().unwrap().parse::<i64>().unwrap(),
                    )
                })
                .collect()
        })
        .collect()
}

/// 回転位置を固定せず、凸包の頂点集合と反時計回りを確認します。
pub fn assert_hulls(input: &str, output: &str) {
    let mut tokens = output.split_whitespace();
    for points in parse_cases(input) {
        let n = tokens
            .next()
            .expect("missing hull size")
            .parse::<usize>()
            .unwrap();
        let actual = (0..n)
            .map(|_| {
                (
                    tokens
                        .next()
                        .expect("missing hull x")
                        .parse::<i64>()
                        .unwrap(),
                    tokens
                        .next()
                        .expect("missing hull y")
                        .parse::<i64>()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let expected = reference_hull(&points);
        let mut normalized = actual;
        // 始点だけを揃え、自己交差する星形の巡回順も拒否します。
        if let Some((start, _)) = normalized.iter().enumerate().min_by_key(|&(_, p)| p) {
            normalized.rotate_left(start);
        }
        assert_eq!(expected, normalized);
    }
    assert!(tokens.next().is_none());
}

/// 同率の答えを認め、元の異なる二添字と最遠距離を確認します。
pub fn assert_furthest_pairs(input: &str, output: &str) {
    let mut tokens = output.split_whitespace();
    for points in parse_cases(input) {
        let a = tokens
            .next()
            .expect("missing first index")
            .parse::<usize>()
            .unwrap();
        let b = tokens
            .next()
            .expect("missing second index")
            .parse::<usize>()
            .unwrap();
        assert!(a < points.len() && b < points.len());
        assert_ne!(a, b);
        let mut expected = 0;
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                expected = expected.max(distance_squared(points[i], points[j]));
            }
        }
        assert_eq!(expected, distance_squared(points[a], points[b]));
    }
    assert!(tokens.next().is_none());
}

/// 全部分集合のベクトル和から、最長距離を求めます。
pub fn reference_engines(points: &[(i64, i64)]) -> f64 {
    let mut best = 0_i128;
    for mask in 0..1_usize << points.len() {
        let mut sum = (0_i128, 0_i128);
        for (i, &(x, y)) in points.iter().enumerate() {
            if mask >> i & 1 == 1 {
                sum.0 += i128::from(x);
                sum.1 += i128::from(y);
            }
        }
        best = best.max(sum.0 * sum.0 + sum.1 * sum.1);
    }
    (best as f64).sqrt()
}

/// 全ての点を境界候補にし、半周に属する点を直接数えます。
pub fn reference_halfplane(points: &[(i64, i64)]) -> usize {
    points
        .iter()
        .map(|&start| {
            points
                .iter()
                .filter(|&&other| {
                    let cross = det(start, other);
                    let dot = i128::from(start.0) * i128::from(other.0)
                        + i128::from(start.1) * i128::from(other.1);
                    cross > 0 || cross == 0 && dot > 0
                })
                .count()
        })
        .max()
        .unwrap_or(0)
}

/// 全ての三点と角の頂点を調べ、最大の角度を度数法で返します。
pub fn reference_max_angle(points: &[(i64, i64)]) -> f64 {
    let mut best = 0.0_f64;
    for (i, &origin) in points.iter().enumerate() {
        for (j, &a) in points.iter().enumerate() {
            for (k, &b) in points.iter().enumerate() {
                if i == j || i == k || j == k {
                    continue;
                }
                let a = difference(a, origin);
                let b = difference(b, origin);
                let dot = i128::from(a.0) * i128::from(b.0) + i128::from(a.1) * i128::from(b.1);
                best = best.max((det(a, b) as f64).abs().atan2(dot as f64).to_degrees());
            }
        }
    }
    best
}

/// 各三角形の三つの内積から、鋭角・直角・鈍角の個数を求めます。
pub fn reference_triangle_counts(points: &[(i64, i64)]) -> (usize, usize, usize) {
    let mut answer = (0, 0, 0);
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            for k in j + 1..points.len() {
                assert_ne!(
                    0,
                    det(
                        difference(points[j], points[i]),
                        difference(points[k], points[i])
                    )
                );
                let mut kind = cmp::Ordering::Greater;
                for (origin, a, b) in [(i, j, k), (j, i, k), (k, i, j)] {
                    let a = difference(points[a], points[origin]);
                    let b = difference(points[b], points[origin]);
                    let dot = i128::from(a.0) * i128::from(b.0) + i128::from(a.1) * i128::from(b.1);
                    kind = kind.min(dot.cmp(&0));
                }
                match kind {
                    cmp::Ordering::Greater => answer.0 += 1,
                    cmp::Ordering::Equal => answer.1 += 1,
                    cmp::Ordering::Less => answer.2 += 1,
                }
            }
        }
    }
    answer
}

/// 第一象限の可視区間を全ての部分集合について検証します。
pub fn reference_visible_sevens(points: &[(i64, i64)]) -> usize {
    let mut best = 0;
    for mask in 0..1_usize << points.len() {
        let mut valid = true;
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                if mask >> i & 1 == 0 || mask >> j & 1 == 0 {
                    continue;
                }
                let a = points[i];
                let b = points[j];
                let separate = det((a.0 - 1, a.1), (b.0, b.1 - 1)) >= 0
                    || det((b.0 - 1, b.1), (a.0, a.1 - 1)) >= 0;
                valid &= separate;
            }
        }
        if valid {
            best = best.max(mask.count_ones() as usize);
        }
    }
    best
}

/// 参照用の整数係数を約分する最大公約数です。
fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// 全点対から直線の整数係数を作り、点数条件を満たすものを数えます。
pub fn reference_colinear_lines(points: &[(i64, i64)], k: usize) -> String {
    if k == 1 {
        return "Infinity\n".to_owned();
    }
    let mut lines = collections::BTreeSet::new();
    for i in 0..points.len() {
        for j in i + 1..points.len() {
            let mut a = i128::from(points[j].1 - points[i].1);
            let mut b = i128::from(points[i].0 - points[j].0);
            let mut c = -(a * i128::from(points[i].0) + b * i128::from(points[i].1));
            let factor = gcd(gcd(a, b), c);
            a /= factor;
            b /= factor;
            c /= factor;
            if a < 0 || a == 0 && b < 0 {
                a = -a;
                b = -b;
                c = -c;
            }
            lines.insert((a, b, c));
        }
    }
    let count = lines
        .into_iter()
        .filter(|&(a, b, c)| {
            points
                .iter()
                .filter(|&&(x, y)| a * i128::from(x) + b * i128::from(y) + c == 0)
                .count()
                >= k
        })
        .count();
    format!("{count}\n")
}

/// 狭義凸な反時計回りの多角形を、全辺の走査で内外判定します。
pub fn reference_location(vertices: &[(i64, i64)], point: (i64, i64)) -> &'static str {
    let mut boundary = false;
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];
        let cross = det(difference(b, a), difference(point, a));
        if cross < 0 {
            return "OUT";
        }
        boundary |= cross == 0;
    }
    if boundary { "ON" } else { "IN" }
}

//! 固定幅整数の範囲を超える積和を、符号と 129 ビットの絶対値で扱います。

use std::cmp;

/// 二つの積の和までを保持する、非公開の符号付き整数です。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Wide {
    /// 絶対値が 0 のときは偽に正規化します。
    negative: bool,
    /// 絶対値の 129 ビット目を表します。
    high: bool,
    /// 絶対値の下位 128 ビットです。
    low: u128,
}

impl Wide {
    /// i128 の値を同じ符号と絶対値に変換します。
    ///
    /// # Args
    /// - `value`: 変換する値。
    /// # Returns
    /// 同じ整数値を表す内部表現。
    fn from_i128(value: i128) -> Self {
        Self {
            negative: value < 0,
            high: false,
            low: value.unsigned_abs(),
        }
    }

    /// 二つの符号付き積の和を、和の桁上がりも含めて求めます。
    ///
    /// # Args
    /// - `a`, `b`, `c`, `d`: 各絶対値が u64::MAX 以下の因子。
    /// - `subtract`: 真なら二つ目の積を引きます。
    /// # Returns
    /// `a * b + c * d`、または `a * b - c * d` の厳密な値。
    fn products(a: i128, b: i128, c: i128, d: i128, subtract: bool) -> Self {
        // 各積は u128 に収まり、二つの和にだけ一桁の余地が必要です。
        let first = a.unsigned_abs() * b.unsigned_abs();
        let second = c.unsigned_abs() * d.unsigned_abs();
        let first_negative = (a < 0) != (b < 0);
        let second_negative = ((c < 0) != (d < 0)) != subtract;
        if first_negative == second_negative {
            let (low, high) = first.overflowing_add(second);
            Self {
                negative: first_negative && (high || low != 0),
                high,
                low,
            }
        } else if first >= second {
            Self {
                negative: first_negative && first != second,
                high: false,
                low: first - second,
            }
        } else {
            Self {
                negative: second_negative,
                high: false,
                low: second - first,
            }
        }
    }

    /// 符号を比較結果として返します。
    ///
    /// # Returns
    /// 負・ゼロ・正に対応する比較結果。
    /// # Complexity
    /// O(1)。
    #[inline]
    pub(super) fn sign(self) -> cmp::Ordering {
        if self.high || self.low != 0 {
            if self.negative {
                cmp::Ordering::Less
            } else {
                cmp::Ordering::Greater
            }
        } else {
            cmp::Ordering::Equal
        }
    }

    /// 表現できる場合だけ i128 の値を返します。
    ///
    /// # Returns
    /// 符号を含む整数値。
    ///
    /// # Panics
    /// 値が i128 の範囲を超える場合。
    pub(super) fn to_i128(self) -> i128 {
        assert!(!self.high, "geometry value does not fit i128");
        if self.negative && self.low == 1_u128 << 127 {
            i128::MIN
        } else {
            let value = i128::try_from(self.low).expect("geometry value does not fit i128");
            if self.negative { -value } else { value }
        }
    }

    /// 絶対値を u128 として返します。
    ///
    /// # Returns
    /// 非負の絶対値。
    ///
    /// # Panics
    /// 絶対値が u128 の範囲を超える場合。
    pub(super) fn magnitude(self) -> u128 {
        assert!(!self.high, "geometry value does not fit u128");
        self.low
    }

    /// 近似的な角度の計算に使う実数値を返します。
    ///
    /// # Returns
    /// 符号を保った有限な近似値。
    /// # Complexity
    /// O(1)。
    pub(super) fn to_f64(self) -> f64 {
        let magnitude = self.low as f64 + if self.high { 2.0_f64.powi(128) } else { 0.0 };
        if self.negative { -magnitude } else { magnitude }
    }
}

impl cmp::Ord for Wide {
    /// 符号、絶対値の上位桁、下位桁の順で比較します。
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.sign().cmp(&other.sign()).then_with(|| {
            let order = (self.high, self.low).cmp(&(other.high, other.low));
            if self.negative {
                order.reverse()
            } else {
                order
            }
        })
    }
}

impl cmp::PartialOrd for Wide {
    /// 整数の全順序を返します。
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// 二つの符号付き積を、各積が収まる u128 の絶対値で比較します。
///
/// # Args
/// - `a`, `b`, `c`, `d`: 絶対値が u64::MAX 以下の因子。
/// # Returns
/// `a * b` と `c * d` の比較。値の和や差は構成しません。
#[inline]
fn product_cmp(a: i128, b: i128, c: i128, d: i128) -> cmp::Ordering {
    debug_assert!(
        [a, b, c, d]
            .iter()
            .all(|v| v.unsigned_abs() <= u128::from(u64::MAX))
    );
    // 各因子の幅を示すことで、64 ビット同士の積として生成できます。
    let first = u128::from(a.unsigned_abs() as u64) * u128::from(b.unsigned_abs() as u64);
    let second = u128::from(c.unsigned_abs() as u64) * u128::from(d.unsigned_abs() as u64);
    let first_negative = ((a < 0) != (b < 0)) && first != 0;
    let second_negative = ((c < 0) != (d < 0)) && second != 0;
    match (first_negative, second_negative) {
        (true, false) => cmp::Ordering::Less,
        (false, true) => cmp::Ordering::Greater,
        (true, true) => second.cmp(&first),
        (false, false) => first.cmp(&second),
    }
}

/// 外積の符号だけを、積同士の比較で返します。
///
/// # Args
/// - `a`, `b`: 各成分の絶対値が u64::MAX 以下のベクトル。
/// # Returns
/// 外積の符号を表す比較結果。
/// # Complexity
/// O(1)。
#[inline]
pub(super) fn cross_sign(a: (i128, i128), b: (i128, i128)) -> cmp::Ordering {
    product_cmp(a.0, b.1, a.1, b.0)
}

/// 内積の符号だけを、符号反転した積との比較で返します。
///
/// # Args
/// - `a`, `b`: 各成分の絶対値が u64::MAX 以下のベクトル。
/// # Returns
/// 内積の符号を表す比較結果。
/// # Complexity
/// O(1)。
#[inline]
pub(super) fn dot_sign(a: (i128, i128), b: (i128, i128)) -> cmp::Ordering {
    product_cmp(a.0, b.0, -a.1, b.1)
}

/// 外積を、通常の座標では i64 の経路で計算します。
///
/// # Args
/// - `a`, `b`: 各成分の絶対値が u64::MAX 以下の二次元ベクトル。
/// # Returns
/// 厳密な外積。
#[inline]
pub(super) fn cross(a: (i128, i128), b: (i128, i128)) -> Wide {
    let bits = a.0.unsigned_abs() | a.1.unsigned_abs() | b.0.unsigned_abs() | b.1.unsigned_abs();
    if bits <= i32::MAX as u128 {
        // 各絶対値が 2^31 未満なら、二つの積の差まで i64 に収まります。
        let value = a.0 as i64 * b.1 as i64 - a.1 as i64 * b.0 as i64;
        Wide::from_i128(i128::from(value))
    } else {
        Wide::products(a.0, b.1, a.1, b.0, true)
    }
}

/// 内積を、必要な場合だけ 129 ビットへ広げて計算します。
///
/// # Args
/// - `a`, `b`: 各成分の絶対値が u64::MAX 以下の二次元ベクトル。
/// # Returns
/// 厳密な内積。
#[inline]
pub(super) fn dot(a: (i128, i128), b: (i128, i128)) -> Wide {
    let bits = a.0.unsigned_abs() | a.1.unsigned_abs() | b.0.unsigned_abs() | b.1.unsigned_abs();
    if bits <= i32::MAX as u128 {
        let value = a.0 as i64 * b.0 as i64 + a.1 as i64 * b.1 as i64;
        Wide::from_i128(i128::from(value))
    } else {
        Wide::products(a.0, b.0, a.1, b.1, false)
    }
}

/// i64 の座標を広げてから二点差を求めます。
///
/// # Args
/// - `from`, `to`: 差の始点と終点。
/// # Returns
/// `to - from` の二成分。
pub(super) fn difference(from: (i64, i64), to: (i64, i64)) -> (i128, i128) {
    (
        i128::from(to.0) - i128::from(from.0),
        i128::from(to.1) - i128::from(from.1),
    )
}

/// 原点を正の x 軸と同じ位置に置き、(-π, π] の偏角順を比較します。
///
/// # Args
/// - `a`, `b`: 各成分の絶対値が u64::MAX 以下のベクトル。
/// # Returns
/// 同じ方向を等しいとする偏角の比較。
#[inline]
pub(super) fn argument_cmp(a: (i128, i128), b: (i128, i128)) -> cmp::Ordering {
    let a = if a == (0, 0) { (1, 0) } else { a };
    let b = if b == (0, 0) { (1, 0) } else { b };
    let half = |(x, y)| y > 0 || y == 0 && x < 0;
    half(a)
        .cmp(&half(b))
        .then_with(|| cross_sign(a, b).reverse())
}

/// 任意の非ゼロ始線から反時計回りに進む偏角順を比較します。
///
/// # Args
/// - `start`, `a`, `b`: 非ゼロの方向ベクトル。
/// # Returns
/// 始線と同方向を先頭にした比較。
#[inline]
pub(super) fn ccw_cmp(start: (i128, i128), a: (i128, i128), b: (i128, i128)) -> cmp::Ordering {
    let a_before = argument_cmp(a, start).is_lt();
    let b_before = argument_cmp(b, start).is_lt();
    a_before.cmp(&b_before).then_with(|| argument_cmp(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: 広い積の差と平方和を、i128 の上限に依存せず比較できます。
    /// - Given: 外積が -1 となるほぼ平行な方向と、最大の二点差があります。
    /// - When: 積和の符号、値、大小を取得します。
    /// - Then: 桁上がりと打ち消しを保った整数の結果になります。
    #[test]
    fn preserves_cancellation_and_the_extra_carry_bit() {
        // Given
        let maximum = i128::from(u64::MAX);
        let first = (maximum, maximum - 1);
        let second = (maximum - 1, maximum - 2);
        // When
        let determinant = cross(first, second);
        let horizontal = dot((maximum, 0), (maximum, 0));
        let diagonal = dot((maximum, maximum), (maximum, maximum));
        // Then
        assert_eq!(-1, determinant.to_i128());
        assert_eq!(cmp::Ordering::Greater, dot(first, second).sign());
        assert!(diagonal > horizontal);
        assert_eq!(u128::from(u64::MAX).pow(2), horizontal.magnitude());
        assert!(diagonal.high);
        let negative = dot((maximum, maximum), (-maximum, -maximum));
        assert!(negative < Wide::from_i128(i128::MIN));
        assert!(negative.to_f64().is_sign_negative());
        assert_eq!(
            Wide::from_i128(0),
            dot((maximum, maximum), (maximum, -maximum))
        );
    }

    /// Scenario: 高速経路と広い値の経路の境界で結果が変わりません。
    /// - Given: i32 の最大値と、その直後の成分があります。
    /// - When: 積和の値と符号を求めます。
    /// - Then: i128 の参照計算と一致し、i64 を超える値も保持します。
    #[rstest::rstest]
    #[case::last_small(i128::from(i32::MAX))]
    #[case::first_wide(i128::from(i32::MAX) + 1)]
    #[case::next_wide(i128::from(i32::MAX) + 2)]
    fn agrees_across_the_fast_path_boundary(#[case] value: i128) {
        // Given
        let a = (value, value);
        let b = (-value, value);
        let expected = 2 * value * value;
        // When / Then
        assert_eq!(expected, dot(a, a).to_i128());
        assert_eq!(expected, cross(a, b).to_i128());
        assert_eq!(0, dot(a, b).to_i128());
        assert_eq!(cmp::Ordering::Greater, cross_sign(a, b));
        assert_eq!(cmp::Ordering::Equal, dot_sign(a, b));
    }

    /// Scenario: ゼロの符号と最小の i128 の値を正規化できます。
    /// - Given: 正負の積の打ち消しと、i128 の最小値があります。
    /// - When: 内部表現を比較し、値を戻します。
    /// - Then: ゼロは同じ表現になり、最小値の絶対値変換で失敗しません。
    #[test]
    fn normalizes_zero_and_round_trips_the_minimum() {
        // Given
        let minimum = i128::MIN;
        // When
        let zero = Wide::products(-1, 1, 1, 1, false);
        // Then
        assert_eq!(Wide::from_i128(0), zero);
        assert_eq!(minimum, Wide::from_i128(minimum).to_i128());
        assert!(Wide::from_i128(minimum) < Wide::from_i128(-1));
    }
}

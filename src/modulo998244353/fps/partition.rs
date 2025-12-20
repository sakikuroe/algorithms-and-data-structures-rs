//! 分割の数え上げなどで用いる生成関数の構築を提供するモジュールである.

use super::super::convolution;
use super::super::modulo;

/// 対数級数 `Σ cnt[k] * log(1 ± x^k)` の符号パターンを表す.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogSeriesKind {
    /// `log(1 + x^k)` を用いる. 係数は `(+,-,+,-,...)` で交互に現れる.
    OnePlus,
    /// `log(1 - x^k)` を用いる. 係数は常に負である.
    OneMinus,
    /// `-log(1 + x^k)` を用いる. 係数は `(-,+,-,+,...)` で交互に現れる.
    InvOnePlus,
    /// `-log(1 - x^k)` を用いる. 係数は常に正である.
    InvOneMinus,
}

/// `LogSeriesKind` と `j` に応じて, 対象の項を加算するか減算するかを返す.
///
/// # Args
/// - `kind`: 対応する生成関数の種類.
/// - `j`: `x^(k*j)` の `j` を表す (1 以上).
///
/// # Returns
/// `bool`: 加算なら `true`, 減算なら `false`.
///
/// # Constraints
/// - `j >= 1` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(1).
/// - Space complexity: O(1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
fn should_add_term(kind: LogSeriesKind, j: usize) -> bool {
    debug_assert!(j >= 1);
    match kind {
        LogSeriesKind::OnePlus => j % 2 == 1,
        LogSeriesKind::OneMinus => false,
        LogSeriesKind::InvOnePlus => j % 2 == 0,
        LogSeriesKind::InvOneMinus => true,
    }
}

/// `exponents` を `degree` 以下で数え上げ, `log` に相当する級数を構築する.
///
/// # Args
/// - `exponents`: `x` の指数列であり, `a[i]` は `(1 ± x^{a[i]})` の指数に対応する.
/// - `degree`: 計算する最高次数 (含む).
/// - `kind`: 構築する対数級数の種類.
///
/// # Returns
/// `Option<(super::FPS, usize)>`: 対数級数と, `a[i] = 0` の個数を返す.
///
/// # Constraints
/// - `degree + 1 < 998244353` を満たす.
/// - `degree + 1 <= convolution::MAX_NTT_LEN` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(N + (degree + 1) log(degree + 1)).
/// - Space complexity: O(degree + 1).
///
/// # Examples
/// ```rust,ignore
/// // 内部関数のため, 直接の使用例は省略する.
/// ```
fn build_log_series(
    exponents: &[u32],
    degree: usize,
    kind: LogSeriesKind,
) -> Option<(super::FPS, usize)> {
    let target_len = degree.checked_add(1)?;
    if target_len >= modulo::M as usize {
        return None;
    }
    if target_len > convolution::MAX_NTT_LEN {
        return None;
    }

    let mut zero_count = 0usize;
    let mut counts = vec![0_u32; target_len];
    for &x in exponents {
        if x == 0 {
            zero_count += 1;
            continue;
        }
        let idx = x as usize;
        if idx < target_len {
            counts[idx] = modulo::add(counts[idx], 1);
        }
    }

    let inv_indices = modulo::build_inv_indices(target_len);
    let mut coeffs = vec![0_u32; target_len];

    for k in 1..target_len {
        let count_k = counts[k];
        if count_k == 0 {
            continue;
        }
        let mut j = 1usize;
        while k * j < target_len {
            let idx = k * j;
            let term = modulo::mul(count_k, inv_indices[j]);
            if should_add_term(kind, j) {
                coeffs[idx] = modulo::add(coeffs[idx], term);
            } else {
                coeffs[idx] = modulo::sub(coeffs[idx], term);
            }
            j += 1;
        }
    }

    Some((super::FPS::new(coeffs), zero_count))
}

/// `Π(1 + x^{a[i]})` を `x^degree` まで (含む) 計算する.
///
/// # Args
/// - `exponents`: `x` の指数列.
/// - `degree`: 計算する最高次数 (含む).
///
/// # Returns
/// `Option<super::FPS>`: 制約を満たすときの切り詰められた生成関数.
///
/// # Constraints
/// - `degree + 1 < 998244353` を満たす.
/// - `degree + 1 <= convolution::MAX_NTT_LEN` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(N + (degree + 1) log(degree + 1)).
/// - Space complexity: O(degree + 1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps;
/// use anmitsu::modulo998244353::fps::partition;
///
/// let exponents = vec![1, 2, 2];
/// let f = partition::product_one_plus_x_powers(&exponents, 4).unwrap();
/// assert_eq!(1, f.get(0));
/// ```
pub fn product_one_plus_x_powers(exponents: &[u32], degree: usize) -> Option<super::FPS> {
    let (log_series, zero_count) = build_log_series(exponents, degree, LogSeriesKind::OnePlus)?;
    let mut res = log_series.exp(degree)?;

    if zero_count > 0 {
        let factor = modulo::pow(2, zero_count);
        for c in res.coeffs.iter_mut() {
            *c = modulo::mul(*c, factor);
        }
        res.trim();
    }

    Some(res)
}

/// `Π(1 - x^{a[i]})` を `x^degree` まで (含む) 計算する.
///
/// # Args
/// - `exponents`: `x` の指数列.
/// - `degree`: 計算する最高次数 (含む).
///
/// # Returns
/// `Option<super::FPS>`: 制約を満たすときの切り詰められた生成関数.
///
/// # Constraints
/// - `degree + 1 < 998244353` を満たす.
/// - `degree + 1 <= convolution::MAX_NTT_LEN` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(N + (degree + 1) log(degree + 1)).
/// - Space complexity: O(degree + 1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps::partition;
///
/// let exponents = vec![1];
/// let f = partition::product_one_minus_x_powers(&exponents, 2).unwrap();
/// assert_eq!(1, f.get(0));
/// ```
pub fn product_one_minus_x_powers(exponents: &[u32], degree: usize) -> Option<super::FPS> {
    let (log_series, zero_count) = build_log_series(exponents, degree, LogSeriesKind::OneMinus)?;

    if zero_count > 0 {
        return Some(super::FPS::new(Vec::new()));
    }

    log_series.exp(degree)
}

/// `Π(1/(1 + x^{a[i]}))` を `x^degree` まで (含む) 計算する.
///
/// # Args
/// - `exponents`: `x` の指数列.
/// - `degree`: 計算する最高次数 (含む).
///
/// # Returns
/// `Option<super::FPS>`: 制約を満たすときの切り詰められた生成関数.
///
/// # Constraints
/// - `degree + 1 < 998244353` を満たす.
/// - `degree + 1 <= convolution::MAX_NTT_LEN` を満たす.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(N + (degree + 1) log(degree + 1)).
/// - Space complexity: O(degree + 1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps::partition;
///
/// let exponents = vec![1];
/// let f = partition::product_inv_one_plus_x_powers(&exponents, 3).unwrap();
/// assert_eq!(1, f.get(0));
/// ```
pub fn product_inv_one_plus_x_powers(exponents: &[u32], degree: usize) -> Option<super::FPS> {
    let (log_series, zero_count) = build_log_series(exponents, degree, LogSeriesKind::InvOnePlus)?;
    let mut res = log_series.exp(degree)?;

    if zero_count > 0 {
        let inv_2 = modulo::inv(2);
        let factor = modulo::pow(inv_2, zero_count);
        for c in res.coeffs.iter_mut() {
            *c = modulo::mul(*c, factor);
        }
        res.trim();
    }

    Some(res)
}

/// `Π(1/(1 - x^{a[i]}))` を `x^degree` まで (含む) 計算する.
///
/// # Args
/// - `exponents`: `x` の指数列.
/// - `degree`: 計算する最高次数 (含む).
///
/// # Returns
/// `Option<super::FPS>`: 制約を満たすときの切り詰められた生成関数.
///
/// # Constraints
/// - `degree + 1 < 998244353` を満たす.
/// - `degree + 1 <= convolution::MAX_NTT_LEN` を満たす.
/// - `a[i] = 0` を含まない.
///
/// # Panics
/// - この関数はパニックしない.
///
/// # Complexity
/// - Time complexity: O(N + (degree + 1) log(degree + 1)).
/// - Space complexity: O(degree + 1).
///
/// # Examples
/// ```rust
/// use anmitsu::modulo998244353::fps::partition;
///
/// let exponents = vec![2];
/// let f = partition::product_inv_one_minus_x_powers(&exponents, 5).unwrap();
/// assert_eq!(1, f.get(0));
/// ```
pub fn product_inv_one_minus_x_powers(exponents: &[u32], degree: usize) -> Option<super::FPS> {
    let (log_series, zero_count) = build_log_series(exponents, degree, LogSeriesKind::InvOneMinus)?;

    if zero_count > 0 {
        return None;
    }

    log_series.exp(degree)
}

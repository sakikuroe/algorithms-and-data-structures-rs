//! 整数の `k` 乗根に関連する機能を提供するモジュールである。

/// `a` の `k` 乗根の整数部分 (`floor(a^(1/k))`) を求める。
///
/// # Args
/// - `a` - 対象の整数
/// - `k` - 乗根の次数であり、`1` 以上である必要がある。この契約に違反する
///   場合、`debug_assert!` によりパニックする。
///
/// # Returns
/// `floor(a^(1/k))` の値。
///
/// # Complexity
/// - 時間計算量: ならし $O(\log k)$
///   - CPU の浮動小数点演算 (`k = 2` は `f64::sqrt`、`k >= 3` は `f64::powf`)
///     によって真の値にごく近い初期値を求め、[`pow_leq`] による前後わずかな
///     回数の判定だけで真の値に到達する (根拠は [`kth_root_2`]、
///     [`kth_root_general`] のコメントを参照)。判定 1 回あたり
///     $O(\log k)$ 回の乗算を要する。
///
/// # Examples
/// ```
/// use anmitsu::math::kth_root;
///
/// assert_eq!(0, kth_root::kth_root(0, 5));
/// assert_eq!(2, kth_root::kth_root(8, 3));
/// assert_eq!(3, kth_root::kth_root(10, 2));
/// assert_eq!(u64::MAX, kth_root::kth_root(u64::MAX, 1));
/// ```
#[must_use]
pub fn kth_root(a: u64, k: u64) -> u64 {
    debug_assert!(k >= 1);

    // k = 1 のとき a^(1/1) = a は自明である。この早期リターンは、単なる
    // 高速化ではなく必須のガードである。真の値は a 自身であり、a as f64 の
    // 丸め誤差 (相対誤差 2^-53) がそのまま絶対誤差 (a が大きいほど拡大する)
    // になるため、kth_root_general に処理を任せると探索が数百〜数千回に
    // 及ぶことがある (実測: a = 2^63 付近で 1000 回超)。さらに a = u64::MAX
    // の場合、探索が x = u64::MAX まで到達すると kth_root_general 内の
    // `x + 1` がオーバーフローするため、速度だけでなく正しさの面でも
    // 必須のガードである。
    //
    // なお a = 0 や k >= 64 には同様の早期リターンを設けていない。これらは
    // kth_root_2 / kth_root_general にそのまま渡しても、真の値が 0 または 1
    // という小さい値に固定されるため、丸め誤差が絶対誤差として拡大せず、
    // 探索は数回で収束する (実測でも反復回数は最大 2)。
    if k == 1 {
        return a;
    }

    if k == 2 {
        kth_root_2(a)
    } else {
        kth_root_general(a, k)
    }
}

/// `k = 2` の場合の `kth_root`。IEEE 754 は `sqrt` を「正しく丸める」ことを
/// 規格として要求しており (相対誤差 <= 0.5 ULP = `2^-53`)、これは特定の
/// 実装に依存しない理論的な保証である。
fn kth_root_2(a: u64) -> u64 {
    // a as f64 へのキャストも IEEE 754 で正しく丸められる (相対誤差 <= 2^-53)。
    // したがって sqrt(a as f64) は、真の数学的平方根 sqrt(a) に対して相対誤差
    // 高々 2^-54 + 2^-53 < 2^-52 で近似する。a < 2^64 より sqrt(a) < 2^32 なので、
    // 絶対誤差は 2^32 * 2^-52 = 2^-20 未満であり、0 に極めて近い。
    //
    // この誤差 (2^-20 未満) が整数境界をまたぐのは、真の平方根が整数のごく
    // 近傍にある場合に限られ、その場合でも計算結果は真の値からたかだか 1
    // だけしかずれ得ない。すなわち、計算結果を切り捨てて得た値は、真の値
    // floor(sqrt(a)) から高々 1 しかずれないことが理論的に保証される。
    // MARGIN = 1 を引いておけば、探索の初期値は必ず真の値以下になるため、
    // 減らす方向の補正は理論上不要であり、増やす方向の探索だけで済む。
    const MARGIN: u64 = 1;

    // f64 as u64 キャストは IEEE 754 上の範囲外の値を自動的に飽和させる
    // 仕様であるため (NaN は 0、負値は 0、u64::MAX を超える値は
    // u64::MAX になる)、追加の範囲チェックなしに安全に変換できる。
    let mut x = ((a as f64).sqrt() as u64).saturating_sub(MARGIN);

    // 真の値は x^2 <= a < (x+1)^2 を満たす唯一の x である。
    while pow_leq(x + 1, 2, a) {
        x += 1;
    }

    x
}

/// `k >= 3` の場合の `kth_root`。
fn kth_root_general(a: u64, k: u64) -> u64 {
    // f64::powf(a, 1/k) = exp((1/k) * ln(a)) を使う。powf は sqrt と異なり
    // 正しい丸めが規格上保証されているわけではなく、精度は libm の実装に
    // 依存する (一般的な実装では数 ULP 程度に収まることが知られているが、
    // 規格として保証された上界ではない)。したがって sqrt の場合のように
    // 「初期値は理論上必ず真の値以下になる」とは言い切れず、初期値が
    // 真の値より大きすぎる可能性を残したまま探索する必要がある。
    //
    // 実装依存の誤差を安全側に見積もり、相対誤差を 2^-40 程度と仮定しても、
    // 初期値 x0 の絶対誤差は x0 * 2^-40 であり、x0 の最大値
    // (a < 2^64, k >= 3 より x0 < 2^(64/3) < 2^22) を踏まえても
    // 2^22 * 2^-40 = 2^-18 未満と極めて小さい。それでも規格上の保証が
    // ない以上、余裕を持ったマージンで両方向を探索する。
    const MARGIN: u64 = 4;

    let approx = (a as f64).powf(1.0 / k as f64);
    let mut x = (approx as u64).saturating_sub(MARGIN);

    // 初期値が真の値より大きすぎた場合に備え、まず減らす方向に補正する。
    while x > 0 && !pow_leq(x, k, a) {
        x -= 1;
    }
    // 真の値は x^k <= a < (x+1)^k を満たす唯一の x であるため、
    // (x+1)^k <= a である限り増やす方向へ補正する。
    while pow_leq(x + 1, k, a) {
        x += 1;
    }

    x
}

/// `x^k <= a` かどうかを、二分累乗法によって `O(log k)` 回の乗算で判定する。
///
/// 二分累乗法では、部分積の基数 `base` (`x^(2^i)` に相当) は単調非減少であり、
/// `k` を 2 進数表現したときの最上位ビットは必ず 1 であるため、`base` が
/// 一度でも `a` を超えれば、そのビットは遅くとも最上位ビットの処理までに
/// 必ず `result` に掛け合わされ、`result` も最終的に必ず `a` を超える。
/// したがって `base > a` になった時点で `false` を返してよく、この早期
/// 打ち切りにより `base`、`result` は常に `<= a < 2^64` に保たれるため、
/// 続く二乗・乗算 (`checked_mul`) が `u64` の範囲を超えるのは、その真の値が
/// `a` を上回った場合に限られる。
///
/// `checked_mul` を使うのは `saturating_mul` では正確に判定できないためで
/// ある。「真の値がちょうど `u64::MAX`」なのか「`u64::MAX` を超えている」の
/// かを区別できず、`a == u64::MAX` の境界で誤判定しうる。
fn pow_leq(x: u64, k: u64, a: u64) -> bool {
    let mut base = x;
    let mut result = 1_u64;
    let mut exp = k;

    while exp > 0 {
        if base > a {
            return false;
        }
        if exp & 1 == 1 {
            result = match result.checked_mul(base) {
                Some(v) if v <= a => v,
                _ => return false,
            };
        }
        exp >>= 1;
        if exp > 0 {
            base = match base.checked_mul(base) {
                Some(v) => v,
                None => return false,
            };
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // kth_root のテスト: 戻り値を検証する。
    mod kth_root {
        use super::*;

        /// Scenario: `0` に対しては `k` によらず `0` を返す (境界値)。
        /// - Given: `0` と、いくつかの `k` がある。
        /// - When: `kth_root` を呼ぶ。
        /// - Then: `0` が返る。
        #[test]
        fn returns_zero_for_zero() {
            let cases = [1_u64, 2, 5, 64];

            for k in cases {
                // Given, When
                let result = kth_root(0, k);
                // Then
                assert_eq!(0, result);
            }
        }

        /// Scenario: `k = 1` の場合は `a` 自身を返す (境界値)。
        /// - Given: いくつかの `a` がある。
        /// - When: `k = 1` として `kth_root` を呼ぶ。
        /// - Then: `a` がそのまま返る。
        #[test]
        fn returns_itself_when_k_is_one() {
            let cases = [1_u64, 12345, u64::MAX];

            for a in cases {
                // Given, When
                let result = kth_root(a, 1);
                // Then
                assert_eq!(a, result);
            }
        }

        /// Scenario: 完全累乗数に対しては、その底を厳密に返す。
        /// - Given: `b^k` の形で表せる典型的な完全累乗数がある。
        /// - When: `kth_root` を呼ぶ。
        /// - Then: `b` が返る。
        #[test]
        fn returns_exact_base_for_perfect_powers() {
            let cases = [
                (8_u64, 3_u64, 2_u64),
                (27, 3, 3),
                (1_000_000_000_000_u64, 4, 1000),
                (81, 4, 3),
            ];

            for (a, k, expected) in cases {
                // Given, When
                let result = kth_root(a, k);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 完全累乗数でない値に対しては、切り捨てた値を返す。
        /// - Given: `k` 乗根が整数にならない値がある。
        /// - When: `kth_root` を呼ぶ。
        /// - Then: 真の値を切り捨てた整数が返る。
        #[test]
        fn returns_floored_value_for_non_perfect_powers() {
            let cases = [(10_u64, 2_u64, 3_u64), (7, 3, 1), (26, 3, 2), (28, 3, 3)];

            for (a, k, expected) in cases {
                // Given, When
                let result = kth_root(a, k);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: `k >= 64` の場合、`a >= 1` であれば常に `1` を返す (境界値)。
        /// - Given: `k` が `64` 以上であり、`a` が `1` 以上である。
        /// - When: `kth_root` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_when_k_is_at_least_64() {
            let cases = [(1_u64, 64_u64), (u64::MAX, 64), (u64::MAX, 100)];

            for (a, k) in cases {
                // Given, When
                let result = kth_root(a, k);
                // Then
                assert_eq!(1, result);
            }
        }

        /// Scenario: `u64` の範囲に収まる大きな値でも正しく計算できる (境界値)。
        /// - Given: `u64::MAX` 付近の大きな値がある。
        /// - When: `kth_root` を呼ぶ。
        /// - Then: 返った値 `x` が `x^k <= a < (x+1)^k` を満たす。
        #[test]
        fn returns_correct_value_for_values_near_u64_max() {
            let cases = [(u64::MAX, 2_u64), (u64::MAX, 3), (u64::MAX - 1, 63)];

            for (a, k) in cases {
                // Given, When
                let result = kth_root(a, k);

                // Then
                assert!(pow_leq(result, k, a), "{result}^{k} は {a} を超えている");
                assert!(
                    !pow_leq(result + 1, k, a),
                    "{result} は {a} の {k} 乗根として大きすぎる"
                );
            }
        }
    }
}

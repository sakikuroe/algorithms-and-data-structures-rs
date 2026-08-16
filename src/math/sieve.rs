//! 線形篩 (linear sieve) による前処理を提供するモジュールである。
//!
//! `primality::is_prime`/`primality::factorize` は単発のクエリに対してその都度
//! 判定・分解を行うのに対し、このモジュールは `0` から `n` までのすべての整数に
//! ついて最小素因数を一括で前計算する。「大量の値に対して素数判定や素因数分解を
//! 繰り返し行いたい」というユースケースでは、前計算にかかる $O(n)$ の時間と
//! 空間を許容できるならこちらを使うことで、1 回あたりのクエリを高速化できる。

/// `0` から `n` までの各整数について、最小素因数 (smallest prime factor) を
/// 線形篩により前計算する。
///
/// # Args
/// - `n` - 前計算する範囲の上限
///
/// # Returns
/// 添字 `i` の要素が `i` の最小素因数であるような `Vec<usize>` を返す。
/// 具体的には:
/// - `spf[0] = 0`, `spf[1] = 1` とする。`0` と `1` はいずれも素因数を持たない
///   ため、番兵的な値として扱う。
/// - `i >= 2` の場合、`spf[i]` は `i` を割り切る最小の素数になる
///   (`i` 自身が素数であれば `spf[i] == i`)。
///
/// # Complexity
/// - 時間計算量: $O(n)$
///   - 各合成数はその最小素因数によってちょうど 1 回だけ篩い落とされるため、
///     単純なエラトステネスの篩 ($O(n \log \log n)$) より高速である。
/// - 空間計算量: $O(n)$
///   - 篩の結果と、見つかった素数の一覧を保持するために、$n$ に比例した
///     メモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::sieve;
///
/// assert_eq!(vec![0], sieve::smallest_prime_factors(0));
/// assert_eq!(vec![0, 1], sieve::smallest_prime_factors(1));
/// assert_eq!(vec![0, 1, 2, 3, 2, 5, 2], sieve::smallest_prime_factors(6));
/// ```
#[must_use]
pub fn smallest_prime_factors(n: usize) -> Vec<usize> {
    // spf[i] == 0 であることを「未確定」を表す番兵として使い、確定次第
    // 書き換えていく。
    let mut spf = vec![0_usize; n + 1];

    // これまでに見つかった素数を昇順に保持する。合成数を篩い落とす際は、
    // この一覧を小さい方から順に走査する。
    let mut primes = Vec::new();

    for i in 2..=n {
        // spf[i] が未確定のままであれば、i はそれより小さいどの素数でも
        // 割り切れなかったことを意味し、i 自身が素数である。
        if spf[i] == 0 {
            spf[i] = i;
            primes.push(i);
        }

        for &p in &primes {
            // p が spf[i] を上回る場合、i * p の最小素因数は p ではなく
            // spf[i] になるはずであり、ここで p を使って篩うと不整合が
            // 生じるため打ち切る。i * p が範囲 n を超える場合も同様に打ち切る。
            if p > spf[i] || i * p > n {
                break;
            }
            // このとき i * p の最小素因数は p である。上記の break 条件に
            // より、各合成数はちょうど 1 つの (i, p) の組から篩い落とされる
            // ため、全体の計算量は O(n) に収まる。
            spf[i * p] = p;
        }
    }

    // 0 と 1 はいずれも素因数を持たないが、0 は初期値のままでよい一方、
    // 1 は上のループで確定しないため、番兵として明示的に設定する。
    if n >= 1 {
        spf[1] = 1;
    }

    spf
}

/// `n` 以下の素数を昇順に列挙する。
///
/// # Args
/// - `n` - 列挙する範囲の上限
///
/// # Returns
/// `n` 以下の素数を昇順に格納した `Vec<usize>`。`n < 2` の場合、`n` 以下に
/// 素数は存在しないため空の `Vec` を返す。
///
/// # Complexity
/// - 時間計算量: $O(n)$
///   - 内部で呼び出す [`smallest_prime_factors`] の計算量が支配的である。
/// - 空間計算量: $O(n)$
///   - 内部で構築する最小素因数の篩と、結果として返す素数の一覧を保持する
///     ために、$n$ に比例したメモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::sieve;
///
/// assert_eq!(Vec::<usize>::new(), sieve::primes_up_to(0));
/// assert_eq!(Vec::<usize>::new(), sieve::primes_up_to(1));
/// assert_eq!(vec![2, 3, 5, 7], sieve::primes_up_to(10));
/// ```
#[must_use]
pub fn primes_up_to(n: usize) -> Vec<usize> {
    if n < 2 {
        return Vec::new();
    }

    // spf[i] == i であることは、i がそれより小さいどの素数によっても篩われ
    // なかったこと、すなわち i が素数であることと同値である。
    let spf = smallest_prime_factors(n);
    (2..=n).filter(|&i| spf[i] == i).collect::<Vec<usize>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::primality;

    // smallest_prime_factors のテスト: 戻り値を検証する。
    mod smallest_prime_factors {
        use super::*;

        /// Scenario: `n = 0` の場合、`spf[0] = 0` のみを持つ `Vec` を返す (境界値)。
        /// - Given: `n` が `0` である。
        /// - When: `smallest_prime_factors` を呼ぶ。
        /// - Then: `[0]` が返る。
        #[test]
        fn returns_single_zero_for_n_zero() {
            // Given, When
            let result = smallest_prime_factors(0);
            // Then
            assert_eq!(vec![0], result);
        }

        /// Scenario: `n = 1` の場合、`spf[0] = 0`, `spf[1] = 1` を持つ `Vec` を
        /// 返す (境界値)。
        /// - Given: `n` が `1` である。
        /// - When: `smallest_prime_factors` を呼ぶ。
        /// - Then: `[0, 1]` が返る。
        #[test]
        fn returns_zero_and_one_for_n_one() {
            // Given, When
            let result = smallest_prime_factors(1);
            // Then
            assert_eq!(vec![0, 1], result);
        }

        /// Scenario: 素数の添字には、その素数自身が格納される。
        /// - Given: `n` が複数の素数を含む値である。
        /// - When: `smallest_prime_factors` を呼ぶ。
        /// - Then: 各素数 `p` について `spf[p] == p` が成り立つ。
        #[test]
        fn returns_itself_for_prime_indices() {
            // Given
            let n = 30;
            let primes = [2_usize, 3, 5, 7, 11, 13, 17, 19, 23, 29];

            // When
            let spf = smallest_prime_factors(n);

            // Then
            for p in primes {
                assert_eq!(p, spf[p]);
            }
        }

        /// Scenario: 合成数の添字には、その最小素因数が格納される。
        /// - Given: `n` が複数の合成数を含む値である。
        /// - When: `smallest_prime_factors` を呼ぶ。
        /// - Then: 各合成数について、既知の最小素因数と一致する。
        #[test]
        fn returns_smallest_prime_factor_for_composite_indices() {
            let cases = [(4_usize, 2_usize), (9, 3), (15, 3), (21, 3), (25, 5)];

            for (i, expected) in cases {
                // Given
                let n = 30;

                // When
                let spf = smallest_prime_factors(n);

                // Then
                assert_eq!(expected, spf[i]);
            }
        }
    }

    // primes_up_to のテスト: 戻り値を検証する。
    mod primes_up_to {
        use super::*;

        /// Scenario: `n = 0` の場合、空の `Vec` を返す (境界値)。
        /// - Given: `n` が `0` である。
        /// - When: `primes_up_to` を呼ぶ。
        /// - Then: 空の `Vec` が返る。
        #[test]
        fn returns_empty_vec_for_n_zero() {
            // Given, When
            let result = primes_up_to(0);
            // Then
            assert_eq!(Vec::<usize>::new(), result);
        }

        /// Scenario: `n = 1` の場合、素数が存在しないため空の `Vec` を返す
        /// (境界値)。
        /// - Given: `n` が `1` である。
        /// - When: `primes_up_to` を呼ぶ。
        /// - Then: 空の `Vec` が返る。
        #[test]
        fn returns_empty_vec_for_n_one() {
            // Given, When
            let result = primes_up_to(1);
            // Then
            assert_eq!(Vec::<usize>::new(), result);
        }

        /// Scenario: `n` 以下の素数を昇順に列挙する。
        /// - Given: `n` が典型的な値である。
        /// - When: `primes_up_to` を呼ぶ。
        /// - Then: 既知の素数列と一致する。
        #[test]
        fn returns_ascending_primes_for_typical_n() {
            // Given, When
            let result = primes_up_to(30);
            // Then
            assert_eq!(vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29], result);
        }

        /// Scenario: `n` が素数ちょうどである場合、その値自身を末尾に含む
        /// (境界値)。
        /// - Given: `n` が素数である。
        /// - When: `primes_up_to` を呼ぶ。
        /// - Then: 結果の末尾が `n` と一致する。
        #[test]
        fn includes_n_itself_when_n_is_prime() {
            // Given, When
            let result = primes_up_to(29);
            // Then
            assert_eq!(Some(&29), result.last());
        }

        /// Scenario: `primality::is_prime` による判定結果と一致する。
        /// - Given: `n` が典型的な値である。
        /// - When: `1` から `n` までの各整数について、`primes_up_to` の結果に
        ///   含まれるかどうかと `primality::is_prime` の判定結果を比較する。
        /// - Then: すべての整数について両者が一致する。
        #[test]
        fn matches_primality_is_prime_for_each_integer() {
            // Given
            let n = 100;

            // When
            let primes = primes_up_to(n);

            // Then
            for i in 1..=n {
                let expected = primality::is_prime(i as u64);
                let actual = primes.contains(&i);
                assert_eq!(expected, actual, "mismatch at i = {i}");
            }
        }
    }
}

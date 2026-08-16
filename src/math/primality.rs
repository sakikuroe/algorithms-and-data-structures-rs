//! 素数判定・素因数分解に関連する機能を提供するモジュールである。

use crate::math::modular_arithmetic;
use std::collections::HashMap;
use std::mem;

/// 奇数の法 `m` (`m < 2^62`) に対する、モンゴメリ乗算による高速なモジュラー
/// 演算を提供する。
///
/// `crate::math::modular_arithmetic` の `mul_mod`/`pow_mod` は `u128` への
/// キャストと除算命令によって剰余を求めるが、モンゴメリ乗算はビットシフトと
/// 乗算のみで剰余相当の計算ができるため、`is_prime`/`factorize` のように同じ法
/// のもとで大量のモジュラー演算を繰り返す場面で大きく高速化できる。ただし奇数の
/// 法にしか適用できないため、偶数の法も扱う汎用的な `modular_arithmetic` とは
/// 独立させ、このモジュール内部でのみ用いる。
///
/// 値は「モンゴメリ表現」(`x * 2^64 mod m`) で扱う。[`Montgomery::encode`] で
/// 通常表現から変換する (通常表現へ戻す `decode` は、これまでの利用箇所が
/// いずれもモンゴメリ表現のまま扱えたため用意していない。必要になれば
/// `fma(x, 1, 0)` で計算できる)。速度を優先し、[`Montgomery::fma`]/
/// [`Montgomery::mul`] の結果は `[0, m)` へ正規化せず `[0, 2m)` の範囲のまま
/// 返す。そのため、モンゴメリ表現の値同士を比較する際は単純な `==` ではなく
/// [`Montgomery::eq`] を使う必要がある。
struct Montgomery {
    /// 奇数の法である。
    m: u64,
    /// `m * m_inv ≡ 1 (mod 2^64)` を満たす値であり、モンゴメリ簡約で使う。
    m_inv: u64,
    /// `2^128 mod m` であり、通常表現からモンゴメリ表現への変換で使う。
    r2: u64,
}

impl Montgomery {
    fn new(m: u64) -> Self {
        debug_assert!(m % 2 == 1);
        debug_assert!(m < (1 << 62));

        // ニュートン法により m * m_inv ≡ 1 (mod 2^64) を満たす m_inv を求める。
        // 1 を初期値とし、反復のたびに正しい下位ビット数が倍化する
        // (1, 2, 4, 8, 16, 32, 64 ビットの精度で収束するため、64 ビットに
        // 到達するには 6 回の反復で十分である)。
        let mut m_inv = 1_u64;
        for _ in 0..6 {
            m_inv = m_inv.wrapping_mul(2u64.wrapping_sub(m.wrapping_mul(m_inv)));
        }

        let r_mod_m = ((1_u128 << 64) % m as u128) as u64;
        let r2 = modular_arithmetic::mul_mod(r_mod_m, r_mod_m, m);

        Montgomery { m, m_inv, r2 }
    }

    /// `a * b / 2^64 + c (mod m)` を計算する融合乗算加算 (fused multiply-add)。
    ///
    /// `a`, `b` は `[0, 2m)`、`c` は `[0, m)` の範囲である前提とする。結果は
    /// `[0, 2m)` の範囲になり、正規化は行わない。乗算と加算を 1 回の簡約に
    /// まとめ、かつ結果を毎回 `[0, m)` へ正規化しないことで、`u128` の比較や
    /// 減算を伴わない `u64` の `wrapping_*` 演算のみで完結させている。
    ///
    /// `a, b < 2m` より `a * b < 4m^2` であり、これが `(m - c) * 2^64` 未満で
    /// あれば `a * b + c * 2^64 < m * 2^64` となり、簡約結果は理論上
    /// `[0, 2m)` に収まる。`c = 0` で呼ぶ [`Montgomery::mul`] ではこの条件は
    /// `4m^2 < m * 2^64` すなわち `m < 2^62` に単純化でき、[`Montgomery::new`]
    /// がこの上限を強制しているのはこのためである。`c` が `0` でない一般の
    /// `fma` 呼び出しでは、`c` が `m` に近いほど条件は厳しくなる点に注意する。
    #[inline(always)]
    fn fma(&self, a: u64, b: u64, c: u64) -> u64 {
        debug_assert!(a < self.m * 2);
        debug_assert!(b < self.m * 2);
        debug_assert!(c < self.m);

        let t = a as u128 * b as u128;
        let tc = ((t >> 64) as u64).wrapping_add(c);
        let q = (t as u64).wrapping_mul(self.m_inv);
        let qm = ((q as u128 * self.m as u128) >> 64) as u64;
        tc.wrapping_sub(qm).wrapping_add(self.m)
    }

    /// モンゴメリ表現の値同士の乗算を行う。
    #[inline(always)]
    fn mul(&self, a: u64, b: u64) -> u64 {
        self.fma(a, b, 0)
    }

    /// 通常表現の値 (`< m`) をモンゴメリ表現へ変換する。
    #[inline(always)]
    fn encode(&self, x: u64) -> u64 {
        self.mul(x, self.r2)
    }

    /// `[0, 2m)` の範囲にある 2 つのモンゴメリ表現の値が、mod `m` で等しいか
    /// どうかを判定する。正規化していない値同士は、差が `0` または `m` の
    /// いずれかであれば mod `m` で等しい。
    #[inline(always)]
    fn eq(&self, a: u64, b: u64) -> bool {
        let d = a.abs_diff(b);
        d == 0 || d == self.m
    }

    /// モンゴメリ表現の `base` を、通常表現の指数 `exp` で累乗する。
    fn pow(&self, base: u64, mut exp: u64) -> u64 {
        let mut result = self.encode(1);
        let mut base = base;
        while exp > 0 {
            if exp & 1 == 1 {
                result = self.mul(result, base);
            }
            exp >>= 1;
            if exp > 0 {
                base = self.mul(base, base);
            }
        }
        result
    }
}

/// 決定的 Miller-Rabin 法により、`n` が素数かどうかを判定する。
///
/// # Args
/// - `n` - 判定対象の非負整数
///
/// # Returns
/// `n` が素数であれば `true`、そうでなければ `false`。
///
/// # Complexity
/// - 時間計算量: $O(\log^2 n)$
///   - 基底 `[2, 325, 9375, 28178, 450775, 9780504, 1795265022]` の 7 個それぞれについて
///     $O(\log n)$ 回のモジュラー乗算を行う。この基底は `u64` の全域で決定的に
///     正しい結果を返すことが知られているが、`n` が `4759123141` 未満の場合は
///     基底 `[2, 7, 61]` の 3 個で決定的な判定に十分であることも知られており、
///     その場合はこちらを用いて基底数を減らす。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert!(!primality::is_prime(0));
/// assert!(!primality::is_prime(1));
/// assert!(primality::is_prime(2));
/// assert!(!primality::is_prime(4));
/// assert!(primality::is_prime(998244353));
/// ```
#[must_use]
pub fn is_prime(n: u64) -> bool {
    if n == 0 || n == 1 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    // n がこの値未満であれば、基底 [2, 7, 61] の 3 個だけで決定的に判定できる
    // ことが知られている。
    const SMALL_WITNESSES_THRESHOLD: u64 = 4_759_123_141;
    const SMALL_WITNESSES: [u64; 3] = [2, 7, 61];
    const FULL_WITNESSES: [u64; 7] = [2, 325, 9375, 28178, 450775, 9780504, 1795265022];

    let witnesses: &[u64] = if n < SMALL_WITNESSES_THRESHOLD {
        &SMALL_WITNESSES
    } else {
        &FULL_WITNESSES
    };

    // Montgomery は m < 2^62 のみに対応するため、それ以上の n では
    // modular_arithmetic の汎用実装に切り替える。u64 全域のうち n >= 2^62 と
    // なる割合は 4 分の 3 に上るが、Library Checker など実用上扱う入力の
    // 多くは 10^18 (< 2^60) 程度までであり、高速な経路を通ることが多い。
    if n < (1 << 62) {
        is_prime_by_montgomery(n, witnesses)
    } else {
        is_prime_by_fallback(n, witnesses)
    }
}

/// `n < 2^62` の場合に用いる、モンゴメリ乗算による高速な Miller-Rabin 法。
fn is_prime_by_montgomery(n: u64, witnesses: &[u64]) -> bool {
    // n - 1 = 2^s * d (d は奇数) と分解する。
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;

    let mont = Montgomery::new(n);
    let one = mont.encode(1);
    let minus_one = mont.encode(n - 1);

    // 基底 a が n の合成数性の証拠にならない (= n が合成数であってもこの a だけでは
    // 見抜けない) 場合に true を返す。a^d ≡ 1 または a^(2^r * d) ≡ -1 (mod n) と
    // なる 0 <= r < s が存在するなら、a はフェルマーテストを ("合成数の証拠なし" の
    // 意味で) 通過する。
    witnesses.iter().all(|&a| {
        let a = a % n;
        if a == 0 {
            return true;
        }

        let mut x = mont.pow(mont.encode(a), d);
        if mont.eq(x, one) || mont.eq(x, minus_one) {
            return true;
        }

        for _ in 1..s {
            x = mont.mul(x, x);
            if mont.eq(x, minus_one) {
                return true;
            }
        }

        false
    })
}

/// `n >= 2^62` の場合に用いる、`modular_arithmetic` の汎用実装による
/// Miller-Rabin 法。判定のロジックは [`is_prime_by_montgomery`] と同じである。
fn is_prime_by_fallback(n: u64, witnesses: &[u64]) -> bool {
    let s = (n - 1).trailing_zeros();
    let d = (n - 1) >> s;

    witnesses.iter().all(|&a| {
        let a = a % n;
        if a == 0 {
            return true;
        }

        let mut x = modular_arithmetic::pow_mod(a, d, n);
        if x == 1 || x == n - 1 {
            return true;
        }

        for _ in 1..s {
            x = modular_arithmetic::mul_mod(x, x, n);
            if x == n - 1 {
                return true;
            }
        }

        false
    })
}

/// 試し割り法によって、閾値未満の合成数 `n` を素因数分解する。
fn factorize_by_trial_division(mut n: u64) -> HashMap<u64, usize> {
    let mut result = HashMap::new();

    let mut p = 2;
    while p * p <= n {
        while n % p == 0 {
            *result.entry(p).or_insert(0) += 1;
            n /= p;
        }
        p += 1;
    }
    if n > 1 {
        *result.entry(n).or_insert(0) += 1;
    }

    result
}

/// `u64` 同士の最大公約数を、除算命令を使わずに求める (Stein のアルゴリズム、
/// バイナリ GCD)。
///
/// `number_theory::gcd` はユークリッドの互除法により `%` (除算命令) で剰余を
/// 求めるが、除算命令はシフトや乗算に比べて何倍も重い。バイナリ GCD は
/// 「両方偶数なら 2 で割って `2 * gcd(x/2, y/2)`」「片方だけ偶数ならその値だけ
/// 2 で割ってよい」「両方奇数なら差を取ると必ず偶数になるので次のステップに
/// 戻れる」という 3 つの性質だけで計算でき、右シフト・比較・引き算のみで
/// 完結する。`find_divisor` はバッチのたびに `gcd` を頻繁に呼ぶため、
/// `number_theory::gcd` を書き換えず、この関数だけをここで使う。
fn gcd_binary(mut x: u64, mut y: u64) -> u64 {
    if x == 0 {
        return y;
    }
    if y == 0 {
        return x;
    }

    // x, y に共通する 2 の因数を先に取り除いておき、最後に掛け戻す。
    let common_twos = (x | y).trailing_zeros();
    x >>= x.trailing_zeros();

    // 以降、x は常に奇数に保つ。y は毎回先頭で奇数化してから x と比較し、
    // 大きい方から小さい方を引く (差は必ず偶数になり、次のループの先頭で
    // 再び奇数化される)。x == y になった時点で、それが (2 の共通因数を除いた)
    // 最大公約数である。
    loop {
        y >>= y.trailing_zeros();
        if x == y {
            break;
        }
        if x > y {
            mem::swap(&mut x, &mut y);
        }
        y -= x;
    }

    x << common_twos
}

/// Pollard's rho 法 (Brent のサイクル検出) により、合成数 `n` の非自明な約数を
/// 1 つ見つける。
///
/// Floyd のサイクル検出 (「亀と兎」を毎ステップ動かす方式) は 1 ステップあたり
/// 3 回の関数評価 (`f(x)` を 1 回、`f(f(y))` で 2 回) を要するのに対し、Brent の
/// 方式は一定間隔で固定する参照点 `x` との差分を追跡することで、1 ステップ
/// あたり 1 回の関数評価で済む。さらに、素朴な実装が `gcd(|x - y|, n)` を毎
/// ステップ計算するのに対し、`gcd(a, n) = 1` かつ `gcd(b, n) = 1` ならば
/// `gcd(a * b, n) = 1` であることを利用し、差分の積を `BATCH` ステップぶん
/// 蓄積してから `gcd` を 1 回だけ計算することで、`gcd` の呼び出し回数を
/// `1 / BATCH` に減らす。
fn find_divisor(n: u64) -> u64 {
    if n % 2 == 0 {
        return 2;
    }

    // Montgomery は m < 2^62 のみに対応するため、それ以上の n では
    // modular_arithmetic の汎用実装に切り替える。u64 全域のうち n >= 2^62 と
    // なる割合は 4 分の 3 に上るが、Library Checker など実用上扱う入力の
    // 多くは 10^18 (< 2^60) 程度までであり、高速な経路を通ることが多い。
    if n < (1 << 62) {
        find_divisor_by_montgomery(n)
    } else {
        find_divisor_by_fallback(n)
    }
}

/// `n < 2^62` の場合に用いる、モンゴメリ乗算による高速な Pollard's rho 法
/// (Brent のサイクル検出)。
fn find_divisor_by_montgomery(n: u64) -> u64 {
    // gcd の呼び出し回数と、約数の混入位置を後から逐次探索するコストとの
    // バランスを取るための、経験的に妥当なバッチサイズである。
    const BATCH: usize = 512;

    let mont = Montgomery::new(n);

    // c を変えながら繰り返す。1 つの c で約数が見つからなかった場合は、
    // 別の c (別の多項式 f(a) = a^2 + c) で仕切り直す。c は encode せず生の
    // 値のまま使う。多項式定数の「意味」がモンゴメリ表現としてズレても、
    // どのみち c は数列にサイクル構造を作れれば十分な値であり、最終的な
    // gcd の結果には影響しない (gcd_binary に渡す直前の値についても同様)。
    for c in 1..n {
        let f = |a: u64| -> u64 { mont.fma(a, a, c) };

        // x: 一定間隔 (バッチ境界) で更新する参照点。
        // y: 毎ステップ f を適用して進めていく点。
        // ys: 直近のバッチ開始時点の y の値。バッチ全体では gcd が n に退化
        //     した場合に、この位置から 1 ステップずつ確認し直すための
        //     巻き戻し地点として使う。
        let (mut x, mut y, mut ys) = (0_u64, 0_u64, 0_u64);
        // g: 直近に計算した gcd。1 である間は探索を続ける。
        // product: バッチ内で蓄積する差分の積 (モンゴメリ表現のまま)。
        // r: 参照点 x を切り替えるまでに進む合計ステップ数で、バッチが尽きる
        //    たびに倍化していく。
        // k: 現在の参照点のもとで、これまでに進んだステップ数。
        let (mut g, mut product, mut r, mut k) = (1_u64, 1_u64, 1_usize, 0_usize);

        while g == 1 {
            x = y;
            while k < r && g == 1 {
                ys = y;
                for _ in 0..BATCH.min(r - k) {
                    y = f(y);
                    // x, y はいずれも [0, 2n) の範囲のモンゴメリ表現の値
                    // (Montgomery::fma は正規化を行わない) だが、差の絶対値
                    // は [0, 2n) に収まるため、そのまま mul の入力に使える。
                    product = mont.mul(product, x.abs_diff(y));
                }
                // モンゴメリの基数 R (= 2^64) は奇数の n と互いに素であるため、
                // gcd(q * R mod n, n) == gcd(q, n) が成り立つ。decode (除算を
                // 伴う) を経由せず、モンゴメリ表現のまま gcd を取ってよい。
                g = gcd_binary(product, n);
                k += BATCH;
            }
            k = r;
            r <<= 1;
        }

        if g == n {
            // バッチ単位の gcd が n に退化した場合、直近のバッチ開始地点まで
            // 巻き戻し、1 ステップずつ gcd を確認して混入した位置を特定する。
            g = 1;
            y = ys;
            while g == 1 {
                y = f(y);
                g = gcd_binary(x.abs_diff(y), n);
            }
        }

        if g != n {
            return g;
        }
    }

    unreachable!()
}

/// `n >= 2^62` の場合に用いる、`modular_arithmetic` の汎用実装による
/// Pollard's rho 法 (Floyd の閉路検出)。
///
/// 素朴な実装では `gcd(|x - y|, n)` をステップごとに計算するが、`gcd` は除算を
/// 伴い比較的重い演算である。`gcd(a, n) = 1` かつ `gcd(b, n) = 1` ならば
/// `gcd(a * b, n) = 1` であることを利用し、差分の積を `BATCH` ステップぶん
/// 蓄積してから `gcd` を 1 回だけ計算することで、`gcd` の呼び出し回数を
/// `1 / BATCH` に減らす。
fn find_divisor_by_fallback(n: u64) -> u64 {
    const BATCH: usize = 128;

    // c を変えながら繰り返す。1 つの c で閉路検出が n 自身に退化した (d == n) 場合は
    // 別の c で仕切り直す。
    for c in 1.. {
        let c = c % n;
        let f = |x: u64| -> u64 {
            modular_arithmetic::add_mod(modular_arithmetic::mul_mod(x, x, n), c, n)
        };

        let mut x = 2;
        let mut y = 2;
        let mut product = 1;

        loop {
            // このバッチで計算した差分の積の履歴を保持しておき、gcd(積, n) が
            // 1 でなくなった場合に、その中のどのステップで約数が混入したかを
            // 逐次探索で特定する。
            let mut history = Vec::with_capacity(BATCH);
            for _ in 0..BATCH {
                x = f(x);
                y = f(f(y));
                product = modular_arithmetic::mul_mod(product, x.abs_diff(y), n);
                history.push(product);
            }

            if gcd_binary(product, n) == 1 {
                continue;
            }

            let d = history
                .into_iter()
                .map(|p| gcd_binary(p, n))
                .find(|&d| d != 1)
                .unwrap();

            if d != n {
                return d;
            }
            break;
        }
    }

    unreachable!()
}

/// `n` を素因数分解する。
///
/// # Args
/// - `n` - 素因数分解の対象となる非負整数
///
/// # Returns
/// `n` の素因数分解を、素因数からその指数への `HashMap<u64, usize>` として返す。
/// 具体的には:
/// - `n = 0` または `n = 1` の場合、空の `HashMap` を返す。
/// - `n >= 2` の場合、`n == product(p.pow(e) for (p, e) in result)` を満たす。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n)$
///   - `n` が試し割りで素早く分解できる程度に小さい場合は $O(\sqrt{n})$ の試し割り法を、
///     そうでない場合は [`is_prime`] と Pollard's rho 法を組み合わせて分解する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
/// use std::collections::HashMap;
///
/// assert_eq!(HashMap::new(), primality::factorize(1));
/// assert_eq!(HashMap::from([(2, 2), (3, 1)]), primality::factorize(12));
/// assert_eq!(HashMap::from([(17, 1)]), primality::factorize(17));
/// ```
#[must_use]
pub fn factorize(n: u64) -> HashMap<u64, usize> {
    // 試し割り法が O(sqrt(n)) の時間で十分実用的に収まる閾値である。
    const TRIAL_DIVISION_THRESHOLD: u64 = 1_000_000_000;
    // Pollard's rho に入る前に、この値未満の素因数を試し割りで先に取り除く
    // 閾値である。Pollard's rho は大きな素因数の発見は得意だが、小さな
    // 素因数の発見は試し割りに劣るため、事前に取り除いておくことで
    // find_divisor (ひいては Montgomery のセットアップや Brent の探索) の
    // 呼び出し回数を減らせる。ただし閾値を上げすぎると、この試し割り自体の
    // コストが上回る。Library Checker `factorize` の全テストケースで計測した
    // ところ、閾値なしで約 117ms、1000 では約 124ms (悪化) だったのに対し、
    // 40 前後が約 111ms と最も速かった。
    const SMALL_PRIME_LIMIT: u64 = 40;

    if n == 0 || n == 1 {
        return HashMap::new();
    }
    if is_prime(n) {
        return HashMap::from([(n, 1)]);
    }
    if n < TRIAL_DIVISION_THRESHOLD {
        return factorize_by_trial_division(n);
    }

    let mut result = HashMap::new();
    let n = extract_small_prime_factors(n, SMALL_PRIME_LIMIT, &mut result);

    if n == 1 {
        return result;
    }
    if is_prime(n) {
        *result.entry(n).or_insert(0) += 1;
        return result;
    }

    let mut composites = vec![n];

    while let Some(m) = composites.pop() {
        let d = find_divisor(m);
        for divisor in [d, m / d] {
            if is_prime(divisor) {
                *result.entry(divisor).or_insert(0) += 1;
            } else {
                composites.push(divisor);
            }
        }
    }

    result
}

/// `n` から `limit` 未満の素因数を試し割りで取り除き、`result` に積算した
/// うえで、取り除いた後に残った値を返す。
fn extract_small_prime_factors(mut n: u64, limit: u64, result: &mut HashMap<u64, usize>) -> u64 {
    let mut p = 2;
    while p < limit && p * p <= n {
        while n % p == 0 {
            *result.entry(p).or_insert(0) += 1;
            n /= p;
        }
        p += 1;
    }
    n
}

/// `n` の正の約数を昇順に列挙する。
///
/// # Args
/// - `n` - 約数を求める対象の整数であり、`0` より大きい必要がある。
///
/// # Returns
/// `n` の約数を昇順に格納した `Vec<u64>`。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n + d(n))$
///   - 内部で呼び出す [`factorize`] の計算量が支配的であり、素因数分解の結果から
///     約数を列挙する処理は約数の個数 $d(n)$ に比例した時間で行える。
/// - 空間計算量: $O(d(n))$
///   - 列挙された約数を保持するために、約数の個数に比例したメモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(vec![1], primality::divisors(1));
/// assert_eq!(vec![1, 7], primality::divisors(7));
/// assert_eq!(vec![1, 2, 3, 4, 6, 12], primality::divisors(12));
/// ```
#[must_use]
pub fn divisors(n: u64) -> Vec<u64> {
    debug_assert!(n >= 1);

    // 約数の列挙結果を蓄積するためのベクタ。初期状態では、どんな n に対しても
    // 約数である 1 のみを持つ。
    let mut result = vec![1_u64];

    // 素因数 p とその指数 e ごとに、それまでに求めた約数それぞれに
    // p^0, p^1, ..., p^e を掛け合わせた値を、新たな約数集合として構築していく。
    for (p, e) in factorize(n) {
        let mut next = Vec::with_capacity(result.len() * (e + 1));
        let mut power = 1_u64;
        for _ in 0..=e {
            for &d in &result {
                next.push(d * power);
            }
            power *= p;
        }
        result = next;
    }

    result.sort_unstable();
    result
}

/// オイラーの $\varphi$ 関数の値を計算する。
///
/// # Args
/// - `n` - 計算対象の整数であり、`0` より大きい必要がある。
///
/// # Returns
/// `1` から `n` までの整数のうち、`n` と互いに素であるものの個数。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(n^{1/4} \log n)$
///   - 内部で呼び出す [`factorize`] の計算量が支配的である。
/// - 空間計算量: $O(\log n)$
///   - `n` が持つ相異なる素因数の個数に比例したメモリを使用する。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(1, primality::euler_phi(1));
/// assert_eq!(6, primality::euler_phi(9));
/// assert_eq!(16, primality::euler_phi(17));
/// ```
#[must_use]
pub fn euler_phi(n: u64) -> u64 {
    debug_assert!(n >= 1);

    let mut result = n;
    for p in factorize(n).into_keys() {
        // 先に n を p で割ってから (p - 1) を掛けることで、掛け算を先に行う場合に
        // 生じうる u64 のオーバーフローを避ける。result が p で割り切れることは、
        // p が factorize(n) の素因数であることから数学的に保証されている。
        result = result / p * (p - 1);
    }
    result
}

/// `p` を法として `a` が原始根であるかどうかを判定する。
///
/// # Args
/// - `a` - 判定対象の整数であり、`0 <= a < p` を満たす必要がある。
/// - `p` - 法であり、素数である必要がある。この契約に違反する場合、
///   `debug_assert!` によりパニックする。
///
/// # Returns
/// `a` が `p` を法とする原始根であれば `true`、そうでなければ `false`。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(p^{1/4} \log p)$
///   - 内部で呼び出す [`factorize`] の計算量が支配的であり、`p - 1` の相異なる
///     素因数それぞれについて [`modular_arithmetic::pow_mod`] を1回ずつ呼び出す。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert!(primality::is_primitive_root(3, 7));
/// assert!(!primality::is_primitive_root(2, 7));
/// ```
#[must_use]
pub fn is_primitive_root(a: u64, p: u64) -> bool {
    debug_assert!(is_prime(p));
    debug_assert!(a < p);

    // p = 2 の場合、乗法群 (Z/2Z)^* の位数は 1 であり、唯一の元である 1 が
    // 原始根となる。
    if p == 2 {
        return a == 1;
    }

    // a が原始根であることは、a の位数 (乗法群における周期) が p - 1 と一致する
    // ことと同値である。これはさらに、p - 1 の相異なる素因数 q それぞれについて
    // a^((p-1)/q) != 1 (mod p) が成り立つことと同値であるため、この条件を判定する。
    factorize(p - 1)
        .into_keys()
        .all(|q| modular_arithmetic::pow_mod(a, (p - 1) / q, p) != 1)
}

/// `p` を法とする原始根を1つ見つける。
///
/// # Args
/// - `p` - 法であり、素数である必要がある。この契約に違反する場合、
///   `debug_assert!` によりパニックする。
///
/// # Returns
/// `p` を法とする原始根の1つ。最小のものであるとは限らない。
///
/// # Complexity
/// - 時間計算量: 期待値 $O(p^{1/4} \log^2 p)$
///   - 最小の原始根は $O(\log \log p)$ 個程度の候補を試せば見つかることが経験的に
///     知られており、候補ごとに [`is_primitive_root`] による判定を行う。
///
/// # Examples
/// ```
/// use anmitsu::math::primality;
///
/// assert_eq!(1, primality::find_primitive_root(2));
/// assert!(primality::is_primitive_root(primality::find_primitive_root(7), 7));
/// ```
#[must_use]
pub fn find_primitive_root(p: u64) -> u64 {
    debug_assert!(is_prime(p));

    if p == 2 {
        return 1;
    }

    // 小さい候補から順に原始根であるかを判定し、最初に見つかったものを返す。
    for a in 2.. {
        if is_primitive_root(a, p) {
            return a;
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // is_prime のテスト: 戻り値を検証する。
    mod is_prime {
        use super::*;

        /// Scenario: `0` と `1` は素数ではない (境界値)。
        /// - Given: `0` と `1` がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_zero_and_one() {
            let cases = [0_u64, 1_u64];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: `2` は素数である (境界値)。
        /// - Given: `2` がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_two() {
            // Given, When
            let result = is_prime(2);
            // Then
            assert!(result);
        }

        /// Scenario: 典型的な素数に対して `true` を返す。
        /// - Given: 典型的な素数がいくつかある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_typical_primes() {
            let cases = [3_u64, 7, 13, 97, 998244353];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(result);
            }
        }

        /// Scenario: 偶数の合成数に対して `false` を返す。
        /// - Given: `2` より大きい偶数がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_even_composites() {
            let cases = [4_u64, 100, 998244352];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: フェルマーテストを誤って通過しやすいカーマイケル数に対して `false` を返す。
        /// - Given: カーマイケル数がいくつかある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_carmichael_numbers() {
            let cases = [561_u64, 1105, 1729, 2465, 41041];

            for n in cases {
                // Given, When
                let result = is_prime(n);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: `u64` の範囲に収まる大きな素数・合成数でも正しく判定できる (境界値)。
        /// - Given: `u64` の範囲に収まる大きな素数と、その素数同士の積である合成数がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: 期待した判定結果が返る。
        #[test]
        fn returns_correct_result_for_large_numbers() {
            // Given
            // 18446744073709551557 は u64::MAX 未満の最大の素数である。
            let large_prime = 18446744073709551557_u64;
            // 4295098369 = 65537 * 65537 は大きな平方数の合成数である。
            let large_composite = 4295098369_u64;

            // When
            let prime_result = is_prime(large_prime);
            let composite_result = is_prime(large_composite);

            // Then
            assert!(prime_result);
            assert!(!composite_result);
        }

        /// Scenario: `[2^62, 2^63)` の範囲にある素数を正しく判定できる (境界値)。
        /// - Given: `[2^62, 2^63)` の範囲にある素数がある。
        /// - When: `is_prime` を呼ぶ。
        /// - Then: `true` が返る。
        ///
        /// この範囲は、モンゴメリ乗算の適用上限を誤って `2^63` としていた際に
        /// 誤って `false` を返していた区間である (`m < 2^62` が正しい上限)。
        #[test]
        fn returns_true_for_primes_in_2_pow_62_to_2_pow_63() {
            // Given
            let n = 7094011965265554437_u64;
            assert!(n >= 1 << 62 && n < 1 << 63);

            // When
            let result = is_prime(n);

            // Then
            assert!(result);
        }
    }

    // factorize のテスト: 戻り値を検証する。
    mod factorize {
        use super::*;

        /// Scenario: `0` と `1` は空の `HashMap` を返す (境界値)。
        /// - Given: `0` と `1` がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 空の `HashMap` が返る。
        #[test]
        fn returns_empty_map_for_zero_and_one() {
            let cases = [0_u64, 1_u64];

            for n in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(HashMap::new(), result);
            }
        }

        /// Scenario: 素数はそれ自身を指数 `1` として返す (境界値)。
        /// - Given: 素数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: `{n: 1}` が返る。
        #[test]
        fn returns_itself_for_prime_numbers() {
            let cases = [2_u64, 17, 998244353];

            for n in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(HashMap::from([(n, 1)]), result);
            }
        }

        /// Scenario: 典型的な合成数を試し割り法の範囲内で正しく素因数分解する。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 期待した素因数分解が返る。
        #[test]
        fn returns_correct_factorization_for_typical_values() {
            let cases = [
                (12_u64, HashMap::from([(2, 2), (3, 1)])),
                (100, HashMap::from([(2, 2), (5, 2)])),
                (360, HashMap::from([(2, 3), (3, 2), (5, 1)])),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 完全平方数を正しく素因数分解する。
        /// - Given: 素数の平方である完全平方数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: `{p: 2}` が返る。
        #[test]
        fn returns_correct_factorization_for_perfect_squares() {
            // Given
            // 1000003 は素数であり、r * r は試し割りの閾値を超える。
            let p = 1000003_u64;
            let n = p * p;

            // When
            let result = factorize(n);

            // Then
            assert_eq!(HashMap::from([(p, 2)]), result);
        }

        /// Scenario: 試し割りの閾値を超える大きな合成数を、Pollard's rho 法の経路で
        /// 正しく素因数分解する (境界値)。
        /// - Given: 2 つの大きな素数の積である合成数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 期待した素因数分解が返る。
        #[test]
        fn returns_correct_factorization_via_pollards_rho_for_large_semiprimes() {
            let cases = [
                // 1000000007, 1000000009 はともに大きな素数である。
                (
                    1000000007_u64 * 1000000009,
                    HashMap::from([(1000000007, 1), (1000000009, 1)]),
                ),
                // 999999999000000007 は 1e18 級の大きな素数である。
                (
                    999999999000000007_u64 * 2,
                    HashMap::from([(2, 1), (999999999000000007, 1)]),
                ),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = factorize(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: `2^62` 以上の大きな素因数を含む合成数でも、モンゴメリ乗算
        /// の適用範囲外として正しく処理できる (境界値)。
        /// - Given: `2^62` 以上の大きな素数と `2` の積である合成数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: 期待した素因数分解が返る。
        #[test]
        fn returns_correct_factorization_for_numbers_at_or_above_2_pow_62() {
            // Given
            // 4611686018427388039 は 2^62 を超える大きな素数である。
            let p = 4611686018427388039_u64;
            let n = p * 2;
            assert!(p >= 1 << 62);

            // When
            let result = factorize(n);

            // Then
            assert_eq!(HashMap::from([(2, 1), (p, 1)]), result);
        }

        /// Scenario: `[2^62, 2^63)` の範囲にある素数を正しく素因数分解する (境界値)。
        /// - Given: `[2^62, 2^63)` の範囲にある素数がある。
        /// - When: `factorize` を呼ぶ。
        /// - Then: `{n: 1}` が返る。
        ///
        /// この範囲は、モンゴメリ乗算の適用上限を誤って `2^63` としていた際に
        /// `is_prime` が誤って `false` を返し、結果として Pollard's rho が
        /// 存在しない非自明な約数を探し続けてハングしていた区間である。
        #[test]
        fn returns_itself_for_primes_in_2_pow_62_to_2_pow_63() {
            // Given
            let n = 7094011965265554437_u64;
            assert!(n >= 1 << 62 && n < 1 << 63);

            // When
            let result = factorize(n);

            // Then
            assert_eq!(HashMap::from([(n, 1)]), result);
        }
    }

    // divisors のテスト: 戻り値を検証する。
    mod divisors {
        use super::*;

        /// Scenario: 典型的な合成数に対して、昇順に並んだ約数の一覧を返す。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: 期待した約数の一覧が昇順で返る。
        #[test]
        fn returns_sorted_divisors_for_typical_values() {
            let cases = [
                (12_u64, vec![1_u64, 2, 3, 4, 6, 12]),
                (
                    360,
                    vec![
                        1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 18, 20, 24, 30, 36, 40, 45, 60, 72, 90,
                        120, 180, 360,
                    ],
                ),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = divisors(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 素数の約数は `1` と自分自身のみになる。
        /// - Given: 素数がいくつかある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: `[1, n]` が返る。
        #[test]
        fn returns_one_and_itself_for_prime_numbers() {
            let cases = [7_u64, 17, 998244353];

            for n in cases {
                // Given, When
                let result = divisors(n);
                // Then
                assert_eq!(vec![1, n], result);
            }
        }

        /// Scenario: `1` の約数は `1` のみになる (境界値)。
        /// - Given: `1` がある。
        /// - When: `divisors` を呼ぶ。
        /// - Then: `[1]` が返る。
        #[test]
        fn returns_singleton_for_one() {
            // Given, When
            let result = divisors(1);
            // Then
            assert_eq!(vec![1], result);
        }
    }

    // euler_phi のテスト: 戻り値を検証する。
    mod euler_phi {
        use super::*;

        /// Scenario: 典型的な合成数に対して正しい `φ` の値を返す。
        /// - Given: 複数の素因数からなる典型的な合成数がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: 期待した `φ` の値が返る。
        #[test]
        fn returns_correct_value_for_typical_values() {
            let cases = [(12_u64, 4_u64), (9, 6)];

            for (n, expected) in cases {
                // Given, When
                let result = euler_phi(n);
                // Then
                assert_eq!(expected, result);
            }
        }

        /// Scenario: 素数 `p` に対しては `p - 1` を返す。
        /// - Given: 素数がいくつかある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: `p - 1` が返る。
        #[test]
        fn returns_predecessor_for_prime_numbers() {
            let cases = [17_u64, 998244353];

            for p in cases {
                // Given, When
                let result = euler_phi(p);
                // Then
                assert_eq!(p - 1, result);
            }
        }

        /// Scenario: `1` に対しては `1` を返す (境界値)。
        /// - Given: `1` がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_for_one() {
            // Given, When
            let result = euler_phi(1);
            // Then
            assert_eq!(1, result);
        }

        /// Scenario: 試し割りの閾値を超える大きな `n` でも、オーバーフローせずに
        /// 正しい `φ` の値を返す (境界値)。
        /// - Given: `factorize` が Pollard's rho 法の経路を通るような、大きな
        ///   素因数からなる合成数がある。
        /// - When: `euler_phi` を呼ぶ。
        /// - Then: 期待した `φ` の値が返る。
        #[test]
        fn returns_correct_value_without_overflow_for_large_numbers() {
            let cases = [
                // 1000000007, 1000000009 はともに大きな素数であり、
                // φ(p * q) = (p - 1) * (q - 1) である。
                (1000000007_u64 * 1000000009, 1000000006_u64 * 1000000008),
                // 999999999000000007 は 1e18 級の大きな素数であり、
                // φ(2 * p) = p - 1 である。
                (999999999000000007_u64 * 2, 999999999000000006_u64),
            ];

            for (n, expected) in cases {
                // Given, When
                let result = euler_phi(n);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // is_primitive_root のテスト: 戻り値を検証する。
    mod is_primitive_root {
        use super::*;

        /// Scenario: 原始根に対して `true` を返す。
        /// - Given: 素数 `7` を法とする原始根 (位数が `6` となる元) がある。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `true` が返る。
        #[test]
        fn returns_true_for_primitive_roots() {
            let cases = [3_u64, 5];

            for a in cases {
                // Given, When
                let result = is_primitive_root(a, 7);
                // Then
                assert!(result);
            }
        }

        /// Scenario: 原始根でない元に対して `false` を返す。
        /// - Given: 素数 `7` を法とする、原始根でない元 (位数が `6` 未満となる元) が
        ///   いくつかある。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `false` が返る。
        #[test]
        fn returns_false_for_non_primitive_roots() {
            let cases = [1_u64, 2, 4, 6];

            for a in cases {
                // Given, When
                let result = is_primitive_root(a, 7);
                // Then
                assert!(!result);
            }
        }

        /// Scenario: 法が `2` の場合、`1` のみが原始根と判定される (境界値)。
        /// - Given: 法が `2` であり、`a` が `0` または `1` である。
        /// - When: `is_primitive_root` を呼ぶ。
        /// - Then: `a` が `1` のときのみ `true` が返る。
        #[test]
        fn returns_true_only_for_one_when_modulus_is_two() {
            let cases = [(0_u64, false), (1, true)];

            for (a, expected) in cases {
                // Given, When
                let result = is_primitive_root(a, 2);
                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // find_primitive_root のテスト: 戻り値を検証する。
    mod find_primitive_root {
        use super::*;

        /// Scenario: 見つけた元が、実際に原始根としての性質を満たす。
        /// - Given: 素数がいくつかある。
        /// - When: `find_primitive_root` を呼ぶ。
        /// - Then: 返った元が `is_primitive_root` で `true` と判定される。
        #[test]
        fn returns_value_satisfying_is_primitive_root_property() {
            let cases = [7_u64, 13, 998244353];

            for p in cases {
                // Given, When
                let result = find_primitive_root(p);
                // Then
                assert!(is_primitive_root(result, p));
            }
        }

        /// Scenario: 法が `2` の場合、`1` を返す (境界値)。
        /// - Given: 法が `2` である。
        /// - When: `find_primitive_root` を呼ぶ。
        /// - Then: `1` が返る。
        #[test]
        fn returns_one_when_modulus_is_two() {
            // Given, When
            let result = find_primitive_root(2);
            // Then
            assert_eq!(1, result);
        }
    }
}

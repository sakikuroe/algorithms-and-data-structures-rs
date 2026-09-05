//! 素数 998244353 を法とするモジュラー整数の実装を提供するモジュールである。

use std::{fmt, ops};

/// モジュラー演算の法である。
const MOD: u32 = 998244353; // 998244353 = 119 * 2^23 + 1 という NTT フレンドリーな素数である。

/// 素数 998244353 を法とするモジュラー整数を表現する。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModInt998244353 {
    /// `[0, MOD)` の範囲に正規化された内部表現である。
    val: u32,
}

impl ModInt998244353 {
    /// `u64` 値から新しい `ModInt998244353` インスタンスを生成する。
    ///
    /// # Args
    /// - `n`: モジュラー整数に変換する `u64` 値
    ///
    /// # Returns
    /// `Self`: 値が `MOD` で還元された新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::new(1_000_000_000);
    /// assert_eq!(1755647, m.val());
    /// ```
    pub fn new(n: u64) -> Self {
        // n を MOD で還元し、[0, MOD) の範囲に収める。
        ModInt998244353 {
            val: (n % MOD as u64) as u32,
        }
    }

    /// 生の `u32` 値から新しい `ModInt998244353` インスタンスを生成する。
    ///
    /// # Args
    /// - `n`: `MOD` 未満でなければならない `u32` 値
    ///
    /// # Returns
    /// `Self`: 新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// - `n` は `MOD` 未満でなければならない。
    ///
    /// # Panics
    /// `n` が `MOD` 以上の場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::new_raw(100);
    /// assert_eq!(100, m.val());
    /// ```
    pub fn new_raw(n: u32) -> Self {
        // 呼び出し側が n < MOD を保証している前提で、還元処理を省略して直接構築する。
        assert!(n < MOD, "Raw value {} must be less than MOD {}", n, MOD);
        ModInt998244353 { val: n }
    }

    /// モジュラー整数の基となる `u32` 値を返す。
    ///
    /// # Returns
    /// `u32`: モジュラー整数の生の `u32` 値
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::new(123);
    /// assert_eq!(123, m.val());
    /// ```
    pub fn val(&self) -> u32 {
        self.val
    }

    /// モジュラー整数のモジュラー乗法逆元を計算する。
    ///
    /// このメソッドはフェルマーの小定理を用いており、法 `MOD` が素数であることを前提とする。
    /// `MOD` は素数 (998244353) であるため、逆元 `a^(MOD-2)` を計算する。
    ///
    /// # Returns
    /// `Option<Self>`: `self.val` が非ゼロの場合は `Some(逆元)` を、ゼロの場合は
    /// 逆元が存在しないため `None` を返す。
    ///
    /// # Constraints
    /// - `MOD` が素数であるとき、逆元が存在するには `self.val` がゼロであってはならない。
    ///
    /// # Complexity
    /// - 時間計算量: O(log MOD)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::new(2);
    /// let inv_m = m.inv().unwrap();
    /// assert_eq!(1, (m * inv_m).val());
    ///
    /// let zero = modint::ModInt998244353::new(0);
    /// assert!(zero.inv().is_none());
    /// ```
    pub fn inv(&self) -> Option<Self> {
        if self.val == 0 {
            None
        } else {
            // フェルマーの小定理より、MOD が素数のとき a^(MOD-2) mod MOD が乗法逆元となる。
            Some(self.pow((MOD - 2) as usize))
        }
    }

    /// `self` の `n` 乗を計算する。
    ///
    /// # Args
    /// - `n`: 非負の冪指数
    ///
    /// # Returns
    /// `Self`: `self` を `n` 乗した結果
    ///
    /// # Complexity
    /// - 時間計算量: O(log n)
    ///   - n は冪指数である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let base = modint::ModInt998244353::new(3);
    /// let result = base.pow(4);
    /// assert_eq!(81, result.val());
    /// ```
    pub fn pow(&self, mut n: usize) -> Self {
        // 累積する結果。初期値は乗法単位元の 1 である。
        let mut res = ModInt998244353::new_raw(1);
        // 2 乗を繰り返していく途中経過の底。
        let mut base = *self;

        // 二分累乗法 (繰り返し二乗法) により、O(log n) 回の乗算で self^n を計算する。
        while n > 0 {
            // n の最下位ビットが 1 の場合、現在の底を結果に掛け合わせる。
            if n % 2 == 1 {
                res *= base;
            }
            // 底を 2 乗し、次の桁に備える。
            base *= base;
            n /= 2;
        }
        res
    }
}

/// `u32` から `ModInt998244353` への変換を可能にする。
impl From<u32> for ModInt998244353 {
    /// `u32` の値から `ModInt998244353` インスタンスを生成する。
    ///
    /// # Args
    /// - `num`: 変換する `u32` の値
    ///
    /// # Returns
    /// `num` を `998244353` で割った余りと等価な、新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let val = 1_000_000_007_u32;
    /// let m = modint::ModInt998244353::from(val);
    /// assert_eq!(1755654, m.val()); // 1_000_000_007 % 998244353
    /// ```
    fn from(num: u32) -> Self {
        // u64 に拡張して new に委譲し、MOD による還元を行う。
        ModInt998244353::new(num as u64)
    }
}

/// `i32` から `ModInt998244353` への変換を可能にする。
impl From<i32> for ModInt998244353 {
    /// `i32` の値から `ModInt998244353` インスタンスを生成する。負数も正しく扱う。
    ///
    /// # Args
    /// - `num`: 変換する `i32` の値
    ///
    /// # Returns
    /// 新しい `ModInt998244353` インスタンス。負の入力は、法演算における正の等価値に変換される。
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m_pos = modint::ModInt998244353::from(10_i32);
    /// assert_eq!(10, m_pos.val());
    ///
    /// let m_neg = modint::ModInt998244353::from(-10_i32);
    /// assert_eq!(998244353 - 10, m_neg.val());
    /// ```
    fn from(num: i32) -> Self {
        // num が負の場合、num % MOD as i32 は負の値になり得るため、MOD を加えて非負の値に補正する。
        let val = if num >= 0 {
            num as u64
        } else {
            (num % MOD as i32 + MOD as i32) as u64
        };
        ModInt998244353::new(val)
    }
}

/// 2 つの `ModInt998244353` インスタンスに対する加算演算 (`+`) を実装する。
impl ops::Add for ModInt998244353 {
    type Output = Self;

    /// 2 つの `ModInt998244353` インスタンスを加算する。
    ///
    /// # Args
    /// - `self`: 左辺のオペランド
    /// - `rhs`: 右辺のオペランド
    ///
    /// # Returns
    /// 和を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(998244350);
    /// let b = modint::ModInt998244353::new(10);
    /// assert_eq!(modint::ModInt998244353::new(7), a + b);
    /// ```
    fn add(mut self, rhs: Self) -> Self::Output {
        // += 演算子に処理を委譲し、結果を新しい値として返す。
        self += rhs;
        self
    }
}

/// `ModInt998244353` と `u32` に対する加算演算 (`+`) を実装する。
impl ops::Add<u32> for ModInt998244353 {
    type Output = Self;

    /// `ModInt998244353` インスタンスに `u32` の値を加算する。
    ///
    /// # Args
    /// - `self`: `ModInt998244353` インスタンス
    /// - `rhs`: 加算する `u32` の値
    ///
    /// # Returns
    /// 和を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(998244350);
    /// let b = 10_u32;
    /// assert_eq!(modint::ModInt998244353::new(7), a + b);
    /// ```
    fn add(mut self, rhs: u32) -> Self::Output {
        // += 演算子に処理を委譲し、結果を新しい値として返す。
        self += rhs;
        self
    }
}

/// `ModInt998244353` に対する加算代入演算 (`+=`) を実装する。
impl ops::AddAssign for ModInt998244353 {
    /// 別の `ModInt998244353` インスタンスを `self` に加算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 右辺のオペランド
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(998244350);
    /// let b = modint::ModInt998244353::new(10);
    /// a += b;
    /// assert_eq!(modint::ModInt998244353::new(7), a);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        // self.val, rhs.val はいずれも MOD 未満なので、和は u32 でオーバーフローしない。
        self.val += rhs.val;
        // 和が MOD 以上になった場合のみ MOD を引き、[0, MOD) の範囲に戻す。
        if self.val >= MOD {
            self.val -= MOD;
        }
    }
}

/// `ModInt998244353` と `u32` に対する加算代入演算 (`+=`) を実装する。
impl ops::AddAssign<u32> for ModInt998244353 {
    /// `u32` の値を `self` に加算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 加算する `u32` の値
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(998244350);
    /// a += 10_u32;
    /// assert_eq!(modint::ModInt998244353::new(7), a);
    /// ```
    fn add_assign(&mut self, rhs: u32) {
        // rhs は MOD 未満とは限らないため、先に MOD で還元しておく。
        let rhs_mod = rhs % MOD;
        self.val += rhs_mod;
        // 和が MOD 以上になった場合のみ MOD を引き、[0, MOD) の範囲に戻す。
        if self.val >= MOD {
            self.val -= MOD;
        }
    }
}

/// 2 つの `ModInt998244353` インスタンスに対する減算演算 (`-`) を実装する。
impl ops::Sub for ModInt998244353 {
    type Output = Self;

    /// ある `ModInt998244353` インスタンスから別のインスタンスを減算する。
    ///
    /// # Args
    /// - `self`: 左辺のオペランド (被減数)
    /// - `rhs`: 右辺のオペランド (減数)
    ///
    /// # Returns
    /// 差を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(10);
    /// let b = modint::ModInt998244353::new(20);
    /// assert_eq!(modint::ModInt998244353::new(998244343), a - b);
    /// ```
    fn sub(mut self, rhs: Self) -> Self::Output {
        // -= 演算子に処理を委譲し、結果を新しい値として返す。
        self -= rhs;
        self
    }
}

/// `ModInt998244353` と `u32` に対する減算演算 (`-`) を実装する。
impl ops::Sub<u32> for ModInt998244353 {
    type Output = Self;

    /// `ModInt998244353` インスタンスから `u32` の値を減算する。
    ///
    /// # Args
    /// - `self`: `ModInt998244353` インスタンス
    /// - `rhs`: 減算する `u32` の値
    ///
    /// # Returns
    /// 差を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(10);
    /// let b = 20_u32;
    /// assert_eq!(modint::ModInt998244353::new(998244343), a - b);
    /// ```
    fn sub(mut self, rhs: u32) -> Self::Output {
        // -= 演算子に処理を委譲し、結果を新しい値として返す。
        self -= rhs;
        self
    }
}

/// `ModInt998244353` に対する減算代入演算 (`-=`) を実装する。
impl ops::SubAssign for ModInt998244353 {
    /// 別の `ModInt998244353` インスタンスを `self` から減算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 右辺のオペランド
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(10);
    /// let b = modint::ModInt998244353::new(20);
    /// a -= b;
    /// assert_eq!(modint::ModInt998244353::new(998244343), a);
    /// ```
    fn sub_assign(&mut self, rhs: Self) {
        // self.val が rhs.val 以上であれば、そのまま引くだけで [0, MOD) の範囲に収まる。
        if self.val >= rhs.val {
            self.val -= rhs.val;
        } else {
            // self.val が rhs.val 未満の場合はアンダーフローするため、先に MOD を足してから引く。
            self.val += MOD - rhs.val;
        }
    }
}

/// `ModInt998244353` と `u32` に対する減算代入演算 (`-=`) を実装する。
impl ops::SubAssign<u32> for ModInt998244353 {
    /// `u32` の値を `self` から減算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 減算する `u32` の値
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(10);
    /// a -= 20_u32;
    /// assert_eq!(modint::ModInt998244353::new(998244343), a);
    /// ```
    fn sub_assign(&mut self, rhs: u32) {
        // rhs は MOD 未満とは限らないため、先に MOD で還元しておく。
        let rhs_mod = rhs % MOD;
        // self.val が rhs_mod 以上であれば、そのまま引くだけで [0, MOD) の範囲に収まる。
        if self.val >= rhs_mod {
            self.val -= rhs_mod;
        } else {
            // self.val が rhs_mod 未満の場合はアンダーフローするため、先に MOD を足してから引く。
            self.val += MOD - rhs_mod;
        }
    }
}

/// 2 つの `ModInt998244353` インスタンスに対する乗算演算 (`*`) を実装する。
impl ops::Mul for ModInt998244353 {
    type Output = Self;

    /// 2 つの `ModInt998244353` インスタンスを乗算する。
    ///
    /// # Args
    /// - `self`: 左辺のオペランド
    /// - `rhs`: 右辺のオペランド
    ///
    /// # Returns
    /// 積を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(100_000);
    /// let b = modint::ModInt998244353::new(100_000);
    /// assert_eq!(modint::ModInt998244353::new(17556470), a * b);
    /// ```
    fn mul(self, rhs: Self) -> Self::Output {
        // self.val * rhs.val は最大で (MOD - 1)^2 程度になり u32 を超えるため、
        // u64 に拡張してから乗算し、new で MOD により還元する。
        ModInt998244353::new((self.val as u64) * (rhs.val as u64))
    }
}

/// `ModInt998244353` と `u32` に対する乗算演算 (`*`) を実装する。
impl ops::Mul<u32> for ModInt998244353 {
    type Output = Self;

    /// `ModInt998244353` インスタンスに `u32` の値を乗算する。
    ///
    /// # Args
    /// - `self`: `ModInt998244353` インスタンス
    /// - `rhs`: 乗算する `u32` の値
    ///
    /// # Returns
    /// 積を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(100_000);
    /// let b = 100_000_u32;
    /// assert_eq!(modint::ModInt998244353::new(17556470), a * b);
    /// ```
    fn mul(self, rhs: u32) -> Self::Output {
        // self.val * rhs は最大で (MOD - 1) * (u32::MAX) 程度になり u32 を超えるため、
        // u64 に拡張してから乗算し、new で MOD により還元する。
        ModInt998244353::new((self.val as u64) * (rhs as u64))
    }
}

/// `ModInt998244353` に対する乗算代入演算 (`*=`) を実装する。
impl ops::MulAssign for ModInt998244353 {
    /// `self` に別の `ModInt998244353` インスタンスを乗算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 右辺のオペランド
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(100_000);
    /// let b = modint::ModInt998244353::new(100_000);
    /// a *= b;
    /// assert_eq!(modint::ModInt998244353::new(17556470), a);
    /// ```
    fn mul_assign(&mut self, rhs: Self) {
        // 乗算演算子に処理を委譲する。
        *self = *self * rhs;
    }
}

/// `ModInt998244353` と `u32` に対する乗算代入演算 (`*=`) を実装する。
impl ops::MulAssign<u32> for ModInt998244353 {
    /// `self` に `u32` の値を乗算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 乗算する `u32` の値
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(100_000);
    /// a *= 100_000_u32;
    /// assert_eq!(modint::ModInt998244353::new(17556470), a);
    /// ```
    fn mul_assign(&mut self, rhs: u32) {
        // 乗算演算子に処理を委譲する。
        *self = *self * rhs;
    }
}

/// 2 つの `ModInt998244353` インスタンスに対する除算演算 (`/`) を実装する。
impl ops::Div for ModInt998244353 {
    type Output = Self;

    /// モジュラー逆元を用いて、`self` を別の `ModInt998244353` インスタンスで除算する。
    ///
    /// # Args
    /// - `self`: 被除数
    /// - `rhs`: 除数
    ///
    /// # Returns
    /// 商を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// - 除数がゼロであってはならない。
    ///
    /// # Panics
    /// 除数 `rhs` がゼロの場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log M)
    ///   - M は法である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(20);
    /// let b = modint::ModInt998244353::new(4);
    /// assert_eq!(modint::ModInt998244353::new(5), a / b);
    /// ```
    fn div(self, rhs: Self) -> Self::Output {
        // 除算を乗算として扱うため、除数の逆元をあらかじめ計算する。
        // 逆元が存在しない (rhs が 0 の) 場合はパニックする。
        let inv_rhs = rhs
            .inv()
            .unwrap_or_else(|| panic!("Division by zero is not allowed for ModInt998244353"));
        // 逆元との積が商になる。積は (MOD-1)^2 未満で u64 に収まるため、
        // `wrapping_mul` は通常の乗算と一致し、`new` で法による還元を行う。
        Self::new((self.val as u64).wrapping_mul(inv_rhs.val as u64))
    }
}

/// `ModInt998244353` と `u32` に対する除算演算 (`/`) を実装する。
impl ops::Div<u32> for ModInt998244353 {
    type Output = Self;

    /// モジュラー逆元を用いて、`ModInt998244353` インスタンスを `u32` の値で除算する。
    ///
    /// # Args
    /// - `self`: 被除数
    /// - `rhs`: `u32` の除数
    ///
    /// # Returns
    /// 商を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// - 除数がゼロであってはならない。
    ///
    /// # Panics
    /// 除数 `rhs` がゼロの場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log M)
    ///   - M は法である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(20);
    /// let b = 4_u32;
    /// assert_eq!(modint::ModInt998244353::new(5), a / b);
    /// ```
    fn div(self, rhs: u32) -> Self::Output {
        // rhs を ModInt998244353 に変換したうえで、除算演算子に処理を委譲する。
        // 委譲先で逆元による乗算として商を求めるため、結果は直接の逆元計算と一致する。
        let rhs_mod = ModInt998244353::new(rhs as u64);
        self / rhs_mod
    }
}

/// `ModInt998244353` に対する除算代入演算 (`/=`) を実装する。
impl ops::DivAssign for ModInt998244353 {
    /// `self` を別の `ModInt998244353` インスタンスで除算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: 除数
    ///
    /// # Constraints
    /// - 除数がゼロであってはならない。
    ///
    /// # Panics
    /// 除数 `rhs` がゼロの場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log M)
    ///   - M は法である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(20);
    /// let b = modint::ModInt998244353::new(4);
    /// a /= b;
    /// assert_eq!(modint::ModInt998244353::new(5), a);
    /// ```
    fn div_assign(&mut self, rhs: Self) {
        // 除算演算子に処理を委譲する。
        *self = *self / rhs;
    }
}

/// `ModInt998244353` と `u32` に対する除算代入演算 (`/=`) を実装する。
impl ops::DivAssign<u32> for ModInt998244353 {
    /// `self` を `u32` の値で除算する。
    ///
    /// # Args
    /// - `self`: 変更される `ModInt998244353` インスタンス
    /// - `rhs`: `u32` の除数
    ///
    /// # Constraints
    /// - 除数がゼロであってはならない。
    ///
    /// # Panics
    /// 除数 `rhs` がゼロの場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: O(log M)
    ///   - M は法である。
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let mut a = modint::ModInt998244353::new(20);
    /// a /= 4_u32;
    /// assert_eq!(modint::ModInt998244353::new(5), a);
    /// ```
    fn div_assign(&mut self, rhs: u32) {
        // 除算演算子に処理を委譲する。
        *self = *self / rhs;
    }
}

/// `ModInt998244353` に対する単項否定演算 (`-`) を実装する。
impl ops::Neg for ModInt998244353 {
    type Output = Self;

    /// `ModInt998244353` インスタンスの単項否定 (符号反転) を計算する。
    ///
    /// # Args
    /// - `self`: 符号反転する値
    ///
    /// # Returns
    /// 符号反転された値を表す新しい `ModInt998244353` インスタンス
    ///
    /// # Constraints
    /// 入力値に関する制約はない。
    ///
    /// # Complexity
    /// - 時間計算量: O(1)
    /// - 空間計算量: O(1)
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let a = modint::ModInt998244353::new(10);
    /// assert_eq!(modint::ModInt998244353::new(998244343), -a);
    ///
    /// let zero = modint::ModInt998244353::new(0);
    /// assert_eq!(zero, -zero);
    /// ```
    fn neg(self) -> Self::Output {
        // 0 の加法逆元は 0 自身であり、それ以外は MOD から val を引いた値が加法逆元となる。
        if self.val == 0 {
            Self::new_raw(0)
        } else {
            Self::new_raw(MOD - self.val)
        }
    }
}

/// `ModInt998244353` の表示形式 (`Display`) を実装する。
impl fmt::Display for ModInt998244353 {
    /// 内部の値をそのまま文字列として書式化する。
    ///
    /// # Args
    /// - `f`: 書き込み先のフォーマッタ
    ///
    /// # Returns
    /// 書式化の結果を表す `fmt::Result`
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::new(42);
    /// assert_eq!("42", format!("{}", m));
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

/// `ModInt998244353` の既定値を提供する。
impl Default for ModInt998244353 {
    /// 既定値として、値が `0` である `ModInt998244353` を返す。
    ///
    /// # Returns
    /// 値が `0` である `ModInt998244353` インスタンス
    ///
    /// # Examples
    /// ```rust
    /// use anmitsu::modulo998244353::modint;
    ///
    /// let m = modint::ModInt998244353::default();
    /// assert_eq!(0, m.val());
    /// ```
    fn default() -> Self {
        ModInt998244353 { val: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // new のテスト: 戻り値を検証する。
    mod new {
        use super::*;

        /// Scenario: MOD 未満の値を渡すと、そのままの値を持つインスタンスが生成される。
        /// - Given: MOD 未満の値がある。
        /// - When: `new` でインスタンスを生成する。
        /// - Then: 入力と同じ値を持つインスタンスが得られる。
        #[test]
        fn creates_instance_with_value_unchanged_when_less_than_mod() {
            // Given
            let cases = [0_u64, 1, 123, (MOD - 1) as u64];

            for n in cases {
                // When
                let sut = ModInt998244353::new(n);
                // Then
                assert_eq!(n as u32, sut.val());
            }
        }

        /// Scenario: MOD 以上の値を渡すと、MOD で還元された値になる。
        /// - Given: MOD 以上の値がある。
        /// - When: `new` でインスタンスを生成する。
        /// - Then: `n mod MOD` の値を持つインスタンスが得られる。
        #[test]
        fn wraps_around_when_value_is_ge_mod() {
            // Given
            let cases = [
                (MOD as u64, 0_u32),
                (MOD as u64 + 1, 1),
                (2 * MOD as u64, 0),
                (u64::MAX, (u64::MAX % MOD as u64) as u32),
            ];

            for (n, expected) in cases {
                // When
                let sut = ModInt998244353::new(n);
                // Then
                assert_eq!(expected, sut.val());
            }
        }
    }

    // new_raw のテスト: 戻り値および異常系を検証する。
    mod new_raw {
        use super::*;

        /// Scenario: MOD 未満の値を渡すと、そのままの値を持つインスタンスが生成される。
        /// - Given: MOD 未満の値がある。
        /// - When: `new_raw` でインスタンスを生成する。
        /// - Then: 入力と同じ値を持つインスタンスが得られる。
        #[test]
        fn creates_instance_for_valid_value() {
            // Given
            let n = MOD - 1;
            // When
            let sut = ModInt998244353::new_raw(n);
            // Then
            assert_eq!(n, sut.val());
        }

        /// Scenario: MOD と等しい値を渡すとパニックする。
        /// - Given: MOD と等しい値がある。
        /// - When: `new_raw` でインスタンスを生成する。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "Raw value 998244353 must be less than MOD 998244353")]
        fn panics_when_value_equals_mod() {
            // Given, When, Then (パニックする)
            ModInt998244353::new_raw(MOD);
        }

        /// Scenario: MOD を超える値を渡すとパニックする。
        /// - Given: MOD より大きい値がある。
        /// - When: `new_raw` でインスタンスを生成する。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "Raw value 998244354 must be less than MOD 998244353")]
        fn panics_when_value_exceeds_mod() {
            // Given, When, Then (パニックする)
            ModInt998244353::new_raw(MOD + 1);
        }
    }

    // From<u32>, From<i32> のテスト: 戻り値を検証する。
    mod from {
        use super::*;

        /// Scenario: `u32` の値は、MOD で還元されて変換される。
        /// - Given: 0、MOD 未満の値、MOD 以上の値、u32::MAX を含む複数の入力がある。
        /// - When: `ModInt998244353::from` で変換する。
        /// - Then: 各ケースで `input mod MOD` の値が得られる。
        #[test]
        fn converts_u32_reducing_by_modulus() {
            // Given
            let cases = [
                (0_u32, 0_u32),
                (123, 123),
                (MOD, 0),
                (MOD + 10, 10),
                (u32::MAX, u32::MAX % MOD),
            ];

            for (input, expected) in cases {
                // When
                let sut = ModInt998244353::from(input);
                // Then
                assert_eq!(expected, sut.val());
            }
        }

        /// Scenario: `i32` の非負の値は、MOD で還元されて変換される。
        /// - Given: 0、通常の正の値、型の最大値 (i32::MAX) がある。
        /// - When: `ModInt998244353::from` で変換する。
        /// - Then: 各ケースで `input mod MOD` の値が得られる。
        #[test]
        fn converts_nonnegative_i32_reducing_by_modulus() {
            // Given
            let cases = [
                (0_i32, 0_u32),
                (123, 123),
                (i32::MAX, (i32::MAX as u64 % MOD as u64) as u32),
            ];

            for (input, expected) in cases {
                // When
                let sut = ModInt998244353::from(input);
                // Then
                assert_eq!(expected, sut.val());
            }
        }

        /// Scenario: `i32` の負の値は、法演算における正の等価値に変換される。
        /// - Given: -1、型の最小値 (i32::MIN) を含む負の値がある。
        /// - When: `ModInt998244353::from` で変換する。
        /// - Then: MOD を加えて正規化された値が得られる。
        #[test]
        fn converts_negative_i32_to_positive_equivalent() {
            // Given
            let cases = [
                (-1_i32, MOD - 1),
                (-123, MOD - 123),
                // (-2147483648 % 998244353 + 998244353) % 998244353 = 847249411
                (i32::MIN, 847249411),
            ];

            for (input, expected) in cases {
                // When
                let sut = ModInt998244353::from(input);
                // Then
                assert_eq!(expected, sut.val());
            }
        }
    }

    // add, add_assign のテスト: 戻り値と状態変化を検証する。
    mod add {
        use super::*;

        /// Scenario: `+` および `+=` は、MOD の下での和を返す。
        /// - Given: 和が MOD を超えない、または超える複数の値の組がある。
        /// - When: 加算 (`+`) および加算代入 (`+=`) を行う。
        /// - Then: いずれも `(a + b) mod MOD` になる。
        #[test]
        fn computes_sum_modulo_mod() {
            // Given
            let cases = [
                (1_u32, 1_u32, 2_u32),
                (MOD - 1, 1, 0),
                (MOD - 1, 2, 1),
                (0, 0, 0),
                (MOD - 1, MOD - 1, MOD - 2),
            ];

            for (a_val, b_val, expected) in cases {
                let a = ModInt998244353::new(a_val as u64);
                let b = ModInt998244353::new(b_val as u64);

                // When (+)
                let sut = a + b;
                // Then (+)
                assert_eq!(expected, sut.val());

                // When (+=)
                let mut sut = a;
                sut += b;
                // Then (+=)
                assert_eq!(expected, sut.val());
            }
        }

        /// Scenario: `u32` を右辺に取る `+` および `+=` も、MOD の下での和を返す。
        /// - Given: `ModInt998244353` と `u32` の組がある。
        /// - When: 加算 (`+`) および加算代入 (`+=`) を行う。
        /// - Then: いずれも `(a + b) mod MOD` になる。
        #[test]
        fn computes_sum_with_u32_rhs() {
            // Given
            let a = ModInt998244353::new(998244350);
            let b = 10_u32;

            // When (+)
            let sut = a + b;
            // Then (+)
            assert_eq!(7, sut.val());

            // When (+=)
            let mut sut = a;
            sut += b;
            // Then (+=)
            assert_eq!(7, sut.val());
        }
    }

    // sub, sub_assign のテスト: 戻り値と状態変化を検証する。
    mod sub {
        use super::*;

        /// Scenario: `-` および `-=` は、MOD の下での差を返す。
        /// - Given: 被減数が減数以上、または未満の複数の値の組がある。
        /// - When: 減算 (`-`) および減算代入 (`-=`) を行う。
        /// - Then: いずれも `(a - b) mod MOD` になる。
        #[test]
        fn computes_difference_modulo_mod() {
            // Given
            let cases = [
                (2_u64, 1_u64, 1_u32),
                (1, 2, MOD - 1),
                (0, 0, 0),
                (123, 123, 0),
            ];

            for (a_val, b_val, expected) in cases {
                let a = ModInt998244353::new(a_val);
                let b = ModInt998244353::new(b_val);

                // When (-)
                let sut = a - b;
                // Then (-)
                assert_eq!(expected, sut.val());

                // When (-=)
                let mut sut = a;
                sut -= b;
                // Then (-=)
                assert_eq!(expected, sut.val());
            }
        }

        /// Scenario: `u32` を右辺に取る `-` および `-=` も、MOD の下での差を返す。
        /// - Given: `ModInt998244353` と `u32` の組がある。
        /// - When: 減算 (`-`) および減算代入 (`-=`) を行う。
        /// - Then: いずれも `(a - b) mod MOD` になる。
        #[test]
        fn computes_difference_with_u32_rhs() {
            // Given
            let a = ModInt998244353::new(10);
            let b = 20_u32;

            // When (-)
            let sut = a - b;
            // Then (-)
            assert_eq!(MOD - 10, sut.val());

            // When (-=)
            let mut sut = a;
            sut -= b;
            // Then (-=)
            assert_eq!(MOD - 10, sut.val());
        }
    }

    // mul, mul_assign のテスト: 戻り値と状態変化を検証する。
    mod mul {
        use super::*;

        /// Scenario: `*` および `*=` は、MOD の下での積を返す。
        /// - Given: 0 を含む値、MOD - 1 同士など複数の値の組がある。
        /// - When: 乗算 (`*`) および乗算代入 (`*=`) を行う。
        /// - Then: いずれも `(a * b) mod MOD` になる。
        #[test]
        fn computes_product_modulo_mod() {
            // Given
            let cases = [
                (2_u64, 3_u64, 6_u32),
                (0, 123, 0),
                (MOD as u64 - 1, MOD as u64 - 1, 1), // (-1) * (-1) = 1
                (100_000, 100_000, 17556470),        // 10_000_000_000 % MOD
            ];

            for (a_val, b_val, expected) in cases {
                let a = ModInt998244353::new(a_val);
                let b = ModInt998244353::new(b_val);

                // When (*)
                let sut = a * b;
                // Then (*)
                assert_eq!(expected, sut.val());

                // When (*=)
                let mut sut = a;
                sut *= b;
                // Then (*=)
                assert_eq!(expected, sut.val());
            }
        }

        /// Scenario: `u32` を右辺に取る `*` および `*=` も、MOD の下での積を返す。
        /// - Given: `ModInt998244353` と `u32` の組がある。
        /// - When: 乗算 (`*`) および乗算代入 (`*=`) を行う。
        /// - Then: いずれも `(a * b) mod MOD` になる。
        #[test]
        fn computes_product_with_u32_rhs() {
            // Given
            let a = ModInt998244353::new(100_000);
            let b = 100_000_u32;

            // When (*)
            let sut = a * b;
            // Then (*)
            assert_eq!(17556470, sut.val());

            // When (*=)
            let mut sut = a;
            sut *= b;
            // Then (*=)
            assert_eq!(17556470, sut.val());
        }
    }

    // div, div_assign のテスト: 戻り値、状態変化、および異常系を検証する。
    mod div {
        use super::*;

        /// Scenario: `/` および `/=` は、除数の逆元を用いた商を返す。
        /// - Given: 割り切れる組、割り切れない組を含む複数の値の組がある。
        /// - When: 除算 (`/`) および除算代入 (`/=`) を行う。
        /// - Then: いずれも商と除数の積が被除数に一致する。
        #[test]
        fn computes_quotient_via_modular_inverse() {
            // Given
            let cases = [(6_u64, 2_u64), (7, 2), (0, 123), (123, 123)];

            for (a_val, b_val) in cases {
                let a = ModInt998244353::new(a_val);
                let b = ModInt998244353::new(b_val);

                // When (/)
                let sut = a / b;
                // Then (/)
                assert_eq!(a, sut * b);

                // When (/=)
                let mut sut = a;
                sut /= b;
                // Then (/=)
                assert_eq!(a, sut * b);
            }
        }

        /// Scenario: ゼロによる除算 (`/`) はパニックする。
        /// - Given: 除数がゼロの組がある。
        /// - When: 除算する。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "Division by zero is not allowed for ModInt998244353")]
        fn div_panics_when_divisor_is_zero() {
            // Given
            let a = ModInt998244353::new(10);
            let b = ModInt998244353::new(0);
            // When, Then (パニックする)
            let _ = a / b;
        }

        /// Scenario: ゼロによる除算代入 (`/=`) はパニックする。
        /// - Given: 除数がゼロの組がある。
        /// - When: 除算代入する。
        /// - Then: パニックする。
        #[test]
        #[should_panic(expected = "Division by zero is not allowed for ModInt998244353")]
        fn div_assign_panics_when_divisor_is_zero() {
            // Given
            let mut a = ModInt998244353::new(10);
            let b = ModInt998244353::new(0);
            // When, Then (パニックする)
            a /= b;
        }
    }

    // neg のテスト: 戻り値を検証する。
    mod neg {
        use super::*;

        /// Scenario: 単項否定は加法逆元を返し、2 回適用すると元の値に戻る。
        /// - Given: 0、通常の値、MOD - 1 を含む複数の値がある。
        /// - When: 単項否定を行う。
        /// - Then: `MOD - val` (val が 0 のときは 0) になり、もう一度否定すると元の値に戻る。
        #[test]
        fn negates_value_and_is_involutive() {
            // Given
            let cases = [
                (0_u64, 0_u32),
                (1, MOD - 1),
                (123, MOD - 123),
                (MOD as u64 - 1, 1),
            ];

            for (val, expected) in cases {
                let sut = ModInt998244353::new(val);

                // When
                let negated = -sut;
                // Then
                assert_eq!(expected, negated.val());
                assert_eq!(sut, -negated);
            }
        }
    }

    // pow のテスト: 戻り値を検証する。
    mod pow {
        use super::*;

        /// Scenario: `pow` は、累乗の結果を MOD の下で返す。
        /// - Given: 通常の底と指数、指数 0、指数 1 を含む複数の組がある。
        /// - When: べき乗を計算する。
        /// - Then: 各ケースで期待した値が返る。
        #[test]
        fn computes_power_modulo_mod() {
            // Given
            let cases = [
                (2_u64, 10_usize, 1024_u32),
                (3, 4, 81),
                (123, 0, 1),   // 指数が 0 の場合は底に関わらず 1 になる。
                (123, 1, 123), // 指数が 1 の場合は底そのものになる。
                (0, 5, 0),
                (MOD as u64 - 1, 2, 1),
            ];

            for (base_val, exp, expected) in cases {
                // When
                let sut = ModInt998244353::new(base_val);
                let result = sut.pow(exp);
                // Then
                assert_eq!(expected, result.val());
            }
        }
    }

    // inv のテスト: 戻り値を検証する。
    mod inv {
        use super::*;

        /// Scenario: 非ゼロの値の逆元は、元の値と乗算すると乗法単位元 1 になる。
        /// - Given: 通常の値と、境界値 (MOD - 1) がある。
        /// - When: 逆元を計算する。
        /// - Then: `Some` が返り、元の値との積が 1 になる。
        #[test]
        fn multiplies_to_one_with_original_value() {
            // Given
            let cases = [2_u64, 123, MOD as u64 - 1];

            for val in cases {
                let sut = ModInt998244353::new(val);
                let one = ModInt998244353::new(1);

                // When
                let result = sut.inv();

                // Then
                assert!(result.is_some());
                assert_eq!(one, sut * result.unwrap());
            }
        }

        /// Scenario: ゼロには逆元が存在しない。
        /// - Given: 値が 0 のインスタンスがある。
        /// - When: 逆元を計算する。
        /// - Then: `None` が返る。
        #[test]
        fn returns_none_for_zero() {
            // Given
            let sut = ModInt998244353::new(0);
            // When
            let result = sut.inv();
            // Then
            assert!(result.is_none());
        }
    }

    // Display のテスト: 依存先 (フォーマッタ) への出力内容を検証する。
    mod display {
        use super::*;

        /// Scenario: `Display` は、内部値をそのまま文字列として出力する。
        /// - Given: MOD 未満の値、MOD 以上の値がある。
        /// - When: `format!` で文字列化する。
        /// - Then: 還元後の値がそのまま文字列になる。
        #[test]
        fn formats_underlying_value() {
            // Given
            let cases = [(10_u64, "10"), (MOD as u64 + 5, "5")];

            for (input, expected) in cases {
                let sut = ModInt998244353::new(input);

                // When
                let result = format!("{}", sut);

                // Then
                assert_eq!(expected, result);
            }
        }
    }

    // Default のテスト: 戻り値を検証する。
    mod default {
        use super::*;

        /// Scenario: 既定値は、値が 0 のインスタンスになる。
        /// - Given: (前提なし)
        /// - When: `Default::default` を呼ぶ。
        /// - Then: 値が 0 のインスタンスが得られる。
        #[test]
        fn returns_zero_value() {
            // Given, When
            let sut = ModInt998244353::default();
            // Then
            assert_eq!(0, sut.val());
        }
    }

    // PartialEq, Eq, Copy, Clone の派生実装のテスト: 戻り値を検証する。
    mod derived_traits {
        use super::*;

        /// Scenario: 内部値が等しいインスタンス同士は等価と判定される。
        /// - Given: 同じ値を持つインスタンスと、MOD の加算により同じ剰余類に属するインスタンスがある。
        /// - When: 等価性を比較する。
        /// - Then: いずれも等しいと判定される。
        #[test]
        fn considers_instances_with_same_residue_equal() {
            // Given
            let a1 = ModInt998244353::new(100);
            let a2 = ModInt998244353::new(100);
            let c = ModInt998244353::new(MOD as u64 + 100);

            // When, Then
            assert_eq!(a1, a2);
            assert_eq!(a1, c);
        }

        /// Scenario: 内部値が異なるインスタンス同士は非等価と判定される。
        /// - Given: 異なる値を持つ 2 つのインスタンスがある。
        /// - When: 等価性を比較する。
        /// - Then: 等しくないと判定される。
        #[test]
        fn considers_instances_with_different_residue_unequal() {
            // Given
            let a = ModInt998244353::new(100);
            let b = ModInt998244353::new(200);

            // When, Then
            assert_ne!(a, b);
        }

        /// Scenario: `Copy` によって複製された値は、元の値と独立に変更できる。
        /// - Given: 1 つのインスタンスがある。
        /// - When: 代入によって複製し、複製先だけを変更する。
        /// - Then: 元のインスタンスは変更されない。
        #[test]
        fn copy_leaves_original_unaffected() {
            // Given
            let sut = ModInt998244353::new(12345);

            // When
            let mut copied = sut;
            copied += 1_u32;

            // Then
            assert_eq!(12345, sut.val());
            assert_eq!(12346, copied.val());
        }

        /// Scenario: `Clone` によって複製された値は、元の値と独立に変更できる。
        /// - Given: 1 つのインスタンスがある。
        /// - When: `clone` によって複製し、複製先だけを変更する。
        /// - Then: 元のインスタンスは変更されない。
        #[test]
        fn clone_leaves_original_unaffected() {
            // Given
            let sut = ModInt998244353::new(12345);

            // When
            let mut cloned = sut;
            cloned -= 1_u32;

            // Then
            assert_eq!(12345, sut.val());
            assert_eq!(12344, cloned.val());
        }
    }
}

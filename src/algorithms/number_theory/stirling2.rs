use crate::data_structures::modint::ModInt;

#[allow(unused_macros)]
pub mod macros {
    macro_rules! min { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); std::cmp::min($x, y) } }}
    macro_rules! max { ($x: expr) => { $x }; ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); std::cmp::max($x, y) } }}
    macro_rules! chmin { ($x: expr, $($xs: expr),+) => {{ let y = macros::min!($($xs),+); if $x > y { $x = y; true } else { false } }}}
    macro_rules! chmax { ($x: expr, $($xs: expr),+) => {{ let y = macros::max!($($xs),+); if $x < y { $x = y; true } else { false } }}}
    macro_rules! multi_vec { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_vec![$element; ($($lens),*)]; $len] ); ($element: expr; ($len: expr)) => ( vec![$element; $len] ); }
    macro_rules! multi_box_array { ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_box_array![$element; ($($lens),*)]; $len].into_boxed_slice() ); ($element: expr; ($len: expr)) => ( vec![$element; $len].into_boxed_slice() ); }
    #[allow(unused_imports)]
    pub(super) use {chmax, chmin, max, min, multi_box_array, multi_vec};
}

// Calculate Stirling numbers of the second kind
pub fn stirling2(n: usize, k: usize) -> ModInt {
    if n < k {
        return ModInt::new(0);
    }

    let mut dp = macros::multi_box_array!(ModInt::new(0); (n + 1, k + 1));
    dp[0][0] = ModInt::new(1);

    for i in 1..=k {
        for j in 1..=n {
            dp[j][i] += dp[j - 1][i - 1];
            dp[j][i] += ModInt::new(i) * dp[j - 1][i];
        }
    }

    dp[n][k]
}

// Calculate Bell numbers
pub fn bell(n: usize, k: usize) -> ModInt {
    let mut dp = macros::multi_box_array!(ModInt::new(0); (n + 1, k + 1));
    dp[0][0] = ModInt::new(1);

    for i in 1..=k {
        for j in 1..=n {
            dp[j][i] += dp[j - 1][i - 1];
            dp[j][i] += ModInt::new(i) * dp[j - 1][i];
        }
    }

    dp[n].iter().fold(ModInt::new(0), |x, y| x + *y)
}
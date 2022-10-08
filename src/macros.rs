#[macro_export]
macro_rules! min {
    ($x: expr) => { $x };
    ($x: expr, $($xs: expr),+) => {{
        let y = min!($($xs),+);
        std::cmp::min($x, y)
    }}
}

#[macro_export]
macro_rules! max {
    ($x: expr) => { $x };
    ($x: expr, $($xs: expr),+) => {{
        let y = max!($($xs),+);
        std::cmp::max($x, y)
    }}
}

#[macro_export]
macro_rules! chmin {
    ($x: expr, $($xs: expr),+) => {{
        let y = min!($($xs),+);
        if $x > y { $x = y; true } else { false }
    }}
}

#[macro_export]
macro_rules! chmax {
    ($x: expr, $($xs: expr),+) => {{
        let y = max!($($xs),+);
        if $x < y { $x = y; true } else { false }
    }}
}

#[macro_export]
macro_rules! multi_vec {
    ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_vec![$element; ($($lens),*)]; $len] );
    ($element: expr; ($len: expr)) => ( vec![$element; $len] );
}

#[macro_export]
macro_rules! multi_box_array {
    ($element: expr; ($len: expr, $($lens: expr),*)) => ( vec![macros::multi_box_array![$element; ($($lens),*)]; $len].into_boxed_slice() );
    ($element: expr; ($len: expr)) => ( vec![$element; $len].into_boxed_slice() );
}

#[macro_export]
macro_rules! read_line {
    ($($xs: tt)*) => {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        let mut iter = buf.split_whitespace();
        expand!(iter, $($xs)*);
    };
}

#[macro_export]
macro_rules! expand {
    ($iter: expr,) => {};
    ($iter: expr, mut $var: ident : $type: tt, $($xs: tt)*) => {
        let mut $var = value!($iter, $type);
        expand!($iter, $($xs)*);
    };
    ($iter: expr, $var: ident : $type: tt, $($xs: tt)*) => {
        let $var = value!($iter, $type);
        expand!($iter, $($xs)*);
    };
}

#[macro_export]
macro_rules! value {
    ($iter:expr, ($($type: tt),*)) => {
        ($(value!($iter, $type)),*)
    };
    ($iter: expr, [$type: tt; $len: expr]) => {
        (0..$len).map(|_| value!($iter, $type)).collect::<Vec<_>>()
    };
    ($iter: expr, Chars) => {
        value!($iter, String).unwrap().chars().collect::<Vec<_>>()
    };
    ($iter: expr, $type: ty) => {
        if let Some(v) = $iter.next() {
            v.parse::<$type>().ok()
        } else {
            None
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn min_test() {
        assert_eq!(1, min!(3, 1));
        assert_eq!(1, min!(3, 1, 4, 1, 5));
    }

    #[test]
    fn max_test() {
        assert_eq!(3, max!(3, 1));
        assert_eq!(5, max!(3, 1, 4, 1, 5));
    }

    #[test]
    fn chmin_test() {
        let mut x = 100;
        let f = chmin!(x, 50);
        assert_eq!(f, true);
        assert_eq!(x, 50);

        let f = chmin!(x, 70);
        assert_eq!(f, false);
        assert_eq!(x, 50);

        let f = chmin!(x, 30, 20);
        assert_eq!(f, true);
        assert_eq!(x, 20);
    }

    #[test]
    fn chmax_test() {
        let mut x = 1;
        let f = chmax!(x, 20);
        assert_eq!(f, true);
        assert_eq!(x, 20);

        let f = chmax!(x, 10);
        assert_eq!(f, false);
        assert_eq!(x, 20);

        let f = chmax!(x, 10, 15);
        assert_eq!(f, false);
        assert_eq!(x, 20);

        let f = chmax!(x, 100, 30);
        assert_eq!(f, true);
        assert_eq!(x, 100);
    }
}

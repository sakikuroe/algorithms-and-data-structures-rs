use crate::data_structures::graph::Graph;

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

pub fn traveling_salesman_problem(g: &Graph<usize>) -> Option<usize> {
    fn rec(
        set: usize,
        v: usize,
        memo: &mut Vec<Vec<Option<usize>>>,
        g: &Graph<usize>,
    ) -> Option<usize> {
        if let Some(res) = memo[set][v] {
            return Some(res);
        }

        let res = if (set, v) == (0, 0) {
            Some(0)
        } else if set & (1 << v) == 0 {
            None
        } else {
            let mut res = std::usize::MAX;
            for e in &g.edges[v] {
                if let Some(dist) = rec(set & !(1 << e.src), e.dst, memo, g) {
                    macros::chmin!(res, dist + e.weight);
                }
            }

            if res != std::usize::MAX {
                Some(res)
            } else {
                None
            }
        };

        memo[set][v] = res;
        res
    }

    let mut memo = vec![vec![None; g.len()]; 1 << g.len()];
    rec((1 << g.len()) - 1, 0, &mut memo, &g)
}

//! verified by
//! - Library Checker | [Line Add Get Min](https://judge.yosupo.jp/problem/line_add_get_min) ([submittion](https://judge.yosupo.jp/submission/105957))

use super::splay_bst::SplayBST;
use std::cmp::Ordering;

const INF: isize = std::isize::MAX;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Line {
    a: isize,
    b: isize,
}

impl Line {
    pub fn value(&self, x: isize) -> isize {
        self.a * x + self.b
    }
}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.a).cmp(&(other.a))
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ConvexHullTrickMin {
    set: SplayBST<Line>,
}

impl ConvexHullTrickMin {
    pub fn new() -> Self {
        ConvexHullTrickMin {
            set: SplayBST::new(),
        }
    }

    pub fn add(&mut self, a: isize, b: isize) {
        let need = |left: &Line, center: &Line, right: &Line| {
            ((left.a - center.a) as i128) * ((right.b - center.b) as i128)
                < ((right.a - center.a) as i128) * ((left.b - center.b) as i128)
        };

        if self.set.len() == 0 {
            self.set.insert(Line { a, b });
        } else if self.set.len() == 1 {
            let line = self.set.get_nth(0).unwrap();
            if line.a != a {
                self.set.insert(Line { a, b });
            } else {
                if line.b > b {
                    self.set.remove(line);
                    self.set.insert(Line { a, b });
                }
            }
        } else {
            let mut i = unsafe { (*self.set.root).lower_bound(Line { a, b }) };

            if i == 0
                || i == self.set.len()
                || need(
                    &self.set.get_nth(i - 1).unwrap(),
                    &(Line { a, b }),
                    &self.set.get_nth(i).unwrap(),
                )
            {
                self.set.insert(Line { a, b });
                while i + 2 < self.set.len()
                    && !need(
                        &self.set.get_nth(i).unwrap(),
                        &self.set.get_nth(i + 1).unwrap(),
                        &self.set.get_nth(i + 2).unwrap(),
                    )
                {
                    let line = self.set.get_nth(i + 1).unwrap();
                    self.set.remove(line);
                }
                while i >= 2
                    && !need(
                        &self.set.get_nth(i - 2).unwrap(),
                        &self.set.get_nth(i - 1).unwrap(),
                        &self.set.get_nth(i).unwrap(),
                    )
                {
                    let line = self.set.get_nth(i - 1).unwrap();
                    self.set.remove(line);
                    i -= 1;
                }
            }
        }
    }

    pub fn get_max(&mut self, x: isize) -> isize {
        if self.set.len() == 0 {
            INF
        } else if self.set.len() == 1 {
            let line = self.set.get_nth(0).unwrap();
            line.value(x)
        } else {
            let mut ok = 0;
            let mut ng = self.set.len();
            while ng - ok > 1 {
                let mid = (ng + ok) / 2;
                if {
                    mid + 1 < self.set.len() && {
                        let line1 = self.set.get_nth(mid).unwrap();
                        let line2 = self.set.get_nth(mid + 1).unwrap();
                        line1.value(x) > line2.value(x)
                    }
                } {
                    ok = mid;
                } else {
                    ng = mid;
                }
            }

            let mut ans = std::isize::MAX;

            {
                let line = self.set.get_nth(ok).unwrap();
                macros::chmin!(ans, line.value(x));
            }
            {
                if ok + 1 < self.set.len() {
                    let line = self.set.get_nth(ok + 1).unwrap();
                    macros::chmin!(ans, line.value(x));
                }
            }

            ans
        }
    }
}


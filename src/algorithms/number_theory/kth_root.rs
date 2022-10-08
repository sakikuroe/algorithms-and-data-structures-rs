//! verified by
//! - Library Checker | [Kth Root (Integer)](https://judge.yosupo.jp/problem/kth_root_integer) ([submittion](https://judge.yosupo.jp/submission/106215))

pub fn kth_root(a: usize, k: usize) -> usize {
    if k == 1 || a <= 1 {
        a
    } else {
        let mut ok = 1;
        let mut ng = a.min(1usize << 32);
        let f = |mid: usize, a| match mid.checked_pow(k as u32) {
            Some(v) => v <= a,
            None => false,
        };
        while ng - ok > 1 {
            let mid = (ng + ok) / 2;
            if f(mid, a) {
                ok = mid;
            } else {
                ng = mid;
            }
        }

        ok
    }
}

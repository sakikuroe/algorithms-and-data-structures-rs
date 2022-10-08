//! verified by
//! - AtCoder | [トヨタ自動車プログラミングコンテスト2022(AtCoder Beginner Contest 270)](https://atcoder.jp/contests/abc270/tasks/abc270_g), ([submittion](https://atcoder.jp/contests/abc270/submissions/35453202))
use std::collections::BTreeMap;

/// fn baby_step_giant_step return minimum k ∈ ℕ s.t. f^{k}(s) = g
/// - f: bijective function and ∃p ∈ ℕ s.t. f^{p} = id_{T} (i.e. f is periodic.)
/// - bs := ⌊√p⌋ ∈ ℕ
/// - fff := f^{bs}
///
/// ∀k < p ∈ ℕ, ∃i, j ∈ ℕ (0 ≤ i < ⌈p / ⌊√p⌋⌉, 0 ≤ j < ⌊√p⌋) s.t. k = i * ⌊√p⌋ + j
/// <=> ∀k ∈ ℕ, ∃i, j ∈ ℕ (1 ≤ i ≤ ⌈p / ⌊√p⌋⌉, 0 ≤ j < ⌊√p⌋) s.t. k = i * ⌊√p⌋ + j - ⌈√p⌉
/// <=> ∀k ∈ ℕ, ∃i, j ∈ ℕ (1 ≤ i ≤ ⌈p / ⌊√p⌋⌉, 1 ≤ j ≤ ⌊√p⌋) s.t. k = i * ⌊√p⌋ - j
///
/// ∃k ∈ ℕ s.t. f^{k}(s) = g
/// <=> ∃i, j ∈ ℕ, (1 ≤ i ≤ ⌈p / ⌊√p⌋⌉, 1 ≤ j ≤ ⌊√p⌋) s.t. fff^{i}(s) = f^{j}(g)
pub fn baby_step_giant_step<T, F, G>(s: T, g: T, bs: usize, fff: F, f: G) -> Option<usize>
where
    T: Copy + Clone + Ord + Eq + PartialEq + PartialOrd,
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    let mut f_x = BTreeMap::new();
    let mut x = g;
    for j in 1..=bs {
        x = f(x);
        f_x.insert(x, j);
    }

    let mut x = s;
    for i in 1..=(bs + 1) * (bs + 1) / bs {
        x = fff(x);
        if let Some(j) = f_x.get(&x) {
            return Some(bs * i - *j);
        }
    }

    None
}

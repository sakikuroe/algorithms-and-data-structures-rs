//! verified by
//! - Library Checker | [Z Algorithm](https://judge.yosupo.jp/problem/zalgorithm), ([submittion](https://judge.yosupo.jp/submission/100387))
//! - Aizu Online Judge | [ALDS1_14_B](https://onlinejudge.u-aizu.ac.jp/problems/ALDS1_14_B), ([submittion](https://onlinejudge.u-aizu.ac.jp/solutions/problem/ALDS1_14_B/review/6897161/Kurosaki96/Rust))
//! - AtCoder | [AtCoder Beginner Contest 141 E - Who Says a Pun?](https://atcoder.jp/contests/abc141/tasks/abc141_e), ([submittion](https://atcoder.jp/contests/abc141/submissions/34112357))
//! - AtCoder | [AtCoder Beginner Contest 135 F - Strings of Eternity](https://atcoder.jp/contests/abc135/tasks/abc135_f), ([submittion](https://atcoder.jp/contests/abc135/submissions/37611480))

/// Obtain a vector consisting of the lengths of the longest common prefix array of S and S[i..].
///
/// ```
/// use algorithms_and_data_structures_rs::algorithms::string::z_algorithm::z_algorithm;
///
/// let s = "abaaabcaba".chars().collect::<Vec<_>>();
/// assert_eq!(vec![10, 0, 1, 1, 2, 0, 0, 3, 0, 1], z_algorithm(&s));
/// ```

pub fn z_algorithm<T>(v: &Vec<T>) -> Vec<usize>
where
    T: Ord,
{
    let mut z = vec![0; v.len()];
    z[0] = v.len();
    let mut i = 1;
    let mut j = 0;
    while i < v.len() {
        while i + j < v.len() && v[j] == v[i + j] {
            j += 1;
        }
        z[i] = j;
        if j == 0 {
            i += 1;
            continue;
        }
        let mut k = 1;
        while k < j && k + z[k] < j {
            z[i + k] = z[k];
            k += 1;
        }
        i += k;
        j -= k;
    }

    z
}

pub fn string_search<T>(s: &Vec<T>, t: &Vec<T>) -> Vec<usize>
where
    T: Ord + Clone,
{
    let mut v = t.clone();
    v.append(&mut s.clone());
    let z = z_algorithm(&v);

    (0..s.len())
        .filter(|i| z[*i + t.len()] >= t.len())
        .collect::<Vec<_>>()
}

pub fn string_search_cyclic<T>(s: &Vec<T>, t: &Vec<T>) -> Vec<usize>
where
    T: Ord + Clone,
{
    let ss = {
        let mut res = vec![];
        res.append(&mut s.clone());
        res.append(&mut s.clone());
        while res.len() < t.len() * 2 {
            res.append(&mut s.clone());
        }
        res
    };

    let mut v = t.clone();
    v.append(&mut ss.clone());
    let z = z_algorithm(&v);

    (0..s.len())
        .filter(|i| z[*i + t.len()] >= t.len())
        .collect::<Vec<_>>()
}

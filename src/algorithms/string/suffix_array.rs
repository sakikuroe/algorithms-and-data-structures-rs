use std::cmp::Ordering;

/// verified by
/// - Library Checker | [Suffix Array](https://judge.yosupo.jp/problem/suffixarray) ([submittion](https://judge.yosupo.jp/submission/107819))
pub fn suffix_array(s: &Vec<char>) -> Vec<usize> {
    let mut sa = (0..s.len() + 1).collect::<Vec<_>>();
    let mut rank = vec![0; s.len() + 1];
    for i in 0..s.len() {
        rank[i] = s[i] as isize;
    }
    rank[s.len()] = -1;

    for k in (0..).map(|i| 1 << i).take_while(|x| *x <= s.len()) {
        let cmp_sa = |i, j| -> Ordering {
            if rank[i] != rank[j] {
                return (rank[i] as isize).cmp(&rank[j]);
            } else {
                let ranki = if i + k <= s.len() { rank[i + k] } else { -1 };
                let rankj = if j + k <= s.len() { rank[j + k] } else { -1 };
                return ranki.cmp(&rankj);
            }
        };

        sa.sort_by(|x, y| (cmp_sa(*x, *y)));
        let mut temp = vec![0; s.len() + 1];
        temp[sa[0]] = 0;

        for i in 1..=s.len() {
            temp[sa[i]] = temp[sa[i - 1]]
                + if cmp_sa(sa[i - 1], sa[i]) == Ordering::Less {
                    1
                } else {
                    0
                };
        }
        rank = temp;
    }

    sa
}

pub fn lcp(s: &Vec<char>, sa: &Vec<usize>) -> Vec<usize> {
    let mut lcp = vec![];
    let mut li = 0;
    let mut rank = vec![0; s.len()];
    for i in 0..s.len() {
        rank[sa[i + 1]] = i;
    }
    for p in 0..s.len() {
        let i = rank[p];
        if i > 0 {
            while p + li < s.len() && sa[i] + li < s.len() && s[p + li] == s[sa[i] + li] {
                li += 1;
            }
        } else {
            li = 0;
        }
        lcp.push(li);
        if li > 0 {
            li -= 1;
        }
    }

    lcp
}

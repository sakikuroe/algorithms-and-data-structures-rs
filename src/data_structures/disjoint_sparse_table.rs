/// - Library Checker | [Static RMQ](https://judge.yosupo.jp/problem/staticrmq), ([submittion](https://judge.yosupo.jp/submission/116679))

pub struct DisjointSparseTable<M>
where
    M: Monoid,
{
    m: usize,
    height: usize,
    nodes: Vec<Vec<Vec<M::S>>>,
}

impl<M> DisjointSparseTable<M>
where
    M: Monoid,
    M::S: Clone,
{
    pub fn new(a: &[M::S]) -> Self {
        let m = a.len().next_power_of_two();
        let mut a = a.to_vec();
        a.resize(m, M::id());
        let height = std::cmp::max(m.trailing_zeros() as usize, 1);
        let mut nodes: Vec<Vec<Vec<M::S>>> = vec![vec![]; height];
        for i in 0..height {
            for (idx, j) in (0..m)
                .step_by(std::cmp::max(m / (1 << (i + 1)), 1))
                .enumerate()
            {
                let mut u = vec![];
                let mut sum = M::id();
                if idx % 2 == 0 {
                    for x in a[j..j + std::cmp::max(m / (1 << (i + 1)), 1)].iter().rev() {
                        sum = M::op(&sum, x);
                        u.push(sum.clone());
                    }
                } else {
                    for x in a[j..j + std::cmp::max(m / (1 << (i + 1)), 1)].iter() {
                        sum = M::op(&sum, x);
                        u.push(sum.clone());
                    }
                }
                nodes[i].push(u);
            }
        }

        DisjointSparseTable { m, height, nodes }
    }

    pub fn fold(&self, l: usize, r: usize) -> M::S {
        if r - l == 1 {
            return self.nodes[self.height - 1][l][0].clone();
        }
        let most_significant_bit =
            |x: usize| -> usize { self.height - (64 - (x as u64).leading_zeros() as usize) };
        let i = most_significant_bit(l ^ (r - 1));
        let j_l = l / (self.m / (1 << (i + 1)));
        let j_r = (r - 1) / (self.m / (1 << (i + 1)));
        let mid = (r - 1) / (self.m / (1 << (i + 1))) * (self.m / (1 << (i + 1)));
        let k_l = mid - l - 1;
        let k_r = r - 1 - mid;
        M::op(&self.nodes[i][j_l][k_l], &self.nodes[i][j_r][k_r])
    }
}
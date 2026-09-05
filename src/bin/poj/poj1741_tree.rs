// POJ: 1741 - Tree
// http://poj.org/problem?id=1741

use anmitsu::graph::graph::Graph;
use anmitsu::io::fastio::Fastio;

/// 昇順に並んだ距離の列から、和が `k` 以下になる (添字の) ペアの個数を
/// 二分探索ではなく、両端からの二本の指針 (two-pointer) で数える。
fn count_pairs_within(sorted_dist: &[i64], k: i64) -> i64 {
    if sorted_dist.len() < 2 {
        return 0;
    }
    let mut lo = 0;
    let mut hi = sorted_dist.len() - 1;
    let mut count = 0_i64;
    while lo < hi {
        if sorted_dist[lo] + sorted_dist[hi] <= k {
            count += (hi - lo) as i64;
            lo += 1;
        } else {
            hi -= 1;
        }
    }
    count
}

fn main() {
    let mut io = Fastio::new();

    loop {
        let n = io.u32() as usize;
        let k = io.i64();
        if n == 0 && k == 0 {
            break;
        }

        let mut g = Graph::new(n);
        for _ in 0..n - 1 {
            let u = io.u32() as usize - 1;
            let v = io.u32() as usize - 1;
            let l = io.i64();
            g.add_undirected_edge(u, v, l);
        }

        let cd = g.try_centroid_decomposition().unwrap();

        let mut answer = 0_i64;
        cd.for_each_component_by(
            &g,
            |&w| w,
            |_centroid, whole, branches| {
                let mut sorted_whole = whole.to_vec();
                sorted_whole.sort_unstable();
                answer += count_pairs_within(&sorted_whole, k);

                for branch in branches {
                    let mut sorted_branch = branch.clone();
                    sorted_branch.sort_unstable();
                    answer -= count_pairs_within(&sorted_branch, k);
                }
            },
        );

        io.writeln(answer);
    }

    io.flush();
}

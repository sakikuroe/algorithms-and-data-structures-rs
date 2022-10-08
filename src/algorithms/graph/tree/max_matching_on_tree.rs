use crate::data_structures::graph::Graph;

pub fn get_traversal(start: usize, g: &Graph<usize>) -> Vec<usize> {
    fn dfs(v: usize, p: Option<usize>, g: &Graph<usize>, traversal: &mut Vec<usize>) {
        for e in &g.edges[v] {
            if p == Some(e.dst) {
                continue;
            }
            dfs(e.dst, Some(v), g, traversal);
        }
        traversal.push(v);
    }

    let mut traversal = vec![];
    dfs(start, None, g, &mut traversal);
    traversal
}

pub fn get_max_matching_on_tree(g: &Graph<usize>) -> usize {
    let mut res = 0;
    let mut used = vec![false; g.len()];
    for v in get_traversal(0, g) {
        if used[v] {
            continue;
        }
        used[v] = true;

        for e in g.edges[v].iter().filter(|e| !used[e.dst]) {
            if !used[e.dst] {
                used[e.dst] = true;
                res += 1;
                break;
            }
        }
    }

    res
}

pub fn is_complete_matching(g: &Graph<usize>) -> bool {
    g.len() == 2 * get_max_matching_on_tree(g)
}

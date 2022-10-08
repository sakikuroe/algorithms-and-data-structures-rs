pub fn mex(v: &Vec<usize>) -> usize {
    let mut f = vec![false; v.len()];
    for &x in v.iter().filter(|&x| *x < v.len()) {
        f[x] = true;
    }
    for i in 0..v.len() {
        if !f[i] {
            return i;
        }
    }

    v.len()
}

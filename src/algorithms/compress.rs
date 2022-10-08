use super::search::binary_search::BinarySearch;

/// return (cordinate, scale)
/// for all i, cordinate[i] < v.len()
/// for all i, v[i] = scale[cordinate[i]]
pub fn compress<T>(v: &[T]) -> (Vec<usize>, Vec<T>)
where
    T: Ord + Clone,
{
    let mut scale = v.to_vec();
    scale.sort();
    scale.dedup();

    let mut cordinate = vec![];
    for a in v {
        cordinate.push(scale.lower_bound(a));
    }

    (cordinate, scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_test1() {
        let v: Vec<i32> = vec![3, 1, 4, 1, 5, 9];
        let (cordinate, scale) = compress(&v);
        assert_eq!(cordinate, vec![1, 0, 2, 0, 3, 4]);
        assert_eq!(scale, vec![1, 3, 4, 5, 9]);
        for i in 0..v.len() {
            assert_eq!(v[i], scale[cordinate[i]]);
        }
    }

    #[test]
    fn compress_test2() {
        let v: Vec<usize> = vec![
            3000000000000, 1000000000000, 4000000000000, 1000000000000, 5000000000000, 9000000000000,
        ];
        let (cordinate, scale) = compress(&v);
        assert_eq!(cordinate, vec![1, 0, 2, 0, 3, 4]);
        assert_eq!(
            scale,
            vec![1000000000000, 3000000000000, 4000000000000, 5000000000000, 9000000000000]
        );
        for i in 0..v.len() {
            assert_eq!(v[i], scale[cordinate[i]]);
        }
    }
}

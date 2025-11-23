pub mod algebra {
    pub mod monoid;
    pub mod semi_group;
}

pub mod ds {
    pub mod modint {
        pub mod modint998244353;
    }
    pub mod segment_tree {
        pub mod segment_tree_dense;
    }
    pub mod bit_vector;
    pub mod union_find;
    pub mod wavelet_matrix;
}

pub mod math {
    pub mod number_theory;
}

pub mod string {
    pub mod run_length;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

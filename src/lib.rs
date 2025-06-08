pub mod algebra {
    pub mod monoid;
    pub mod semi_group;
}

pub mod ds {
    pub mod segment_tree {
        pub mod segment_tree_dense;
    }
}

pub mod math {
    pub mod number_theory;
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

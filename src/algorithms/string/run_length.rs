trait RunLength {
    fn run_length_encode(&self) -> Vec<(char, usize)>;
}

impl RunLength for Vec<char> {
    fn run_length_encode(&self) -> Vec<(char, usize)> {
        let mut res: Vec<(char, usize)> = Vec::new();
        for x in self.iter() {
            if res.is_empty() || res.iter().last().unwrap().0 != *x {
                res.push((*x, 1));
            } else {
                res.iter_mut().last().unwrap().1 += 1;
            }
        }

        res
    }
}

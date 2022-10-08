use crate::algorithms::search::binary_search::BinarySearch;

pub trait LIS<T> {
    fn lis_weakly(&self) -> Vec<T>;
    fn lis_strictly(&self) -> Vec<T>;
}

impl<T> LIS<T> for [T]
where
    T: Clone + Ord,
{
    fn lis_weakly(&self) -> Vec<T> {
        let mut dp = vec![];
        let mut prev = vec![None; self.len()];
        for i in 0..self.len() {
            let j = dp.upper_bound(&self[i]);
            if dp.len() == j {
                dp.push(self[i].clone());
            } else {
                dp[j] = self[i].clone()
            }
            if j > 0 {
                prev[i] = Some(dp[j - 1].clone());
            }
        }

        if let Some(temp) = dp.last() {
            let mut look_for = temp.clone();
            let mut res = vec![];
            for i in (0..self.len()).rev() {
                if self[i] == look_for {
                    res.push(look_for);
                    if let Some(next) = prev[i].clone() {
                        look_for = next;
                    } else {
                        break;
                    }
                }
            }
            res.reverse();
            res
        } else {
            vec![]
        }
    }

    fn lis_strictly(&self) -> Vec<T> {
        let mut dp = vec![];
        let mut prev = vec![None; self.len()];
        for i in 0..self.len() {
            let j = dp.lower_bound(&self[i]);
            if dp.len() == j {
                dp.push(self[i].clone());
            } else {
                dp[j] = self[i].clone()
            }
            if j > 0 {
                prev[i] = Some(dp[j - 1].clone());
            }
        }

        if let Some(temp) = dp.last() {
            let mut look_for = temp.clone();
            let mut res = vec![];
            for i in (0..self.len()).rev() {
                if self[i] == look_for {
                    res.push(look_for);
                    if let Some(next) = prev[i].clone() {
                        look_for = next;
                    } else {
                        break;
                    }
                }
            }
            res.reverse();
            res
        } else {
            vec![]
        }
    }
}

pub trait LDS<T> {
    fn lds_weakly(&self) -> Vec<T>;
    fn lds_strictly(&self) -> Vec<T>;
}

impl<T> LDS<T> for [T]
where
    T: Clone + Ord,
{
    fn lds_weakly(&self) -> Vec<T> {
        let rev_self = {
            let mut temp = self.clone().to_vec();
            temp.reverse();
            temp
        };

        let res = {
            let mut temp = rev_self.lis_weakly();
            temp.reverse();
            temp
        };
        res
    }

    fn lds_strictly(&self) -> Vec<T> {
        let rev_self = {
            let mut temp = self.clone().to_vec();
            temp.reverse();
            temp
        };

        let res = {
            let mut temp = rev_self.lis_strictly();
            temp.reverse();
            temp
        };
        res
    }
}

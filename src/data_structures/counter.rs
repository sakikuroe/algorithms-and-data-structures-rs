use std::{cmp, collections::BTreeMap, mem::ManuallyDrop, ptr};

#[derive(Clone)]
pub struct Counter<T> {
    pub map: BTreeMap<T, isize>,
}

impl<T> Counter<T>
where
    T: Eq + Clone + Ord,
{
    pub fn new() -> Self {
        Counter {
            map: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, key: T, cnt: isize) {
        *self.map.entry(key.clone()).or_insert(0) += cnt;
        if self.get(&key) == 0 {
            self.map.remove(&key);
        }
    }

    pub fn get(&self, key: &T) -> isize {
        *self.map.get(key).unwrap_or(&0)
    }

    pub fn get_items(&self) -> Vec<(T, isize)> {
        self.map.clone().into_iter().collect::<Vec<_>>()
    }

    pub fn get_keys(&self) -> Vec<T> {
        self.map.clone().into_iter().map(|x| x.0).collect()
    }

    pub fn get_values(&self) -> Vec<isize> {
        self.map.iter().map(|x| *x.1).collect()
    }

    pub fn pop_min(&mut self) -> Option<T> {
        unsafe {
            if let Some((key, _cnt)) = self.map.iter().next() {
                let mut mdkey = ManuallyDrop::new(ptr::read(key));
                self.add(ManuallyDrop::take(&mut mdkey), -1);
                Some(ManuallyDrop::take(&mut mdkey))
            } else {
                None
            }
        }
    }

    pub fn pop_max(&mut self) -> Option<T> {
        unsafe {
            if let Some((key, _cnt)) = self.map.iter().last() {
                let mut mdkey = ManuallyDrop::new(ptr::read(key));
                self.add(ManuallyDrop::take(&mut mdkey), -1);
                Some(ManuallyDrop::take(&mut mdkey))
            } else {
                None
            }
        }
    }

    pub fn is_subset_of(&self, other: &Counter<T>) -> bool {
        for (k, v) in self.get_items() {
            if v > other.get(&k) {
                return false;
            }
        }

        true
    }
}

pub fn union<T>(sa: &Counter<T>, sb: &Counter<T>) -> Counter<T>
where
    T: Eq + Clone + Ord,
{
    let mut res = sa.clone();
    for (k, &v) in sb.map.iter() {
        let e = res.map.entry(k.clone()).or_insert(0);
        *e = cmp::max(*e, v);
    }

    res
}

pub fn intersection<T>(sa: &Counter<T>, sb: &Counter<T>) -> Counter<T>
where
    T: Eq + Clone + Ord,
{
    let mut res = Counter::new();
    for (k, &v) in sa.map.iter() {
        res.add(k.clone(), cmp::min(v, *sb.map.get(k).unwrap_or(&0)));
    }

    res
}

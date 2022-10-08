use crate::data_structures::graph::Graph;
use std::cmp::Reverse;

impl Graph<usize> {
    pub fn diameter(&self) -> ((usize, usize), usize) {
        let dijkstra1 = self.dijkstra(0);
        let (mut diameter_src, _distance) = (0..self.len())
            .map(|i| (i, dijkstra1.get_distance(i).unwrap()))
            .max_by_key(|(i, d)| (*d, Reverse(*i)))
            .unwrap();

        let dijkstra2 = self.dijkstra(diameter_src);
        let (mut diameter_dst, diameter_distance) = (0..self.len())
            .map(|i| (i, dijkstra2.get_distance(i).unwrap()))
            .max_by_key(|(i, d)| (*d, Reverse(*i)))
            .unwrap();

        if diameter_src > diameter_dst {
            std::mem::swap(&mut diameter_src, &mut diameter_dst);
        }

        ((diameter_src, diameter_dst), diameter_distance)
    }
}

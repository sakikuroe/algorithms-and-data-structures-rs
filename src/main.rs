use algorithms_and_data_structures_rs::algorithms::permutation::gen_kth_in_lexicographical_order;
use algorithms_and_data_structures_rs::algorithms::permutation::Lexicographical;

fn main() {
    println!("{:?}", gen_kth_in_lexicographical_order(10, 1000000));
    println!(
        "{}",
        [2, 7, 8, 3, 9, 1, 5, 6, 0, 4].get_kth_in_lexicographical_order()
    );
}

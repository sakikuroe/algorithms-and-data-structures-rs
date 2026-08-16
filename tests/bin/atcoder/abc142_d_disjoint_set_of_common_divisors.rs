use rstest::rstest;

use super::super::common;

/// `ac-abc142-d-disjoint-set-of-common-divisors` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc142-d-disjoint-set-of-common-divisors");

// ac-abc142-d-disjoint-set-of-common-divisors のテスト: 標準入力にサンプルを
// 与えたときの標準出力を検証する
mod ac_abc142_d_disjoint_set_of_common_divisors {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc142-d-disjoint-set-of-common-divisors バイナリへ標準入力
    ///   として渡す
    /// - Then: 互いに素になるように選べる公約数の個数の最大値が期待値と一致する
    #[rstest]
    // - Given: gcd(12, 18) = 6 = 2 * 3 であり、素因数が 2 種類の合成数である
    #[case::sample_1("12 18\n", "3\n")]
    // - Given: gcd(420, 660) = 60 = 2^2 * 3 * 5 であり、素因数が 3 種類の
    //   合成数である
    #[case::sample_2("420 660\n", "4\n")]
    // - Given: gcd(1, 2019) = 1 であり、素因数を持たない (境界値)
    #[case::sample_3("1 2019\n", "1\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

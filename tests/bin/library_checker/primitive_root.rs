use anmitsu::math::primality;

use super::super::common;

/// `lc-primitive-root` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-primitive-root");

// lc-primitive-root のテスト: 標準入力にサンプルを与えたときの標準出力を検証する。
// この問題は特別ジャッジ (原始根は複数あり得る) であるため、標準出力を固定値と
// 完全一致させず、出力された値が実際に原始根としての性質を満たすことを検証する。
mod lc_primitive_root {
    use super::*;

    /// Scenario: 出力された値が、各問い合わせに対する原始根になっている
    /// - Given: 問題文の公式サンプルである、複数の素数への問い合わせがある
    /// - When: lc-primitive-root バイナリへ標準入力として渡す
    /// - Then: 出力された行数が問い合わせ数と一致し、各行の値が対応する素数を
    ///   法とする原始根の性質 (`0 <= a < p` かつ `is_primitive_root(a, p)`) を満たす
    #[test]
    fn produces_valid_primitive_root_for_each_query() {
        // Given
        let input = "8\n2\n3\n5\n7\n11\n13\n17\n19\n";
        let ps = [2_u64, 3, 5, 7, 11, 13, 17, 19];

        // When
        let output = common::run_binary(BIN, input);
        let outs = output
            .lines()
            .map(|line| line.parse::<u64>().unwrap())
            .collect::<Vec<u64>>();

        // Then
        assert_eq!(ps.len(), outs.len());
        for (&p, &a) in ps.iter().zip(outs.iter()) {
            assert!(a < p, "{a} is not less than {p}");
            assert!(
                primality::is_primitive_root(a, p),
                "{a} is not a primitive root modulo {p}"
            );
        }
    }
}

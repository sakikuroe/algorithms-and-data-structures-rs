use super::super::common;

/// `ac-abc234-d-prefix-kth-max` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc234-d-prefix-kth-max");

// ac-abc234-d-prefix-kth-max のテスト: 標準入出力を検証する。
mod ac_abc234_d_prefix_kth_max {
    use super::*;

    /// Scenario: prefix ごとの k 番目に大きい値を出力する。
    /// - Given: 問題文の公式サンプル 2 がある。
    /// - When: バイナリへ標準入力として渡す。
    /// - Then: 各 prefix の k-th max が公式出力と一致する。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "11 5\n3 7 2 5 11 6 1 9 8 10 4\n";
        let expected = "2\n3\n3\n5\n6\n7\n7\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use super::super::common;

/// `ac-abc441-g-takoyaki` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc441-g-takoyaki");

// ac-abc441-g-takoyaki のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod ac_abc441_g_takoyaki {
    use super::*;

    /// Scenario: 問題文の公式サンプル 1 を解いたときの標準出力を検証する。
    /// - Given: N=6, Q=6 で、加算・食事反転・最大値クエリが混在する入力である。
    /// - When: ac-abc441-g-takoyaki バイナリへ標準入力として渡す。
    /// - Then: 各 Type 3 クエリの出力が "4\n6\n2\n" と一致する。
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "\
6 6
1 3 5 4
3 2 3
1 1 6 2
2 3 4
3 1 6
3 2 3
";
        let expected = "\
4
6
2
";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 問題文の公式サンプル 2 を解いたときの標準出力を検証する。
    /// - Given: N=2, Q=8 で、片方の皿のみ反転し残りに加算を繰り返す入力である。
    /// - When: ac-abc441-g-takoyaki バイナリへ標準入力として渡す。
    /// - Then: 各 Type 3 クエリの出力が "0\n5000000000\n" と一致する。
    #[test]
    fn matches_official_sample_2() {
        // Given
        let input = "\
2 8
1 1 2 1000000000
1 1 2 1000000000
2 2 2
1 1 2 1000000000
1 1 2 1000000000
1 1 2 1000000000
3 2 2
3 1 2
";
        let expected = "\
0
5000000000
";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 問題文の公式サンプル 3 を解いたときの標準出力を検証する。
    /// - Given: N=24, Q=30 で、複雑な操作列が与えられる入力である。
    /// - When: ac-abc441-g-takoyaki バイナリへ標準入力として渡す。
    /// - Then: 各 Type 3 クエリの出力が期待する 11 行と一致する。
    #[test]
    fn matches_official_sample_3() {
        // Given
        let input = "\
24 30
1 11 24 4326
1 4 16 1149
1 14 20 2331
1 12 14 8930
1 22 23 6989
3 15 20
3 10 19
1 3 12 7988
1 18 23 8450
3 9 19
3 13 15
2 8 15
2 9 14
1 11 17 4062
1 6 15 1721
3 7 13
1 11 20 8541
1 8 10 3748
1 1 17 3252
2 9 23
2 1 23
3 2 22
1 5 23 7468
3 1 12
3 12 19
2 6 24
3 2 14
3 1 15
2 15 19
3 2 14
";
        let expected = "\
7806
16736
22393
16736
10858
0
7468
7468
0
0
0
";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// 密な 2 次元版の ABC106 D バイナリへのパス。
const DENSE_BIN: &str = env!("CARGO_BIN_EXE_ac-abc106-d-atcoder-express-2-2d-dense");
/// 疎な offline 2 次元版の ABC106 D バイナリへのパス。
const OFFLINE_BIN: &str = env!("CARGO_BIN_EXE_ac-abc106-d-atcoder-express-2-2d-sparse-offline");
/// 疎な online 2 次元版の ABC106 D バイナリへのパス。
const ONLINE_BIN: &str = env!("CARGO_BIN_EXE_ac-abc106-d-atcoder-express-2-2d-sparse-online");

/// 3 種の 2 次元セグメント木で列車の区間包含を数える。
mod ac_abc106_d_atcoder_express_2_2d {
    use super::*;

    /// Scenario: 公式入力で区間内に完結する列車の本数を得る。
    /// - Given: 公式サンプル 3 件の入力がある。
    /// - When: 3 種の実装それぞれを使ったバイナリに渡す。
    /// - Then: 全クエリの答えが公式出力と一致する。
    #[rstest]
    #[case::sample1("2 3 1\n1 1\n1 2\n2 2\n1 2\n", "3\n")]
    #[case::sample2("10 3 2\n1 5\n2 8\n7 10\n1 7\n3 10\n", "1\n1\n")]
    #[case::sample3(
        "10 10 10\n1 6\n2 9\n4 5\n4 7\n4 7\n5 8\n6 6\n6 7\n7 9\n10 10\n1 8\n1 9\n1 10\n2 8\n2 9\n2 10\n3 8\n3 9\n3 10\n1 10\n",
        "7\n9\n10\n6\n8\n9\n6\n7\n8\n10\n"
    )]
    fn matches_official_samples(
        #[values(DENSE_BIN, OFFLINE_BIN, ONLINE_BIN)] bin: &str,
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // When
        let result = common::run_binary(bin, input);
        // Then
        assert_eq!(expected, result);
    }
}

use rstest::rstest;

use super::super::common;

/// `ac-typical90-q-crossing-segments` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-typical90-q-crossing-segments");

// ac-typical90-q-crossing-segments のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_typical90_q_crossing_segments {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する。
    /// - Given: 問題文の公式サンプルである。
    /// - When: ac-typical90-q-crossing-segments バイナリへ標準入力として渡す。
    /// - Then: 交差する組数が期待値と一致する。
    #[rstest]
    #[case::sample_1("6 3\n2 5\n1 4\n1 3\n", "2\n")]
    #[case::sample_2(
        "250 10\n13 218\n17 99\n24 180\n53 115\n96 97\n111 158\n124 164\n135 227\n158 177\n204 224\n",
        "10\n"
    )]
    #[case::sample_3("100 10\n1 2\n1 3\n1 4\n1 5\n1 6\n1 7\n1 8\n1 9\n1 10\n1 11\n", "0\n")]
    #[case::sample_4(
        "100 10\n1 100\n2 99\n3 98\n4 97\n5 96\n6 95\n7 94\n8 93\n9 92\n10 91\n",
        "0\n"
    )]
    #[case::sample_5(
        "1000 40\n12 43\n23 59\n32 118\n44 751\n68 136\n70 168\n85 328\n88 809\n92 981\n95 540\n98 772\n98 903\n125 896\n173 737\n199 325\n212 369\n227 587\n230 374\n287 442\n306 926\n314 858\n316 371\n318 493\n337 506\n384 887\n387 493\n394 457\n404 652\n414 527\n422 920\n441 730\n445 620\n468 602\n482 676\n568 857\n587 966\n653 757\n710 928\n764 927\n778 916\n",
        "229\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

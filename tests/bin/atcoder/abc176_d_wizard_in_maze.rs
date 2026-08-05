use rstest::rstest;

use super::super::common;

/// `ac-abc176-d-wizard-in-maze` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc176-d-wizard-in-maze");

// ac-abc176-d-wizard-in-maze のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod ac_abc176_d_wizard_in_maze {
    use super::*;

    /// Scenario: 問題文の公式サンプルを解いたときの標準出力を検証する
    /// - When: ac-abc176-d-wizard-in-maze バイナリへ標準入力として渡す
    /// - Then: 目的マスに到達するために必要な魔法の最小回数 (または `-1`) が期待通りである
    #[rstest]
    // - Given: 通常移動だけでは迂回が必要だが、魔法1回で目的マスへ届くグリッドである
    #[case::sample_1("4 4\n1 1\n4 4\n..#.\n..#.\n.#..\n.#..\n", "1\n")]
    // - Given: 壁に阻まれ、目的マスへ到達できないグリッドである
    #[case::sample_2_unreachable("4 4\n1 4\n4 1\n.##.\n####\n####\n.##.\n", "-1\n")]
    // - Given: 開始マスと目的マスが通常移動のみで到達できるグリッドである
    #[case::sample_3_no_magic_needed("4 4\n2 2\n3 3\n....\n....\n....\n....\n", "0\n")]
    // - Given: 魔法2回を要する、より複雑な壁の配置のグリッドである
    #[case::sample_4("4 5\n1 2\n2 5\n#.###\n####.\n#..##\n#..##\n", "2\n")]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

//! ABC435 E の実行バイナリーを標準入出力で検証します。

use super::super::common;

/// ABC435 E を解くバイナリーのパスです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc435-e-cover-query");

// 統合テスト: 区間更新、変更個数の集計、入出力をまとめて検証します。
mod ac_abc435_e_cover_query {
    use super::*;

    /// Scenario: 重複する塗りつぶしを含む公式サンプル 1 を解きます。
    /// - Given: 10 マスと、重複・包含・単点を含む 5 回の塗りつぶしがあります。
    /// - When: バイナリーに公式サンプル 1 を入力します。
    /// - Then: 白マスの個数が順に 7、5、3、2、2 になります。
    #[test]
    fn matches_official_sample_1() {
        // Given
        let input = "10 5\n3 5\n8 9\n5 8\n2 9\n6 6\n";
        let expected = "7\n5\n3\n2\n2\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: マス数が大きい公式サンプル 2 を解きます。
    /// - Given: 10 億マスの左半分を塗りつぶす入力があります。
    /// - When: バイナリーに公式サンプル 2 を入力します。
    /// - Then: 白マスの個数は 5 億になります。
    #[test]
    fn matches_official_sample_2() {
        // Given
        let input = "1000000000 1\n1 500000000\n";
        let expected = "500000000\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 隣接区間の結合後に全体を塗りつぶします。
    /// - Given: 6 マスと、隣接区間・既存区間・全体の更新があります。
    /// - When: バイナリーに各更新を入力します。
    /// - Then: 再塗装では個数が減らず、全体更新後は 0 になります。
    #[test]
    fn handles_adjacent_repeated_and_full_updates() {
        // Given
        let input = "6 5\n2 3\n4 5\n3 4\n1 6\n6 6\n";
        let expected = "4\n2\n2\n0\n0\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

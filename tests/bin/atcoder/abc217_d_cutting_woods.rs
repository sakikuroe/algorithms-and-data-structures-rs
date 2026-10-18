//! ABC217 D のバイナリーを、公式サンプルと切断点の単純な探索で検証します。

use super::super::common;

/// ABC217 D の実行バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc217-d-cutting-woods");

// 統合テスト: 切断で分割される 0 の最大連続区間を検証します。
mod ac_abc217_d_cutting_woods {
    use super::*;

    /// Scenario: 公式サンプルの木材の長さを求めます。
    /// - Given: 問題文の公式サンプルがあります。
    /// - When: バイナリーで切断と長さの取得を処理します。
    /// - Then: 問い合わせの出力が公式の出力と一致します。
    #[rstest::rstest]
    #[case::sample_1("5 3\n2 2\n1 3\n2 2\n", "5\n3\n")]
    #[case::sample_2("5 3\n1 2\n1 4\n2 3\n", "2\n")]
    #[case::sample_3(
        "100 10\n1 31\n2 41\n1 59\n2 26\n1 53\n2 58\n1 97\n2 93\n1 23\n2 84\n",
        "69\n31\n6\n38\n38\n"
    )]
    fn matches_official_samples(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 端に近い切断と隣接する切断点を扱います。
    /// - Given: 未切断、端に近い切断、隣接する切断を含む入力があります。
    /// - When: バイナリーで各時点の木材の長さを取得します。
    /// - Then: 外側の番兵区間と内部の区間が正しい長さを返します。
    #[test]
    fn handles_outer_and_adjacent_cuts() {
        // Given
        let input = "10 9\n2 5\n1 1\n2 2\n1 9\n2 8\n1 4\n1 5\n2 3\n2 6\n";
        let expected = "10\n9\n8\n3\n4\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 繰り返す切断と問い合わせが、切断点の線形探索と一致します。
    /// - Given: 長さ 50 の木材に対する、有効な 100 回の操作があります。
    /// - When: 切断済みの点を避けて入力を作り、左右の切断点から期待値を求めます。
    /// - Then: バイナリーの全出力が期待値と一致します。
    #[test]
    fn matches_naive_neighbor_search() {
        // Given
        let length = 50;
        let q = 100;
        let mut cuts = vec![0, length];
        let mut input = format!("{length} {q}\n");
        let mut expected = String::new();
        for step in 0..q {
            let x = (0..length - 1)
                .map(|i| (i + step * 13) % (length - 1) + 1)
                .find(|x| !cuts.contains(x))
                .unwrap();
            if step % 3 == 0 && cuts.len() < 22 {
                cuts.push(x);
                input.push_str(&format!("1 {x}\n"));
            } else {
                let left = cuts.iter().copied().filter(|&a| a < x).max().unwrap();
                let right = cuts.iter().copied().filter(|&a| a > x).min().unwrap();
                input.push_str(&format!("2 {x}\n"));
                expected.push_str(&format!("{}\n", right - left));
            }
        }
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        assert_eq!(expected, result);
    }
}

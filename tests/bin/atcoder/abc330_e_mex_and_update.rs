//! ABC330 E のバイナリーを、公式サンプルと素朴な mex 探索で検証します。

use super::super::common;

/// ABC330 E の実行バイナリーです。
const BIN: &str = env!("CARGO_BIN_EXE_ac-abc330-e-mex-and-update");

// 統合テスト: 値の削除による区間分割と、mex の前後への変化を検証します。
mod ac_abc330_e_mex_and_update {
    use super::*;

    /// Scenario: 公式サンプルの各更新後の mex を求めます。
    /// - Given: 重複と大きな値への更新を含む公式サンプルがあります。
    /// - When: バイナリーに入力します。
    /// - Then: 4、3、6、5、0 の順に出力します。
    #[test]
    fn matches_official_sample() {
        // Given
        let input = "8 5\n2 0 2 2 1 1 2 5\n4 3\n4 4\n6 3\n8 1000000000\n2 1\n";
        let expected = "4\n3\n6\n5\n0\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 重複の最後の削除と同じ値への更新を区別します。
    /// - Given: 値の出現回数が 0・1・複数の間で変わる入力があります。
    /// - When: バイナリーで各更新を処理します。
    /// - Then: 最後の出現を削除したときだけ、その値が mex の候補に戻ります。
    #[rstest::rstest]
    #[case::duplicates(
        "4 7\n0 0 1 2\n1 3\n2 3\n1 0\n4 1\n3 1\n2 1000000000\n4 2\n",
        "4\n0\n4\n2\n2\n2\n3\n"
    )]
    #[case::single_value("1 3\n1000000000\n1 1000000000\n1 0\n1 0\n", "0\n1\n1\n")]
    fn handles_duplicates_and_unchanged_values(#[case] input: &str, #[case] expected: &str) {
        // Given
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 更新列の出力が、配列を毎回走査する mex 探索と一致します。
    /// - Given: 重複する初期値と、大きな値を含む 150 回の更新があります。
    /// - When: 各更新後に素朴な配列から期待する mex を計算します。
    /// - Then: バイナリーの全出力が期待値と一致します。
    #[test]
    fn matches_naive_mex_after_repeated_updates() {
        // Given
        let n = 12;
        let q = 150;
        let mut values = (0..n).map(|i| i % 4).collect::<Vec<_>>();
        let mut input = format!("{n} {q}\n");
        for value in &values {
            input.push_str(&format!("{value} "));
        }
        input.push('\n');
        let mut expected = String::new();
        for step in 0..q {
            let index = (step * 7 + 3) % n;
            let value = if step % 11 == 0 {
                1_000_000_000
            } else {
                (step * step + 3 * step) % 19
            };
            input.push_str(&format!("{} {value}\n", index + 1));
            values[index] = value;
            let mex = (0..=n).find(|x| !values.contains(x)).unwrap();
            expected.push_str(&format!("{mex}\n"));
        }
        // When
        let result = common::run_binary(BIN, &input);
        // Then
        assert_eq!(expected, result);
    }
}

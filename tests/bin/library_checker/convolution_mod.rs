use std::io::Write;
use std::process::{Command, Stdio};

// lc-convolution-mod のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_convolution_mod {
    use super::*;

    /// Background: lc-convolution-mod バイナリへ標準入力として `input` を渡し、
    /// 標準出力に書き込まれた内容を文字列として返す。
    fn run_binary(input: &str) -> String {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lc-convolution-mod"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn lc-convolution-mod");

        child
            .stdin
            .take()
            .expect("failed to open child stdin")
            .write_all(input.as_bytes())
            .expect("failed to write to child stdin");

        let output = child
            .wait_with_output()
            .expect("failed to wait for lc-convolution-mod");
        assert!(
            output.status.success(),
            "lc-convolution-mod exited with a failure status"
        );
        String::from_utf8(output.stdout).expect("output was not valid UTF-8")
    }

    /// Scenario: 問題文のサンプル 1 を解いたときの標準出力を検証する
    /// - Given: convolution_mod のサンプル 1 の入力である
    /// - When: lc-convolution-mod バイナリへ標準入力として渡す
    /// - Then: サンプル出力と一致する
    #[test]
    fn sample_1() {
        // Given
        let input = "4 5\n1 2 3 4\n5 6 7 8 9\n";
        // When
        let result = run_binary(input);
        // Then
        assert_eq!("5\n16\n34\n60\n70\n70\n59\n36\n", result);
    }

    /// Scenario: 問題文のサンプル 2 を解いたときの標準出力を検証する
    /// - Given: convolution_mod のサンプル 2 の入力である (MOD に近い値の掛け合わせ)
    /// - When: lc-convolution-mod バイナリへ標準入力として渡す
    /// - Then: サンプル出力と一致する
    #[test]
    fn sample_2() {
        // Given
        let input = "1 1\n10000000\n10000000\n";
        // When
        let result = run_binary(input);
        // Then
        assert_eq!("871938225\n", result);
    }
}

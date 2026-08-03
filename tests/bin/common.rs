//! bin/ 配下の統合テストが共有するヘルパーを提供するモジュールである。

use std::io::Write;
use std::path::Path;
use std::process::{self, Stdio};

/// Background: `bin_path`（呼び出し側で `env!("CARGO_BIN_EXE_xxx")` を渡す）が指す
/// コンパイル済みバイナリへ標準入力として `input` を渡し、標準出力に書き込まれた内容を
/// 文字列として返す。bin/ 以下の統合テストで共有する。
///
/// # Args
/// - `bin_path` - 実行するバイナリのパス。`env!("CARGO_BIN_EXE_xxx")` の戻り値を渡す。
/// - `input` - 標準入力として渡す文字列。
///
/// # Returns
/// バイナリが標準出力に書き込んだ内容を UTF-8 文字列として返す。
///
/// # Panics
/// - バイナリの起動・標準入力への書き込み・終了待機のいずれかに失敗した場合。
/// - バイナリが失敗ステータスで終了した場合。
/// - 標準出力が UTF-8 として不正な場合。
pub fn run_binary(bin_path: &str, input: &str) -> String {
    // フルパスではなくバイナリ名のみをパニックメッセージに使うため、切り出しておく。
    let bin_name = Path::new(bin_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(bin_path);

    let mut child = process::Command::new(bin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("failed to spawn {bin_name}"));

    child
        .stdin
        .take()
        .expect("failed to open child stdin")
        .write_all(input.as_bytes())
        .expect("failed to write to child stdin");

    let output = child
        .wait_with_output()
        .unwrap_or_else(|_| panic!("failed to wait for {bin_name}"));
    assert!(
        output.status.success(),
        "{bin_name} exited with a failure status"
    );
    String::from_utf8(output.stdout).expect("output was not valid UTF-8")
}

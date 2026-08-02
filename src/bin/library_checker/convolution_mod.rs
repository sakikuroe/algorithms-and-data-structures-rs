//! Library Checker: Convolution (mod 998244353)
//! https://judge.yosupo.jp/problem/convolution_mod
//!
//! これはジャッジへ手動で貼り付けて提出するためのファイルである。
//! サンプル入出力による動作確認のみを行い、テストケース全体の自動検証は行わない。

use anmitsu::io::fastio::Fastio;
use anmitsu::modulo998244353::convolution;

// 入力の読み取りから出力の書き込みまでを担う。main と、サンプル入出力を用いたテストの
// 両方から呼び出せるよう、Fastio を介した I/O をこの関数に切り出している。
fn run(io: &mut Fastio) {
    let n = io.u32() as usize;
    let m = io.u32() as usize;

    let mut a = Vec::with_capacity(n);
    for _ in 0..n {
        a.push(io.u32());
    }
    let mut b = Vec::with_capacity(m);
    for _ in 0..m {
        b.push(io.u32());
    }

    let c = convolution::convolution(a, b);

    for c in c {
        io.writeln(c);
    }
}

fn main() {
    let mut io = Fastio::new();
    run(&mut io);
    io.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: 問題文のサンプル 1 を解いたときの出力を検証する
    /// - Given: convolution_mod のサンプル 1 の入力である
    /// - When: run を呼び出す
    /// - Then: サンプル出力と一致する
    #[test]
    fn sample_1() {
        // Given
        let mut io = Fastio::from_bytes("4 5\n1 2 3 4\n5 6 7 8 9\n");
        // When
        run(&mut io);
        // Then
        assert_eq!(
            b"5\n16\n34\n60\n70\n70\n59\n36\n".to_vec(),
            io.into_output()
        );
    }

    /// Scenario: 問題文のサンプル 2 を解いたときの出力を検証する
    /// - Given: convolution_mod のサンプル 2 の入力である (MOD に近い値の掛け合わせ)
    /// - When: run を呼び出す
    /// - Then: サンプル出力と一致する
    #[test]
    fn sample_2() {
        // Given
        let mut io = Fastio::from_bytes("1 1\n10000000\n10000000\n");
        // When
        run(&mut io);
        // Then
        assert_eq!(b"871938225\n".to_vec(), io.into_output());
    }
}

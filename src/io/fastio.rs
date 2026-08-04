//! 競技プログラミング向けの高速入出力ユーティリティを提供するモジュールである。
//!
//! 標準入力を丸ごとバッファへ取り込んだうえで、空白区切りの整数や文字を高速に読み取る。
//! 出力側も同様に、整形結果を内部バッファへ蓄積し、まとめてフラッシュすることで
//! システムコールの回数を抑える。`Fastio::new` は実行時 (ジャッジへの提出時) に用いる
//! コンストラクタであり、標準入出力を直接扱う。一方 `Fastio::from_bytes` は、
//! サンプル入出力によるローカルでの動作確認のためのコンストラクタであり、
//! 標準入出力の代わりにメモリ上のバイト列を読み書きする。

use std::{ffi, fs, io, ptr};

#[cfg(target_os = "linux")]
use std::os::unix;

/// 出力バッファのデフォルトサイズ。
const OUT_BUF_SIZE: usize = 1 << 18;

/// バッファの残りが以下の閾値を下回ったらフラッシュする。
const OUT_BUF_FLUSH_THRESHOLD: usize = 32;

/// 以下の閾値を超えたらフラッシュする。
const OUT_FLUSH_LIMIT: usize = OUT_BUF_SIZE - OUT_BUF_FLUSH_THRESHOLD;

/// 0..=9999 を 4 桁の ASCII 数字列として `u32` (little-endian) に詰めたテーブル。
///
/// 数値を 4 桁ずつまとめて書き込む際に、桁ごとの除算・剰余演算を都度行う代わりに
/// このテーブルを引くことで、出力の高速化を図る。
static DIGIT4_TABLE: [u32; 10_000] = {
    let mut table = [b'0' as u32; 10_000];
    let mut i = 0;
    while i < 10_000 {
        table[i] |= ((i / 1000) % 10) as u32 + b'0' as u32;
        table[i] |= (((i / 100) % 10) as u32 + b'0' as u32) << 8;
        table[i] |= (((i / 10) % 10) as u32 + b'0' as u32) << 16;
        table[i] |= ((i % 10) as u32 + b'0' as u32) << 24;
        i += 1;
    }
    table
};

/// 入力 (`u64`) を ASCII 文字 (8 bit) 8 つ分と見なしたとき、全てが数字
/// (`'0'` (0x30) 〜 `'9'` (0x39)) であるかどうかを判定する。
///
/// # Args
/// - `bytes` - ASCII 文字列の 8 バイト分を little-endian の `u64` として解釈した値
///
/// # Returns
/// 8 バイトすべてが `'0'` 〜 `'9'` の範囲に収まる場合は `true` を、
/// そうでない場合は `false` を返す。
///
/// # Panics
/// パニックしない。
///
/// # Complexity
/// - 時間計算量: $O(1)$
/// - 空間計算量: $O(1)$
///
/// # Examples
/// ```ignore
/// // "12345678" を little-endian の u64 として解釈した値を渡す。
/// assert!(is_8digits(0x3837363534333231));
/// ```
fn is_8digits(mut bytes: u64) -> bool {
    // 各バイトから '0' (0x30) を引き、上位ニブルが 0x30 になるかを見る代わりに、
    // 0x30 との XOR を取ってから数字部分 (下位ニブル) を無視するマスクをかける。
    // 数字であれば上位ニブルが 0 になり、全バイトの上位ニブルが 0 なら bytes は 0 になる。
    bytes ^= 0x3030303030303030;
    bytes &= 0xf0f0f0f0f0f0f0f0;
    bytes == 0
}

/// 入力 (`u64`) を ASCII 文字 (8 bit) 8 つ分と見なし、10 進数としての値 (`u32`) を返す。
///
/// # Args
/// - `bytes` - ASCII 数字列の 8 バイト分を little-endian の `u64` として解釈した値であり、
///   [`is_8digits`] が `true` を返す値である必要がある。
///
/// # Returns
/// 8 桁分を 10 進数として解釈した `u32` 値を返す。
///
/// # Panics
/// デバッグビルドでは、`bytes` が数字 8 桁分でない場合にパニックする。
/// リリースビルドではこの検査は行われず、不正な値を渡すと未定義動作にはならないが、
/// 誤った数値が返る。
///
/// # Complexity
/// - 時間計算量: $O(1)$
/// - 空間計算量: $O(1)$
///
/// # Examples
/// ```ignore
/// // "12345678" を little-endian の u64 として解釈した値を渡す。
/// assert_eq!(12345678, parse_8digits(0x3837363534333231));
/// ```
fn parse_8digits(bytes: u64) -> u32 {
    debug_assert!(is_8digits(bytes));

    // SWAR (SIMD Within A Register) 法により、8 バイトの数字列を 3 段階の
    // 乗算とビットシフトだけで 32 bit 整数へまとめて変換する。
    // 各段階で隣接する 2 桁分の値を 1 つのフィールドへ合成していく。
    let v1 = (bytes & 0x0f0f0f0f0f0f0f0f).wrapping_mul(0xA01) >> 8;
    let v2 = (v1 & 0x00ff00ff00ff00ff).wrapping_mul(0x640001) >> 16;
    let v3 = (v2 & 0x0000ffff0000ffff).wrapping_mul(0x271000000001) >> 32;
    v3 as u32
}

/// `Fastio` の出力バッファへ書き込める値を表す trait。
///
/// 数値や文字など、標準出力へ整形して書き出したい値の型ごとにこの trait を実装する。
pub trait FastWrite {
    /// 値を区切り文字なしで書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む値そのもの
    /// - `io` - 書き込み先の [`Fastio`] への可変参照
    ///
    /// # Returns
    /// 戻り値はない。
    fn write_to(self, io: &mut Fastio);

    /// 値を書き込んだ後、改行文字を書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む値そのもの
    /// - `io` - 書き込み先の [`Fastio`] への可変参照
    ///
    /// # Returns
    /// 戻り値はない。
    fn writeln_to(self, io: &mut Fastio);
}

/// 標準入力を丸ごとバッファし、整形済みの出力もメモリに蓄積して高速にフラッシュするリーダー兼ライター。
///
/// Linux 上では `/dev/stdin` を `mmap` することで、入力を `Vec` へコピーする手間を省く。
/// この経路が使えない場合は、標準入力を読み切って `Vec` として保持する経路にフォールバックする。
pub struct Fastio {
    /// 次に読み取る入力バイトを指すポインタ。
    ///
    /// `_input_storage` が所有するバッファ、または `mmap` されたメモリ領域のいずれかを指す。
    /// 末尾には、8 バイト単位のパースが範囲外アクセスを起こさないための番兵バイト列がある。
    in_cursor: *const u8,

    /// 整形済みの出力を書き込むバッファ。
    out_buf: Vec<u8>,

    /// `out_buf` のうち、書き込み済みの範囲を表す位置。
    out_pos: usize,

    /// 出力の送り先を切り替えるためのバッファ。
    ///
    /// `None` の場合は標準出力へ直接書き込む、提出時と同じ経路を使う。
    /// `Some` の場合は標準出力へは書き込まず、フラッシュされた内容をこのバッファへ蓄積する。
    /// これは [`Fastio::from_bytes`] によるサンプル入出力の検証で用いる。
    out_capture: Option<Vec<u8>>,

    /// 標準入力から読み取ったバイト列の所有者。
    ///
    /// `in_cursor` がこの `Vec` の先頭を指す経路でのみ内容を持ち、`mmap` を使う経路では空のままである。
    /// `in_cursor` が指すメモリの生存期間をこのフィールドの生存期間に結び付けている。
    _input_storage: Vec<u8>,
}

impl Fastio {
    /// 所有権を持つ入力バイト列から `Fastio` を組み立てる。
    ///
    /// # Args
    /// - `input_storage` - 読み取り対象の入力バイト列
    /// - `out_capture` - 出力の送り先。標準出力なら `None`、メモリ上への蓄積なら `Some(Vec::new())` を渡す。
    ///
    /// # Returns
    /// 組み立てられた `Fastio` を返す。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: $O(1)$ (番兵バイトの追加による amortized な再確保を除く)
    /// - 空間計算量: $O(1)$ (`input_storage` の所有権を引き継ぐのみで追加のコピーは行わない)
    fn with_storage(mut input_storage: Vec<u8>, out_capture: Option<Vec<u8>>) -> Self {
        // 8 桁ずつのパースの際に、末尾でメモリ外アクセスを起こさないよう、末尾に十分な番兵を追加しておく。
        // 番兵には 0x00 を用いる。0x00 は数字 ('0'..='9') にならないため is_8digits は確実に false を
        // 返し、かつ ASCII の空白文字 (0x20 以下) の範囲にも収まるため、chars や整数パースの
        // 「空白以外/数字が続く間読み進める」ループも番兵の手前で正しく止まる。この 0x00 は、
        // mmap 経路においてファイル末尾を含む最終ページの端数部分が OS によりゼロ埋めされる挙動とも
        // 一致しており、両経路で番兵の性質を揃えられる。
        input_storage.extend_from_slice(&[0_u8; 8]);
        let in_cursor = input_storage.as_ptr();
        Self {
            in_cursor,
            out_buf: vec![0_u8; OUT_BUF_SIZE],
            out_pos: 0,
            out_capture,
            _input_storage: input_storage,
        }
    }

    /// 標準入力を読み取る `Fastio` を作成する。
    ///
    /// ジャッジへ提出する実行時のコードから呼び出すことを想定したコンストラクタである。
    /// Linux 上で `/dev/stdin` が通常ファイルとして開ける場合は、これを `mmap` することで
    /// 入力全体をヒープへコピーする手間を省く。それ以外の場合は、標準入力を最後まで読み切って
    /// `Vec` として保持する経路にフォールバックする。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// 標準入力に接続され、標準出力へ書き込む `Fastio` を返す。
    ///
    /// # Panics
    /// `mmap` が利用できず標準入力からの読み込みにフォールバックした場合に、
    /// その読み込み自体が失敗するとパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: $O(1)$ (`mmap` 経路の場合)、または $O(L)$ (フォールバック経路の場合。$L$ は入力全体のバイト数)
    /// - 空間計算量: $O(1)$ (`mmap` 経路の場合)、または $O(L)$ (フォールバック経路の場合)
    ///
    /// # Examples
    /// ```rust,no_run
    /// let mut io = anmitsu::io::fastio::Fastio::new();
    /// let n = io.u32();
    /// io.writeln(n);
    /// io.flush();
    /// ```
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        unsafe {
            const PROT_READ: i32 = 0x1;
            const MAP_PRIVATE: i32 = 0x02;
            const MAP_FAILED: *mut ffi::c_void = (-1isize) as *mut ffi::c_void;

            // /dev/stdin を通常ファイルとして開けない、あるいは mmap に失敗した場合は、
            // 'mmap_try ブロックを抜けてフォールバック経路へ進む。
            'mmap_try: {
                let Ok(file) = fs::File::open("/dev/stdin") else {
                    break 'mmap_try;
                };
                let Ok(metadata) = file.metadata() else {
                    break 'mmap_try;
                };
                if !metadata.is_file() {
                    break 'mmap_try;
                }

                let len = metadata.len() as usize;
                let fd = unix::io::AsRawFd::as_raw_fd(&file);
                let addr = ptr::null_mut();
                let mapped = mmap(addr, len, PROT_READ, MAP_PRIVATE, fd, 0);
                if mapped == MAP_FAILED {
                    break 'mmap_try;
                }

                // mmap されたメモリ領域を直接読み取り対象とすることで、
                // 標準入力全体をヒープへコピーするコストを避ける。
                let in_cursor = mapped as *const u8;
                return Self {
                    in_cursor,
                    out_buf: vec![0_u8; OUT_BUF_SIZE],
                    out_pos: 0,
                    out_capture: None,
                    _input_storage: vec![],
                };
            }
        }

        // mmap が使えない環境向けのフォールバック経路。標準入力を最後まで読み切って保持する。
        let mut input_storage = vec![];
        io::Read::read_to_end(&mut io::stdin().lock(), &mut input_storage)
            .expect("failed to read from source");
        Self::with_storage(input_storage, None)
    }

    /// メモリ上のバイト列を入力として読み取る `Fastio` を作成する。
    ///
    /// ジャッジの問題文に記載されたサンプル入出力を用いてローカルで動作確認するための
    /// コンストラクタであり、実際の提出コードでは [`Fastio::new`] を使う。出力は標準出力へは
    /// 書き込まれず、[`Fastio::into_output`] で取り出せるようメモリ上に蓄積される。
    ///
    /// # Args
    /// - `input` - 読み取り対象の入力バイト列 (問題のサンプル入力など)
    ///
    /// # Returns
    /// 与えたバイト列を入力とし、出力をメモリへ蓄積する `Fastio` を返す。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: $O(L)$
    ///   - $L$ は `input` のバイト数である。
    /// - 空間計算量: $O(L)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("1 2\n");
    /// assert_eq!(1_u32, io.u32());
    /// assert_eq!(2_u32, io.u32());
    /// ```
    pub fn from_bytes(input: impl Into<Vec<u8>>) -> Self {
        Self::with_storage(input.into(), Some(Vec::new()))
    }

    /// 空白 (改行を含む) を読み飛ばした後、1 文字を読み取る。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った 1 文字を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が尽きた状態で呼び出した場合は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(k)$
    ///   - $k$ は読み飛ばす空白の個数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("a b");
    /// assert_eq!('a', io.char());
    /// assert_eq!('b', io.char());
    /// ```
    pub fn char(&mut self) -> char {
        unsafe {
            self.skip_whitespace();
            let ch = *self.in_cursor as char;
            self.in_cursor = self.in_cursor.add(1);
            ch
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、符号付き 32 bit 整数を読み取る。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った `i32` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が `i32` の範囲を超える、または数字列として不正な場合は
    /// 未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("-42 7");
    /// assert_eq!(-42, io.i32());
    /// assert_eq!(7, io.i32());
    /// ```
    pub fn i32(&mut self) -> i32 {
        unsafe {
            self.skip_whitespace();
            let negative = *self.in_cursor == b'-';
            if negative {
                self.in_cursor = self.in_cursor.add(1);
            }

            // 8 桁ずつまとめて読み取れる間は SWAR 法でまとめて処理し、除算・剰余の回数を減らす。
            let mut value = 0_i32;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                let parsed = parse_8digits(bytes) as i32;
                if negative {
                    value = value * 100_000_000 - parsed;
                } else {
                    value = value * 100_000_000 + parsed;
                }
                self.in_cursor = self.in_cursor.add(8);
            }

            // 残った端数の桁は 1 桁ずつ処理する。
            while *self.in_cursor >= b'0' {
                let digit = (*self.in_cursor - b'0') as i32;
                if negative {
                    value = 10 * value - digit;
                } else {
                    value = 10 * value + digit;
                }
                self.in_cursor = self.in_cursor.add(1);
            }

            value
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、符号なし 64 bit 整数を読み取る。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った `u64` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が `u64` の範囲を超える、または数字列として不正な場合は
    /// 未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("18446744073709551615");
    /// assert_eq!(u64::MAX, io.u64());
    /// ```
    pub fn u64(&mut self) -> u64 {
        unsafe {
            self.skip_whitespace();
            let mut value = 0_u64;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                value = value * 100_000_000_u64 + parse_8digits(bytes) as u64;
                self.in_cursor = self.in_cursor.add(8);
            }

            while *self.in_cursor >= b'0' {
                value = 10 * value + (*self.in_cursor - b'0') as u64;
                self.in_cursor = self.in_cursor.add(1);
            }
            value
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、符号なし 32 bit 整数を読み取る。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った `u32` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が `u32` の範囲を超える、または数字列として不正な場合は
    /// 未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("4 5");
    /// assert_eq!(4_u32, io.u32());
    /// assert_eq!(5_u32, io.u32());
    /// ```
    pub fn u32(&mut self) -> u32 {
        unsafe {
            self.skip_whitespace();
            let mut value = 0_u32;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                value = value * 100_000_000 + parse_8digits(bytes);
                self.in_cursor = self.in_cursor.add(8);
            }

            while *self.in_cursor >= b'0' {
                value = 10 * value + (*self.in_cursor - b'0') as u32;
                self.in_cursor = self.in_cursor.add(1);
            }
            value
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、符号付き 64 bit 整数を読み取る。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った `i64` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が `i64` の範囲を超える、または数字列として不正な場合は
    /// 未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("-9223372036854775808");
    /// assert_eq!(i64::MIN, io.i64());
    /// ```
    pub fn i64(&mut self) -> i64 {
        unsafe {
            self.skip_whitespace();
            let negative = *self.in_cursor == b'-';
            if negative {
                self.in_cursor = self.in_cursor.add(1);
            }

            let mut value = 0_i64;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                let parsed = parse_8digits(bytes) as i64;
                if negative {
                    value = value * 100_000_000 - parsed;
                } else {
                    value = value * 100_000_000 + parsed;
                }
                self.in_cursor = self.in_cursor.add(8);
            }

            while *self.in_cursor >= b'0' {
                let digit = (*self.in_cursor - b'0') as i64;
                if negative {
                    value = 10 * value - digit;
                } else {
                    value = 10 * value + digit;
                }
                self.in_cursor = self.in_cursor.add(1);
            }

            value
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、1-indexed の符号なし整数を読み取り、
    /// 0-indexed の `isize` に変換して返す。
    ///
    /// グラフの頂点番号や配列の添字など、問題文が 1-indexed で与える値を
    /// そのまま 0-indexed の添字として使いたい場面のために用意している。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った値から 1 を引いた `isize` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が不正な場合、または読み取った値が 0 の場合
    /// (0 - 1 のアンダーフローが発生する) は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("1 3");
    /// assert_eq!(0, io.isize1());
    /// assert_eq!(2, io.isize1());
    /// ```
    pub fn isize1(&mut self) -> isize {
        unsafe {
            self.skip_whitespace();
            let mut value = 0_isize;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                value = value * 100_000_000 + parse_8digits(bytes) as isize;
                self.in_cursor = self.in_cursor.add(8);
            }

            while *self.in_cursor >= b'0' {
                value = 10 * value + (*self.in_cursor - b'0') as isize;
                self.in_cursor = self.in_cursor.add(1);
            }

            value - 1
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、1-indexed の符号なし整数を読み取り、
    /// 0-indexed の `usize` に変換して返す。
    ///
    /// [`Fastio::isize1`] の `usize` 版であり、用途は同様である。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った値から 1 を引いた `usize` の値を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が不正な場合、または読み取った値が 0 の場合
    /// (0 - 1 のアンダーフローが発生する) は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(d)$
    ///   - $d$ は読み取る数字の桁数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("1 3");
    /// assert_eq!(0_usize, io.usize1());
    /// assert_eq!(2_usize, io.usize1());
    /// ```
    pub fn usize1(&mut self) -> usize {
        unsafe {
            self.skip_whitespace();
            let mut value = 0_usize;
            loop {
                let bytes = ptr::read_unaligned(self.in_cursor as *const u64);
                if !is_8digits(bytes) {
                    break;
                }
                value = value * 100_000_000 + parse_8digits(bytes) as usize;
                self.in_cursor = self.in_cursor.add(8);
            }

            while *self.in_cursor >= b'0' {
                value = 10 * value + (*self.in_cursor - b'0') as usize;
                self.in_cursor = self.in_cursor.add(1);
            }

            value - 1
        }
    }

    /// 空白 (改行を含む) を読み飛ばした後、続く空白以外の文字を読み取り、`Vec<char>` として返す。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 読み取った文字の列を格納した `Vec<char>` を返す。
    ///
    /// # Panics
    /// パニックしない。ただし、入力が尽きた状態で呼び出した場合は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(m)$
    ///   - $m$ は読み取る文字数である。
    /// - 空間計算量: $O(m)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("abc");
    /// assert_eq!(vec!['a', 'b', 'c'], io.chars());
    /// ```
    pub fn chars(&mut self) -> Vec<char> {
        unsafe {
            self.skip_whitespace();
            let mut res = vec![];
            while *self.in_cursor > b' ' {
                res.push(*self.in_cursor as char);
                self.in_cursor = self.in_cursor.add(1);
            }
            res
        }
    }

    /// 出力バッファに溜めた内容を出力先へ書き込む。
    ///
    /// [`Fastio::new`] で作成した場合は標準出力へ、[`Fastio::from_bytes`] で作成した場合は
    /// メモリ上のキャプチャ用バッファへ書き込む。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// 標準出力への書き込みが失敗した場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: $O(N)$
    ///   - $N$ はフラッシュ対象のバイト数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("");
    /// io.writeln(1_u32);
    /// io.flush();
    /// assert_eq!(b"1\n", io.into_output().as_slice());
    /// ```
    pub fn flush(&mut self) {
        self.out_flush();
    }

    /// フラッシュした内容の蓄積を終え、キャプチャした出力を取り出す。
    ///
    /// [`Fastio::from_bytes`] で作成した `Fastio` に対して、書き込んだ内容を検証する目的で使う。
    /// [`Fastio::new`] で作成した `Fastio` に対して呼び出した場合、標準出力へ既に書き込まれた
    /// 内容はキャプチャされていないため、空の `Vec` を返す。
    ///
    /// # Args
    /// `self` 以外の引数はない。
    ///
    /// # Returns
    /// キャプチャした出力バイト列を返す。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: $O(N)$
    ///   - $N$ は出力済みバイト数 (未フラッシュ分のフラッシュ処理を含む) である。
    /// - 空間計算量: $O(1)$ (所有権の移動のみ)
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("");
    /// io.writeln(42_u32);
    /// assert_eq!(b"42\n", io.into_output().as_slice());
    /// ```
    pub fn into_output(mut self) -> Vec<u8> {
        self.out_flush();
        self.out_capture.take().unwrap_or_default()
    }

    /// 出力バッファに溜めた内容を、実際に出力先 (標準出力またはキャプチャ用バッファ) へ書き込む。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// 標準出力への書き込みが失敗した場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: $O(N)$
    ///   - $N$ はフラッシュ対象のバイト数である。
    /// - 空間計算量: $O(1)$
    #[inline(always)]
    fn out_flush(&mut self) {
        self.try_out_flush()
            .expect("failed to write buffered output");
    }

    /// 出力バッファに溜めた内容を出力先へ書き込む。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 書き込みが成功した場合は `Ok(())` を、失敗した場合は `Err` を返す。
    ///
    /// # Panics
    /// パニックしない。
    ///
    /// # Complexity
    /// - 時間計算量: $O(N)$
    ///   - $N$ はフラッシュ対象のバイト数である。
    /// - 空間計算量: $O(1)$
    fn try_out_flush(&mut self) -> io::Result<()> {
        if self.out_pos == 0 {
            return Ok(());
        }

        // キャプチャモードの場合は、標準出力ではなくメモリ上のバッファへ書き写す。
        if let Some(capture) = &mut self.out_capture {
            capture.extend_from_slice(&self.out_buf[..self.out_pos]);
            self.out_pos = 0;
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        unsafe {
            // write(2) は一度の呼び出しで全バイトを書き込むとは限らないため、
            // 書き込み済みバイト数が対象範囲に達するまでループする。
            let mut written = 0_usize;
            let base = self.out_buf.as_ptr();
            while written < self.out_pos {
                let n = write(
                    1,
                    base.add(written) as *const ffi::c_void,
                    self.out_pos - written,
                );
                if n < 0 {
                    return Err(io::Error::last_os_error());
                }
                written += n as usize;
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let mut stdout = io::stdout().lock();
            io::Write::write_all(&mut stdout, &self.out_buf[..self.out_pos])?;
            io::Write::flush(&mut stdout)?;
        }

        self.out_pos = 0;
        Ok(())
    }

    /// 出力バッファの残りが少なくなっている場合にフラッシュする。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// 標準出力への書き込みが失敗した場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 $O(1)$。フラッシュ自体は $O(N)$ だが、バッファが一杯になるたびに
    ///   1 回しか発生しないため、書き込み 1 回あたりでは償却して $O(1)$ とみなせる。
    /// - 空間計算量: $O(1)$
    #[inline(always)]
    fn out_maybe_flush(&mut self) {
        if self.out_pos > OUT_FLUSH_LIMIT {
            self.out_flush();
        }
    }

    /// 0..=9999 の値を、ちょうど 4 桁の ASCII 数字列として出力バッファへ書き込む。
    ///
    /// 4 桁に満たない値であっても、先頭に `'0'` を補って 4 桁として書き込む。
    ///
    /// # Args
    /// - `x` - 書き込む値であり、0..=9999 の範囲でなければならない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// パニックしない。ただし、出力バッファの残り容量が 4 バイト未満の場合は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(1)$
    /// - 空間計算量: $O(1)$
    #[inline(always)]
    fn out_put_4digits(&mut self, x: u16) {
        unsafe {
            let dst = self.out_buf.as_mut_ptr().add(self.out_pos);
            ptr::write_unaligned(dst as *mut u32, DIGIT4_TABLE[x as usize]);
        }
        self.out_pos += 4;
    }

    /// 0..=9999 の値を、先頭の `'0'` を省いた ASCII 数字列として出力バッファへ書き込む。
    ///
    /// 数値の最上位 4 桁を書き込む際に、無駄な先頭の `'0'` を出力しないために用いる。
    ///
    /// # Args
    /// - `x` - 書き込む値であり、0..=9999 の範囲でなければならない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// パニックしない。ただし、出力バッファの残り容量が 4 バイト未満の場合は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(1)$
    /// - 空間計算量: $O(1)$
    #[inline(always)]
    fn out_put_upto_4digits(&mut self, x: u16) {
        unsafe {
            let dst = self.out_buf.as_mut_ptr().add(self.out_pos);
            if x <= 9 {
                ptr::write_unaligned(dst as *mut u32, DIGIT4_TABLE[(x * 1000) as usize]);
                self.out_pos += 1;
            } else if x <= 99 {
                ptr::write_unaligned(dst as *mut u32, DIGIT4_TABLE[(x * 100) as usize]);
                self.out_pos += 2;
            } else if x <= 999 {
                ptr::write_unaligned(dst as *mut u32, DIGIT4_TABLE[(x * 10) as usize]);
                self.out_pos += 3;
            } else {
                ptr::write_unaligned(dst as *mut u32, DIGIT4_TABLE[x as usize]);
                self.out_pos += 4;
            }
        }
    }

    /// 値を区切り文字なしで出力バッファへ書き込む。
    ///
    /// # Args
    /// - `&mut self` - 書き込み先の `Fastio`
    /// - `value` - 書き込む値。[`FastWrite`] を実装した型であれば渡せる。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// 内部でのフラッシュ処理が失敗した場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 $O(d)$
    ///   - $d$ は `value` を書き込むために必要なバイト数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("");
    /// io.write('a');
    /// io.write('b');
    /// assert_eq!(b"ab", io.into_output().as_slice());
    /// ```
    pub fn write<T>(&mut self, value: T)
    where
        T: FastWrite,
    {
        value.write_to(self);
    }

    /// 値を書き込んだ後、改行文字を出力バッファへ書き込む。
    ///
    /// # Args
    /// - `&mut self` - 書き込み先の `Fastio`
    /// - `value` - 書き込む値。[`FastWrite`] を実装した型であれば渡せる。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// 内部でのフラッシュ処理が失敗した場合にパニックする。
    ///
    /// # Complexity
    /// - 時間計算量: 償却 $O(d)$
    ///   - $d$ は `value` を書き込むために必要なバイト数である。
    /// - 空間計算量: $O(1)$
    ///
    /// # Examples
    /// ```rust
    /// let mut io = anmitsu::io::fastio::Fastio::from_bytes("");
    /// io.writeln(1_u32);
    /// io.writeln(2_u32);
    /// assert_eq!(b"1\n2\n", io.into_output().as_slice());
    /// ```
    pub fn writeln<T>(&mut self, value: T)
    where
        T: FastWrite,
    {
        value.writeln_to(self);
    }

    /// 現在の読み取り位置から、空白文字 (ASCII 0x20 以下のバイト全般) を読み飛ばす。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 戻り値はない。
    ///
    /// # Panics
    /// パニックしない。ただし、入力の残りがすべて空白の場合は未定義動作となる。
    ///
    /// # Complexity
    /// - 時間計算量: $O(k)$
    ///   - $k$ は読み飛ばす空白の個数である。
    /// - 空間計算量: $O(1)$
    #[inline(always)]
    unsafe fn skip_whitespace(&mut self) {
        unsafe {
            while *self.in_cursor <= b' ' {
                self.in_cursor = self.in_cursor.add(1);
            }
        }
    }
}

impl Default for Fastio {
    /// [`Fastio::new`] と同じ、標準入出力に接続された `Fastio` を作成する。
    ///
    /// # Args
    /// 引数はない。
    ///
    /// # Returns
    /// [`Fastio::new`] の戻り値と同じ `Fastio` を返す。
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Fastio {
    /// `Fastio` の破棄時に、出力バッファに残っている内容を可能な限りフラッシュする。
    ///
    /// # Args
    /// `&mut self` 以外の引数はない。
    ///
    /// # Returns
    /// 戻り値はない。
    fn drop(&mut self) {
        // プロセス終了時の取りこぼしを防ぐため、可能な限り残りを出力する。
        // Drop からはエラーを返せないため、失敗は握りつぶす。
        let _ = self.try_out_flush();
    }
}

impl FastWrite for char {
    /// `self` を 1 バイトの ASCII 文字として出力バッファへ書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む文字。ASCII 文字であることを前提とする。
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    fn write_to(self, io: &mut Fastio) {
        io.out_buf[io.out_pos] = self as u8;
        io.out_pos += 1;
        io.out_maybe_flush();
    }

    /// `self` を書き込んだ後、改行文字を書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む文字。ASCII 文字であることを前提とする。
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    fn writeln_to(self, io: &mut Fastio) {
        io.out_buf[io.out_pos] = self as u8;
        io.out_pos += 1;
        io.out_buf[io.out_pos] = b'\n';
        io.out_pos += 1;
        io.out_maybe_flush();
    }
}

impl FastWrite for u64 {
    /// `self` を 10 進数の ASCII 数字列として出力バッファへ書き込み、続けて改行文字を書き込む。
    ///
    /// 数値のみを単独で出力する用途がほとんどであるため、`write_to` と `writeln_to` の
    /// 挙動をあえて区別せず、常に改行文字まで書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn write_to(self, io: &mut Fastio) {
        // 上位桁から 4 桁ずつ区切り、DIGIT4_TABLE を用いてまとめて書き込む。
        // 最上位の 4 桁分のみ先頭の '0' を省いて書き込み、残りは 4 桁固定で書き込む。
        if self >= 10_000_000_000_000_000 {
            io.out_put_upto_4digits((self / 10_000_000_000_000_000) as u16);
            io.out_put_4digits(((self / 1_000_000_000_000) as u32 % 10_000) as u16);
            io.out_put_4digits(((self / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((self / 10_000) % 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else if self >= 1_000_000_000_000 {
            io.out_put_upto_4digits(((self / 1_000_000_000_000) as u32 % 10_000) as u16);
            io.out_put_4digits(((self / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((self / 10_000) % 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else if self >= 100_000_000 {
            io.out_put_upto_4digits(((self / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((self / 10_000) % 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else if self >= 10_000 {
            io.out_put_upto_4digits(((self / 10_000) % 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else {
            io.out_put_upto_4digits((self % 10_000) as u16);
        }
        io.out_buf[io.out_pos] = b'\n';
        io.out_pos += 1;
        io.out_maybe_flush();
    }

    /// [`FastWrite::write_to`] と同じ処理を行う。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn writeln_to(self, io: &mut Fastio) {
        self.write_to(io);
    }
}

impl FastWrite for u32 {
    /// `self` を 10 進数の ASCII 数字列として出力バッファへ書き込み、続けて改行文字を書き込む。
    ///
    /// [`FastWrite for u64`](FastWrite) と同様、`write_to` と `writeln_to` の挙動は区別しない。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn write_to(self, io: &mut Fastio) {
        if self >= 100_000_000 {
            io.out_put_upto_4digits((self / 100_000_000) as u16);
            io.out_put_4digits(((self / 10_000) % 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else if self >= 10_000 {
            io.out_put_upto_4digits((self / 10_000) as u16);
            io.out_put_4digits((self % 10_000) as u16);
        } else {
            io.out_put_upto_4digits(self as u16);
        }
        io.out_buf[io.out_pos] = b'\n';
        io.out_pos += 1;
        io.out_maybe_flush();
    }

    /// [`FastWrite::write_to`] と同じ処理を行う。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn writeln_to(self, io: &mut Fastio) {
        self.write_to(io);
    }
}

impl FastWrite for i64 {
    /// `self` を符号付き 10 進数の ASCII 数字列として出力バッファへ書き込み、続けて改行文字を書き込む。
    ///
    /// `i64::MIN` は絶対値が `i64` の範囲に収まらないため、専用の分岐で定数文字列として書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn write_to(self, io: &mut Fastio) {
        if self < 0 {
            if self == i64::MIN {
                // -self が i64 の範囲を超えてオーバーフローするため、
                // i64::MIN の 10 進表記をそのまま定数として書き込む。
                io.out_buf[io.out_pos..io.out_pos + 20].copy_from_slice(b"-9223372036854775808");
                io.out_pos += 20;
                io.out_buf[io.out_pos] = b'\n';
                io.out_pos += 1;
                io.out_maybe_flush();
                return;
            }
            io.out_buf[io.out_pos] = b'-';
            io.out_pos += 1;
        }
        let x = if self < 0 { -self } else { self };
        if x >= 10_000_000_000_000_000 {
            io.out_put_upto_4digits((x / 10_000_000_000_000_000) as u16);
            io.out_put_4digits(((x / 1_000_000_000_000) % 10_000) as u16);
            io.out_put_4digits(((x / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((x / 10_000) % 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else if x >= 1_000_000_000_000 {
            io.out_put_upto_4digits(((x / 1_000_000_000_000) % 10_000) as u16);
            io.out_put_4digits(((x / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((x / 10_000) % 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else if x >= 100_000_000 {
            io.out_put_upto_4digits(((x / 100_000_000) % 10_000) as u16);
            io.out_put_4digits(((x / 10_000) % 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else if x >= 10_000 {
            io.out_put_upto_4digits(((x / 10_000) % 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else {
            io.out_put_upto_4digits((x % 10_000) as u16);
        }
        io.out_buf[io.out_pos] = b'\n';
        io.out_pos += 1;
        io.out_maybe_flush();
    }

    /// [`FastWrite::write_to`] と同じ処理を行う。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn writeln_to(self, io: &mut Fastio) {
        self.write_to(io);
    }
}

impl FastWrite for i32 {
    /// `self` を符号付き 10 進数の ASCII 数字列として出力バッファへ書き込み、続けて改行文字を書き込む。
    ///
    /// `i32::MIN` は絶対値が `i32` の範囲に収まらないため、専用の分岐で定数文字列として書き込む。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn write_to(self, io: &mut Fastio) {
        if self < 0 {
            if self == i32::MIN {
                // -self が i32 の範囲を超えてオーバーフローするため、
                // i32::MIN の 10 進表記をそのまま定数として書き込む。
                io.out_buf[io.out_pos..io.out_pos + 11].copy_from_slice(b"-2147483648");
                io.out_pos += 11;
                io.out_buf[io.out_pos] = b'\n';
                io.out_pos += 1;
                io.out_maybe_flush();
                return;
            }
            io.out_buf[io.out_pos] = b'-';
            io.out_pos += 1;
        }
        let x = if self < 0 { -self } else { self } as u32;
        if x >= 100_000_000 {
            io.out_put_upto_4digits((x / 100_000_000) as u16);
            io.out_put_4digits(((x / 10_000) % 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else if x >= 10_000 {
            io.out_put_upto_4digits((x / 10_000) as u16);
            io.out_put_4digits((x % 10_000) as u16);
        } else {
            io.out_put_upto_4digits(x as u16);
        }
        io.out_buf[io.out_pos] = b'\n';
        io.out_pos += 1;
        io.out_maybe_flush();
    }

    /// [`FastWrite::write_to`] と同じ処理を行う。
    ///
    /// # Args
    /// - `self` - 書き込む値
    /// - `io` - 書き込み先の [`Fastio`]
    ///
    /// # Returns
    /// 戻り値はない。
    #[inline(always)]
    fn writeln_to(self, io: &mut Fastio) {
        self.write_to(io);
    }
}

#[cfg(target_os = "linux")]
#[link(name = "c")]
unsafe extern "C" {
    /// POSIX の `mmap(2)` を呼び出す。標準入力全体をコピーせずに読み取るために使う。
    fn mmap(
        addr: *mut ffi::c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut ffi::c_void;

    /// POSIX の `write(2)` を呼び出す。出力バッファの内容を直接ファイルディスクリプタへ書き込むために使う。
    fn write(fd: i32, buf: *const ffi::c_void, count: usize) -> isize;
}

#[cfg(test)]
mod tests {
    use super::*;

    // is_8digits のテスト: 戻り値そのものを検証する
    mod is_8digits {
        use super::*;

        /// Scenario: 8 バイトすべてが数字であれば true を返す
        /// - Given: "12345678" を little-endian の u64 として解釈した値がある
        /// - When: is_8digits を呼び出す
        /// - Then: true が返る
        #[test]
        fn returns_true_for_all_digit_bytes() {
            // Given
            let sut = 0x3837363534333231_u64;
            // When
            let result = is_8digits(sut);
            // Then
            assert!(result);
        }

        /// Scenario: 数字以外のバイトが混ざっていれば false を返す
        /// - Given: 末尾が空白文字 (0x20) になっている 8 バイトの値がある
        /// - When: is_8digits を呼び出す
        /// - Then: false が返る
        #[test]
        fn returns_false_when_a_byte_is_not_a_digit() {
            // Given
            let sut = 0x3837363534333220_u64;
            // When
            let result = is_8digits(sut);
            // Then
            assert!(!result);
        }
    }

    // parse_8digits のテスト: 戻り値そのものを検証する
    mod parse_8digits {
        use super::*;

        /// Scenario: 8 桁の数字列を 10 進数の値に変換する
        /// - Given: "12345678" を little-endian の u64 として解釈した値がある
        /// - When: parse_8digits を呼び出す
        /// - Then: 12345678 が返る
        #[test]
        fn parses_eight_digit_bytes() {
            // Given
            let sut = 0x3837363534333231_u64;
            // When
            let result = parse_8digits(sut);
            // Then
            assert_eq!(12_345_678, result);
        }

        /// Scenario: 先頭に 0 を含む 8 桁の数字列も正しく変換する
        /// - Given: "00000001" を little-endian の u64 として解釈した値がある
        /// - When: parse_8digits を呼び出す
        /// - Then: 1 が返る
        #[test]
        fn parses_leading_zeros() {
            // Given
            let sut = 0x3130303030303030_u64;
            // When
            let result = parse_8digits(sut);
            // Then
            assert_eq!(1, result);
        }
    }

    // 各種読み取りメソッドのテスト: from_bytes で与えた入力に対する戻り値を検証する
    mod read {
        use super::*;

        /// Scenario: 空白区切りの符号なし 32 bit 整数を順番に読み取れる
        /// - Given: 空白と改行で区切られた 3 つの数値からなる入力がある
        /// - When: u32 を 3 回呼び出す
        /// - Then: 入力に現れた順番通りの値が返る
        #[test]
        fn u32_reads_values_in_order() {
            // Given
            let mut sut = Fastio::from_bytes("4 5\n100000000");
            // When
            let a = sut.u32();
            let b = sut.u32();
            let c = sut.u32();
            // Then
            assert_eq!(4, a);
            assert_eq!(5, b);
            assert_eq!(100_000_000, c);
        }

        /// Scenario: u32 の最大値も正しく読み取れる
        /// - Given: u32::MAX を 10 進数表記した入力がある
        /// - When: u32 を呼び出す
        /// - Then: u32::MAX が返る
        #[test]
        fn u32_reads_max_value() {
            // Given
            let mut sut = Fastio::from_bytes(u32::MAX.to_string());
            // When
            let result = sut.u32();
            // Then
            assert_eq!(u32::MAX, result);
        }

        /// Scenario: 負の符号付き整数を読み取れる
        /// - Given: 負の数と正の数が空白区切りで並ぶ入力がある
        /// - When: i32 を 2 回呼び出す
        /// - Then: 符号を保ったまま値が返る
        #[test]
        fn i32_reads_negative_and_positive_values() {
            // Given
            let mut sut = Fastio::from_bytes("-42 7");
            // When
            let a = sut.i32();
            let b = sut.i32();
            // Then
            assert_eq!(-42, a);
            assert_eq!(7, b);
        }

        /// Scenario: i32 の最小値・最大値も正しく読み取れる
        /// - Given: i32::MIN と i32::MAX を空白区切りで並べた入力がある
        /// - When: i32 を 2 回呼び出す
        /// - Then: それぞれの境界値が返る
        #[test]
        fn i32_reads_boundary_values() {
            // Given
            let mut sut = Fastio::from_bytes(format!("{} {}", i32::MIN, i32::MAX));
            // When
            let min = sut.i32();
            let max = sut.i32();
            // Then
            assert_eq!(i32::MIN, min);
            assert_eq!(i32::MAX, max);
        }

        /// Scenario: u64 の最大値も正しく読み取れる
        /// - Given: u64::MAX を 10 進数表記した入力がある
        /// - When: u64 を呼び出す
        /// - Then: u64::MAX が返る
        #[test]
        fn u64_reads_max_value() {
            // Given
            let mut sut = Fastio::from_bytes(u64::MAX.to_string());
            // When
            let result = sut.u64();
            // Then
            assert_eq!(u64::MAX, result);
        }

        /// Scenario: i64 の最小値・最大値も正しく読み取れる
        /// - Given: i64::MIN と i64::MAX を空白区切りで並べた入力がある
        /// - When: i64 を 2 回呼び出す
        /// - Then: それぞれの境界値が返る
        #[test]
        fn i64_reads_boundary_values() {
            // Given
            let mut sut = Fastio::from_bytes(format!("{} {}", i64::MIN, i64::MAX));
            // When
            let min = sut.i64();
            let max = sut.i64();
            // Then
            assert_eq!(i64::MIN, min);
            assert_eq!(i64::MAX, max);
        }

        /// Scenario: 1-indexed の入力を 0-indexed の isize として読み取れる
        /// - Given: "1" と "3" が空白区切りで並ぶ入力がある
        /// - When: isize1 を 2 回呼び出す
        /// - Then: それぞれ 1 引かれた値が返る
        #[test]
        fn isize1_shifts_to_zero_indexed() {
            // Given
            let mut sut = Fastio::from_bytes("1 3");
            // When
            let a = sut.isize1();
            let b = sut.isize1();
            // Then
            assert_eq!(0, a);
            assert_eq!(2, b);
        }

        /// Scenario: 1-indexed の入力を 0-indexed の usize として読み取れる
        /// - Given: "1" と "3" が空白区切りで並ぶ入力がある
        /// - When: usize1 を 2 回呼び出す
        /// - Then: それぞれ 1 引かれた値が返る
        #[test]
        fn usize1_shifts_to_zero_indexed() {
            // Given
            let mut sut = Fastio::from_bytes("1 3");
            // When
            let a = sut.usize1();
            let b = sut.usize1();
            // Then
            assert_eq!(0, a);
            assert_eq!(2, b);
        }

        /// Scenario: 空白区切りの 1 文字を順番に読み取れる
        /// - Given: 改行と空白で区切られた 2 文字の入力がある
        /// - When: char を 2 回呼び出す
        /// - Then: 入力に現れた順番通りの文字が返る
        #[test]
        fn char_reads_values_in_order() {
            // Given
            let mut sut = Fastio::from_bytes("a\nb");
            // When
            let a = sut.char();
            let b = sut.char();
            // Then
            assert_eq!('a', a);
            assert_eq!('b', b);
        }

        /// Scenario: 空白区切りの文字列をまとめて読み取れる
        /// - Given: 2 つの単語が空白で区切られた入力がある
        /// - When: chars を 2 回呼び出す
        /// - Then: それぞれの単語が文字の列として返る
        #[test]
        fn chars_reads_whitespace_separated_tokens() {
            // Given
            let mut sut = Fastio::from_bytes("abc def");
            // When
            let a = sut.chars();
            let b = sut.chars();
            // Then
            assert_eq!(vec!['a', 'b', 'c'], a);
            assert_eq!(vec!['d', 'e', 'f'], b);
        }
    }

    // write / writeln / into_output のテスト: キャプチャバッファへの書き込み結果を検証する
    mod write {
        use super::*;

        /// Scenario: write は区切り文字を挟まずに値を書き込む
        /// - Given: 空の入力から作った Fastio がある
        /// - When: write で文字を 2 回書き込む
        /// - Then: 2 文字が連結された出力になる
        #[test]
        fn write_concatenates_without_separator() {
            // Given
            let mut sut = Fastio::from_bytes("");
            // When
            sut.write('a');
            sut.write('b');
            // Then
            assert_eq!(b"ab".to_vec(), sut.into_output());
        }

        /// Scenario: writeln は値ごとに改行を挟んで書き込む
        /// - Given: 空の入力から作った Fastio がある
        /// - When: writeln で u32 を 2 回書き込む
        /// - Then: 改行区切りの出力になる
        #[test]
        fn writeln_appends_newline_per_value() {
            // Given
            let mut sut = Fastio::from_bytes("");
            // When
            sut.writeln(1_u32);
            sut.writeln(2_u32);
            // Then
            assert_eq!(b"1\n2\n".to_vec(), sut.into_output());
        }

        /// Scenario: i64::MIN、i32::MIN は特別扱いされても正しい文字列になる
        /// - Given: 空の入力から作った Fastio がある
        /// - When: i64::MIN、i32::MIN を writeln で書き込む
        /// - Then: それぞれの 10 進数表記が改行付きで出力される
        #[test]
        fn writeln_handles_signed_min_values() {
            // Given
            let mut sut = Fastio::from_bytes("");
            // When
            sut.writeln(i64::MIN);
            sut.writeln(i32::MIN);
            // Then
            assert_eq!(
                b"-9223372036854775808\n-2147483648\n".to_vec(),
                sut.into_output()
            );
        }

        /// Scenario: 内部バッファの容量を超える量を書き込んでも、途中でフラッシュされて出力は失われない
        /// - Given: 空の入力から作った Fastio がある
        /// - When: 内部バッファの容量を超えるだけの個数の u32 を writeln で書き込む
        /// - Then: 書き込んだ全ての値が、書き込んだ順番のまま出力される
        #[test]
        fn writeln_survives_multiple_internal_flushes() {
            // Given
            let mut sut = Fastio::from_bytes("");
            let count = 30_000_usize;
            // When
            for i in 0..count {
                sut.writeln(i as u32);
            }
            // Then
            let expected = (0..count)
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join("\n")
                + "\n";
            assert_eq!(expected.into_bytes(), sut.into_output());
        }
    }
}

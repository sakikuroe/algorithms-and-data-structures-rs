# 区間で管理するビット列

`anmitsu::ds::interval_bit_set::IntervalBitSet` は、`0..=usize::MAX` の各位置に真偽値を保持します。初期値はすべて `false` です。同じ値が連続する部分を区間として扱うため、広い範囲を設定する際も各位置のメモリーを確保する必要はありません。

## 公開インターフェース

範囲引数には `RangeBounds<usize>` を使います。検索と集計は 0 と 1 の両方を指定でき、位置の個数と順序には全値域の長さを表せる `u128` を使います。

```rust,ignore
impl IntervalBitSet {
    pub fn new() -> Self;
    pub fn set<R: RangeBounds<usize>>(&mut self, range: R, value: bool) -> u128;
    pub fn flip<R: RangeBounds<usize>>(&mut self, range: R);
    pub fn get(&self, x: usize) -> bool;
    pub fn run_at(&self, x: usize) -> RangeInclusive<usize>;
    pub fn find<R: RangeBounds<usize>>(&self, range: R, value: bool) -> Option<usize>;
    pub fn select<R: RangeBounds<usize>>(
        &self, range: R, value: bool, k: u128,
    ) -> Option<usize>;
    pub fn count<R: RangeBounds<usize>>(&self, range: R, value: bool) -> u128;
    pub fn count_ranges<R: RangeBounds<usize>>(&self, range: R, value: bool) -> usize;
    pub fn ranges<R: RangeBounds<usize>>(
        &self, range: R, value: bool,
    ) -> impl DoubleEndedIterator<Item = RangeInclusive<usize>> + '_;
}
```

これに加えて `Clone`、`Debug`、`Default`、`PartialEq`、`Eq`、論理演算、シフト、区間列を対象とする `FromIterator` と `Extend` に対応します。`Default` は `new()` と同じ状態です。等値比較は更新履歴によらず、各位置の値が同じかを判定します。

## 範囲の設定と点の取得

`set(range, value)` は範囲を指定値に変更し、値が変わった位置の個数を `u128` で返します。既に同じ値の位置は個数に含めません。`get(x)` は点の値、`run_at(x)` はその点を含む同じ値の最大の連続区間を返します。

`flip(range)` は範囲内の 0 と 1 を反転します。返り値は `()` です。反転対象だけが 1 のマスクと XOR する場合と同じ結果ですが、対象範囲外の区間を走査しません。

```rust
use anmitsu::ds::interval_bit_set;

let mut bits = interval_bit_set::IntervalBitSet::new();
assert_eq!(6, bits.set(2..8, true));
assert_eq!(2, bits.set(4..6, false));
assert!(!bits.get(4));
assert_eq!(4..=5, bits.run_at(4));
```

## 検索、集計と列挙

`find(range, value)` は範囲内で指定値を持つ最小の添字を `Option<usize>` で返します。`find(x.., false)` によって、`x` 以上の最小の未登録整数である mex を取得できます。該当する位置がない場合は `None` です。

`count(range, value)` は範囲内で指定値を持つ位置の個数を `u128` で返します。全範囲の個数は `usize::MAX + 1` になるため、添字より大きな整数型を使います。

`select(range, value, k)` は、範囲内で指定値を持つ位置を小さい順に並べたときの、0 始まりの `k` 番目を `Option<usize>` で返します。個数が足りなければ `None` です。`k` が大きくても、位置や区間を先頭から一つずつ数えません。`select(range, value, 0)` は `find(range, value)` と同じ結果です。

`count_ranges(range, value)` は、指定範囲で切り取った後の、指定値の最大連続区間の個数を `usize` で返します。例えば、1 の区間が `2..8` と `10..15` なら、`count_ranges(4..12, true)` は `2`、`count_ranges(4..12, false)` は `1` です。

`ranges(range, value)` は、指定範囲内で指定値が連続する区間を昇順に列挙します。返す区間は指定範囲で切り取られます。`ranges(.., true)` で 1 の全区間、`ranges(.., false)` で 0 の全区間を取得できます。双方向 iterator なので `.rev()`、`next_back()`、`next()` との交互取得にも対応します。

```rust
use anmitsu::ds::interval_bit_set;

let mut bits = interval_bit_set::IntervalBitSet::new();
bits.set(2..8, true);
bits.set(10..15, true);
assert_eq!(Some(8), bits.find(4.., false));
assert_eq!(6, bits.count(4..12, true));
assert_eq!(vec![4..=7, 10..=11],
    bits.ranges(4..12, true).collect::<Vec<_>>());
assert_eq!(vec![8..=9],
    bits.ranges(4..12, false).collect::<Vec<_>>());
assert!(bits.find(2..8, false).is_none());
assert_eq!(Some(9), bits.select(4..12, false, 1));
assert_eq!(2, bits.count_ranges(4..12, true));
```

## 列どうしの論理演算

2 個の `IntervalBitSet` の共通部分は `&`、和集合は `|`、片方だけが 1 の位置からなる対称差は `^` で求めます。各演算は新しい `IntervalBitSet` を返し、借用した入力を変更しません。結果も区間が結合された状態になり、そのまま `set`、`find`、`count`、`ranges` を利用できます。

```rust
use anmitsu::ds::interval_bit_set;

let mut a = interval_bit_set::IntervalBitSet::new();
let mut b = interval_bit_set::IntervalBitSet::new();
a.set(2..8, true);
b.set(5..10, true);

let both = &a & &b;
let either = &a | &b;
let exactly_one = &a ^ &b;
assert_eq!(3, both.count(.., true));
assert_eq!(8, either.count(.., true));
assert_eq!(vec![2..=4, 8..=9],
    exactly_one.ranges(.., true).collect::<Vec<_>>());

a &= &b;
assert_eq!(vec![5..=7], a.ranges(.., true).collect::<Vec<_>>());
```

左右のオペランドは所有値・参照のどちらでも指定でき、`a & b`、`a & &b`、`&a & b`、`&a & &b` に対応します。OR・XOR も同様です。`(&a & &b) | &c` のように演算結果を続けて渡せます。左辺を更新する `&=`、`|=`、`^=` は、右辺に所有値と参照のどちらも受け付けます。

演算は `0..=usize::MAX` の同じ添字位置どうしに適用します。XOR では、両方が 1 の位置は 0 に、片方だけが 1 の位置は 1 になります。したがって、同じ列どうしの XOR は全体が 0 になります。

補集合は `!&bits` または `!bits` で求めます。基準は全値域であり、`&a & !&b` で差集合を表せます。有限の時間帯だけが必要な場合は、結果の `ranges(range, true)` を使って切り取ります。

## シフトと区間列からの構築

`&bits << distance` は位置 `x` の値を `x + distance` へ、`&bits >> distance` は `x - distance` へ移動します。シフト量は `usize` です。値域外へ移る部分を切り捨て、空いた位置には 0 を入れます。`distance` の剰余は取りません。所有値を使う `bits << distance`・`bits >> distance` と、代入する `<<=`・`>>=` にも対応します。

区間列は `collect::<IntervalBitSet>()` または `IntervalBitSet::from_iter(ranges)` で構築できます。各区間の和集合を作るため、区間の順序・重複・隣接を気にする必要はありません。`extend(ranges)` は既存の列に各区間を追加します。いずれも区間を受け付け、整数の列は受け付けません。点の列なら `points.map(|x| x..=x)` として渡せます。

```rust
use anmitsu::ds::interval_bit_set;

let a = [2..8, 10..15].into_iter()
    .collect::<interval_bit_set::IntervalBitSet>();
let b = [4..6, 12..20].into_iter()
    .collect::<interval_bit_set::IntervalBitSet>();
let available = &a & !&b;
let shifted = &available << 3_usize;
let window = shifted.ranges(5..16, true)
    .collect::<interval_bit_set::IntervalBitSet>();
assert_eq!(vec![5..=6, 9..=10, 13..=14],
    window.ranges(.., true).collect::<Vec<_>>());
```

## 範囲の境界

範囲引数は `RangeBounds<usize>` で指定します。`l..r`、`l..=r`、`..r`、`..=r`、`l..`、`..` に加え、`Bound` の組で排他的下限も指定できます。下端の省略は `0`、上端の省略は `usize::MAX` を含む指定です。

`run_at` と `ranges` の返す区間は `RangeInclusive<usize>` です。閉区間にすることで、`usize::MAX` を含む末尾も表せます。空範囲・逆転範囲・`usize::MAX` を除外する下限では、`set`・`count`・`count_ranges` は `0`、`find`・`select` は `None`、`ranges` は空の iterator を返し、`flip` は何もしません。

## 計算量

保持する 1 の区間数を `K`、更新で触れる区間数を `A` とすると、`set` は `O((A + 1) log(K + 2))` 時間です。隣接区間を結合し、削除で残る両端だけを保持します。記憶領域は `O(K)` です。

`get`、`run_at`、`find` は `O(log(K + 1))` 時間です。`ranges` は構築に `O(log(K + 1))`、`A` 区間の列挙に償却 `O(A + 1)` 時間を使い、列挙結果の配列を事前に確保しません。

`count(.., value)` は全体の 1 の個数を利用して `O(1)` 時間で返します。部分範囲の `count`、`select`、`count_ranges` は `O(log(K + 1))` 時間、追加空間は `O(1)` です。走査用の `BTreeMap` と別に、各部分木の 1 の位置数と区間数を集計する AVL 木を持ち、更新時に同期します。両方を合わせた記憶領域も `O(K)` です。`set` の再帰的な更新には `O(log(K + 2))` のスタック領域を使います。

`flip` は触れる区間数を `A` として `O((A + 1) log(K + A + 2))` 時間、一時的な追加空間は `O(A + log(K + A + 2))` です。範囲反転のたびに多数の区間をまたぐ入力では、その区間数に比例する走査が必要です。

列どうしの論理演算では、入力の区間数を `K, L`、結果の区間数を `R` とすると、境界の走査は `O(K + L)` 時間です。結果を保持する木の構築を含む時間計算量の上限は `O(K + L + R log(R + 1))`、追加空間は `O(R)` です。代入演算も同じです。添字の幅に比例する処理や、位置ごとの展開は行いません。

補集合とシフトの区間走査は `O(K + 1)` 時間です。結果の木の構築を含む保守的な上限は `O((K + 1) log(K + 2))` 時間、追加空間は `O(K + 1)` です。`N` 個の区間からの `FromIterator` は並べ替えと結合を含めて `O(N log(N + 1))` 時間、追加空間は `O(N)` です。`Extend` は各入力区間への `set` と同じ計算量です。

## 想定問題での使い分け

予約を追加・解除して k 番目の空き番号を割り当てる問題は、`select(range, false, k)` と `set(x..=x, true)` で記述できます。島の追加・削除・反転後の島の個数は、`set`・`flip` と `count_ranges(range, true)` で求めます。どちらも問い合わせのたびに全区間を数えません。

予定をずらして共通の空きから禁止時間を取り除く問題は、`&alice & (&bob << distance) & !&blocked` と記述できます。重さ `w` を使う部分和 DP の更新は `reachable |= &reachable << w` です。これらの演算は区間数に応じた計算量なので、圧縮できる区間が少ない入力に向きます。部分和が飛び飛びになる入力では、通常のビット列のほうが適します。

時刻 `x` 以前の最後の空き位置は `ranges(..=x, false).next_back().map(|run| *run.end())` で取得できます。長さ `d` 以上の最初の空き区間は、`ranges(range, false).find(...)` で区間長を調べます。最長の空きも区間長の最大値で求められます。ただし、これらの長さによる検索は対象区間を走査します。多数の連続ブロック検索を対数時間で処理する機能は持ちません。

## 操作の組み合わせによる利用例

[ABC480 C - Self-Stopping Shower](https://atcoder.jp/contests/abc480/tasks/abc480_c) では、時刻 `a` の操作による給湯を `set(a..a + k, true)` で登録できます。添字 `x` は時間帯 `[x, x + 1)` を表します。`ranges(0..t, false)` で対象時間内の未給湯区間を列挙し、それぞれの長さを `k` で割って切り上げた回数を合計します。時刻 `t` より後の操作も、列挙範囲を指定することで自然に除外できます。

[ABC330 E - Mex and Update](https://atcoder.jp/contests/abc330/tasks/abc330_e) では、数列中に存在する値を `true` として登録します。出現回数は数列側で管理し、最後の出現を削除したときだけ `set(value..=value, false)`、初めて追加したときだけ `set(value..=value, true)` を呼びます。`find(.., false)` が更新後の mex です。同じ値が複数回現れることと、集合への所属を分けて管理できます。

[ABC217 D - Cutting Woods](https://atcoder.jp/contests/abc217/tasks/abc217_d) では、切断点と木材の外側を `true` とします。未切断の位置 `x` に対する `run_at(x)` は、左右の切断点に挟まれた目印の区間を返します。切断点が `a` と `b` なら返る区間は `[a + 1, b - 1]` なので、その要素数に 1 を加えると木材の長さ `b - a` が求まります。

[ABC256 D - Union of Interval](https://atcoder.jp/contests/abc256/tasks/abc256_d) では、各入力区間に `set(left..right, true)` を適用し、`ranges(.., true)` で和集合の最大の連続区間を列挙します。この問題の右端は `usize::MAX` より小さいため、返された閉区間の右端に 1 を加えて、要求される半開区間の形に戻せます。

[ABC294 E - 2xN Grid](https://atcoder.jp/contests/abc294/tasks/abc294_e) では、上下それぞれの行について、値ごとの位置集合を作ります。同じ値の位置集合を `&upper_positions & &lower_positions` で AND し、結果の `count(.., true)` を全値について合計すると、上下の値が一致する列数が求まります。連長を区間として登録するため、最大 `10^12` 列の入力をマスごとに展開する必要はありません。

これらの利用では、区間の和集合と補集合、値の削除と mex、点を含む区間の取得、列どうしの論理演算を、共通の API の組み合わせで記述できます。

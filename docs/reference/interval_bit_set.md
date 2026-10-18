# 区間で管理するビット列

`anmitsu::ds::interval_bit_set::IntervalBitSet` は、`0..=usize::MAX` の各位置に真偽値を保持します。初期値はすべて `false` です。同じ値が連続する部分を区間として扱うため、広い範囲を設定する際も各位置のメモリーを確保する必要はありません。

## 範囲の設定と点の取得

`set(range, value)` は範囲を指定値に変更し、値が変わった位置の個数を `u128` で返します。既に同じ値の位置は個数に含めません。`get(x)` は点の値、`run_at(x)` はその点を含む同じ値の最大の連続区間を返します。

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
```

## 範囲の境界

範囲引数は `RangeBounds<usize>` で指定します。`l..r`、`l..=r`、`..r`、`..=r`、`l..`、`..` に加え、`Bound` の組で排他的下限も指定できます。下端の省略は `0`、上端の省略は `usize::MAX` を含む指定です。

`run_at` と `ranges` の返す区間は `RangeInclusive<usize>` です。閉区間にすることで、`usize::MAX` を含む末尾も表せます。空範囲・逆転範囲・`usize::MAX` を除外する下限では、`set` と `count` は `0`、`find` は `None`、`ranges` は空の iterator を返します。

## 計算量

保持する 1 の区間数を `K`、更新で触れる区間数を `A` とすると、`set` は `O((A + 1) log(K + 2))` 時間です。隣接区間を結合し、削除で残る両端だけを保持します。記憶領域は `O(K)` です。

`get`、`run_at`、`find` は `O(log(K + 1))` 時間です。`ranges` は構築に `O(log(K + 1))`、`A` 区間の列挙に償却 `O(A + 1)` 時間を使い、列挙結果の配列を事前に確保しません。

`count(.., value)` は全体の 1 の個数を利用して `O(1)` 時間で返します。部分範囲では、触れる 1 の区間数を `A` として `O(log(K + 1) + A)` 時間です。

# 1 次元の疎なセグメント木

このページは、`ds::segment_tree::segment_tree_sparse_online` と
`ds::segment_tree::segment_tree_sparse_offline` の公開 API を記述する。
どちらも半開区間 `[0, len)` 上の値をモノイドで集約し、
未設定位置を単位元として扱う。

## 構築と座標

Online 版は `SegmentTreeSparseOnline::<M>::new(len)` で作成する。
`0 <= idx < len` を満たす位置は、構築後にいつでも設定できる。
節点は設定した位置への経路上だけに作られる。

Offline 版は `SegmentTreeSparseOffline::<M>::new(len, points)` で作成する。
`points` は更新され得る座標を列挙したものであり、順序は問わず、
重複は除去される。各座標は `len` 未満である必要がある。
構築後に新しい座標は登録できない。区間クエリの端点は登録不要である。
未登録位置の `get` は単位元を返し、未登録位置の `set` と `update` は
パニックする。

`len()` は論理上の区間長を返す。`is_empty()` は `len == 0` のときだけ
真となるため、offline 版で登録座標がない場合でも `len > 0` なら偽となる。

## 公開メソッド

両実装の `new` は全位置が単位元の木を作成する。`len` は論理上の
区間長を返し、`is_empty` はその長さが 0 かを返す。`get(idx)` は
一点の値を返し、`fold(l, r)` は半開区間 `[l, r)` を座標順に集約する。

`set(idx, x)` は葉だけを変更するため、区間集約または境界探索より
前に `build()` を呼ぶ。`build()` はすべての葉から集約値を再構築する。
`update(idx, x)` は葉と祖先の集約値を直ちに更新する。

`max_right(l, f)` は `f(fold(l, r))` が真となる最大の右端を返す。
`min_left(r, f)` は `f(fold(l, r))` が真となる最小の左端を返す。

`fold` は `0 <= l <= r <= len` を要求し、空区間では単位元を返す。
点操作は `idx < len` を要求する。範囲外ではパニックする。
境界探索は `f` が単位元で真となり、探索方向に真から偽へ一度だけ
変わることを前提とする。条件が最後まで真の場合、`max_right` は
`len`、`min_left` は `0` を返す。offline 版でも結果は圧縮後の番号
ではなく元の座標である。

## 計算量

`U = len`、`P` を offline 版へ渡した座標数、`K` を異なる登録座標数、
`N` を online 版で生成された節点数とする。

Online 版の `new` は時間・領域ともに `O(1)` である。`set`、
`update`、`get`、`fold`、境界探索は `O(log U)` 時間で動作し、
`build` は `O(N)` 時間で動作する。1 回の点設定で増える節点数は
高々 `O(log U)` である。

Offline 版の `new` は `O(P log P + K)` 時間と `O(K)` 領域を使う。
`set`、`update`、`get`、`fold`、境界探索は `O(log K)` 時間で動作し、
`build` は `O(K)` 時間で動作する。

Online 版の `N` は、設定した異なる位置の数を `T` とすると
最大 `O(T log U)` である。`U = 0` または `K = 0` の場合も利用できる。

## 使用例

```rust
use anmitsu::{algebra::monoid, ds::segment_tree};

let mut online = segment_tree::segment_tree_sparse_online::
    SegmentTreeSparseOnline::<monoid::AddMonoid>::new(100);
online.update(30, 4);
assert_eq!(online.fold(0, 50), 4);

let mut offline = segment_tree::segment_tree_sparse_offline::
    SegmentTreeSparseOffline::<monoid::AddMonoid>::new(100, [30, 80]);
offline.set(30, 4);
offline.set(80, 6);
offline.build();
assert_eq!(offline.fold(50, 100), 6);
```

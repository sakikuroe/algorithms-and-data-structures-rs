# 疎な offline セグメント木

1 次元の `SegmentTreeSparseOffline` と 2 次元の
`SegmentTree2dSparseOffline` は、更新する可能性がある座標だけを構築時に登録します。
クエリの端点は登録不要です。

```rust
use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_sparse_offline};

let mut seg = segment_tree_sparse_offline::SegmentTreeSparseOffline::<
    monoid::AddMonoid,
>::new([3, 100]);
seg.update(100, 7);
assert_eq!(7, seg.fold(50..1_000_000));
```

2 次元でも構築時に必要なのは更新候補点だけです。

```rust
use anmitsu::{algebra::monoid, ds::segment_tree::segment_tree_2d_sparse_offline};

let mut seg = segment_tree_2d_sparse_offline::SegmentTree2dSparseOffline::<
    monoid::AddMonoid,
>::new([(3, 100), (8, 200)]);
seg.update((3, 100), 7);
assert_eq!(7, seg.fold(0..1_000_000, 0..1_000_000));
```

両版の座標は `[0, usize::MAX)` です。`fold` は元の座標で
`RangeBounds<usize>` を受け付け、`..` は全登録座標を含みます。
未登録点の `get` は単位元を返し、未登録点への `set`・`update` はパニックします。
`set` で値を置いた後は、集約の前に `build` を呼びます。

1 次元の `max_right` は、述語が最後まで真なら `usize::MAX` を返します。
`min_left` は、左端まで真なら `0` を返します。
2 次元の矩形集約には可換モノイドが必要です。

疎な online 版は座標区間を分割して節点を生成するため、構築時に論理領域の長さを指定します。

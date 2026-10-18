# Changelog

## Unreleased

- オンライン版の疎な遅延セグメント木を、更新端点で一様な区間を分割する AVL 木へ変更しました。公開 API と座標の `usize` 型を維持し、更新回数に対する節点数を `O(Q)` に抑えます。子の参照には `u32`、作用には `Option<F>` を使い、ビット列と未初期化領域の手動管理は廃止しました。確保する節点数の上限は `u32::MAX` 個です。詳細は [リファレンス](docs/reference/lazy_segment_tree_sparse_online.md) を参照してください。

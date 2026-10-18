# Changelog

## Unreleased

- オンライン版の疎な遅延セグメント木で、子の参照を `u32` にし、未伝播作用の有無をビット列へ分離しました。公開 API と座標の `usize` 型は維持しています。生成節点数の上限は `u32::MAX` 個になりました。詳細は [リファレンス](docs/reference/lazy_segment_tree_sparse_online.md) を参照してください。

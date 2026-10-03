# 変更履歴

## [Unreleased]

### Changed

- 1 次元・2 次元の疎な offline セグメント木は、構築時の座標上限指定を廃止し、更新候補座標だけで構築できるようにしました。既存の `new` 呼び出しは引数の変更が必要です。
- 1 次元の疎な offline セグメント木では、`max_right` が全登録座標を通過した場合に `usize::MAX` を返します。
- 疎な offline セグメント木の `len`、`height`、`width`、`is_empty` による論理領域の取得は廃止しました。

### Added

- Library Checker の `point_add_rectangle_sum` で疎な online・offline の 2 次元セグメント木を確認する統合テストを追加しました。

### Fixed

- `bundler --strip-docs` が不要な再エクスポートを残して枝刈りを途中で止める問題を修正しました。

# Changelog

## Unreleased

- 区間で 0/1 列を管理する `IntervalBitSet` を追加しました。範囲の設定、点の値と連続区間の取得、値の検索、個数の集計、両方の値の区間列挙に対応します。詳細は [リファレンス](docs/reference/interval_bit_set.md) を参照してください。
- `IntervalBitSet` を使う ABC480 C、ABC330 E、ABC217 D、ABC256 D の実行バイナリーと統合テストを追加しました。公式サンプル、境界例、独立した素朴解との照合で、既存の API の組み合わせを検証します。

use super::super::common;

/// `lc-jump-on-tree` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-jump-on-tree");

// lc-jump-on-tree のテスト: 標準入力にサンプルを与えたときの標準出力を検証する
mod lc_jump_on_tree {
    use super::*;

    /// Scenario: 手計算で検証したサンプルを解いたときの標準出力を検証する
    /// - Given: 0-1-2, 1-3, 3-4 の辺を持つ木があり、頂点0から頂点4への
    ///   パス (0,1,3,4) 上の各位置と、範囲外の位置を問い合わせる
    /// - When: lc-jump-on-tree バイナリへ標準入力として渡す
    /// - Then: 各クエリの出力が、手計算した期待値と一致する
    #[test]
    fn matches_hand_verified_sample() {
        // Given
        let input = "5 5\n0 1\n1 2\n1 3\n3 4\n0 4 0\n0 4 1\n0 4 2\n0 4 3\n0 4 4\n";
        let expected = "0\n1\n3\n4\n-1\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }

    /// Scenario: 起点と終点が同じ場合と、パスの向きを逆にした場合の
    /// 出力を検証する (境界値)
    /// - Given: 上記と同じ木があり、頂点2から頂点2自身へのクエリ2件と、
    ///   頂点4から頂点0へ (逆向きの) パス上の位置を問い合わせる
    /// - When: lc-jump-on-tree バイナリへ標準入力として渡す
    /// - Then: 自身へのパスは頂点2自身のみで、それ以降は -1 になり、
    ///   逆向きのパス (4,3,1,0) 上の位置2は頂点1になる
    #[test]
    fn handles_same_endpoint_and_reversed_direction() {
        // Given
        let input = "5 3\n0 1\n1 2\n1 3\n3 4\n2 2 0\n2 2 1\n4 0 2\n";
        let expected = "2\n-1\n1\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

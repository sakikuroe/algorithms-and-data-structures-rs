use super::super::common;

/// `lc-vertex-set-path-composite` バイナリへのパス。
const BIN: &str = env!("CARGO_BIN_EXE_lc-vertex-set-path-composite");

// lc-vertex-set-path-composite のテスト: 標準入力にサンプルを与えたときの
// 標準出力を検証する
mod lc_vertex_set_path_composite {
    use super::*;

    /// Scenario: 手計算で検証したサンプルを解いたときの標準出力を検証する
    /// - Given: 0-1-2 の単純パスに、頂点ごとに異なる一次関数
    ///   (f_0(x)=2x+1, f_1(x)=3x, f_2(x)=x+5) を持たせた木がある。
    ///   経路の向きで結果が変わること (非可換であること)、および頂点の
    ///   更新が以降のクエリに反映されることを確認する
    /// - When: lc-vertex-set-path-composite バイナリへ標準入力として渡す
    /// - Then: 各クエリの出力が、手計算した期待値と一致する
    #[test]
    fn matches_hand_verified_sample() {
        // Given
        let input = "3 4\n2 1\n3 0\n1 5\n0 1\n1 2\n1 0 2 1\n1 2 0 1\n0 1 5 2\n1 0 2 1\n";
        let expected = "14\n37\n22\n";
        // When
        let result = common::run_binary(BIN, input);
        // Then
        assert_eq!(expected, result);
    }
}

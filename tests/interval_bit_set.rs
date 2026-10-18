//! 想定問題を公開 API だけで解き、区間列としての組み合わせを検証します。

use anmitsu::ds::interval_bit_set;
use std::iter;

/// Scenario: 時間帯内の k 番目の空き番号を選び、その番号を予約します。
/// - Given: 巨大な番号空間の一部が予約済みです。
/// - When: 指定時間帯内の k 番目の空き番号を予約し、別の予約を解除します。
/// - Then: 区間を展開せずに割当番号と残りの空き個数を求められます。
#[test]
fn allocates_the_kth_available_number() {
    // Given
    let end = usize::MAX / 2;
    let mut sut = interval_bit_set::IntervalBitSet::from_iter(iter::once(10..end));
    // When
    let allocated = sut.select(5..=end + 10, false, 8).unwrap();
    sut.set(allocated..=allocated, true);
    sut.set(end - 2..end, false);
    // Then
    assert_eq!(end + 3, allocated);
    assert_eq!(Some(end - 2), sut.select(10..=end + 10, false, 0));
    assert_eq!(12, sut.count(10..=end + 10, false));
    assert_eq!(None, sut.select(10..=end + 10, false, 12));
}

/// Scenario: 陸地を追加、反転、削除して、指定領域内の島の数を答えます。
/// - Given: 海に隔てられた 2 個の島があります。
/// - When: 海を埋め立て、岸を反転させ、中央に水路を作ります。
/// - Then: 全体と切り取った領域の島および海の区間数を答えられます。
#[test]
fn tracks_islands_and_water_components() {
    // Given
    let mut sut = interval_bit_set::IntervalBitSet::from_iter([2..8, 10..15]);
    // When
    sut.set(8..10, true);
    let joined = sut.count_ranges(.., true);
    sut.flip(0..4);
    sut.set(7..12, false);
    // Then
    assert_eq!(1, joined);
    assert_eq!(
        vec![0..=1, 4..=6, 12..=14],
        sut.ranges(.., true).collect::<Vec<_>>()
    );
    assert_eq!(3, sut.count_ranges(.., true));
    assert_eq!(2, sut.count_ranges(1..13, false));
    assert_eq!(2, sut.count_ranges(5..14, true));
}

/// Scenario: 予定を平行移動し、禁止時間を除いた共通の空きを切り出します。
/// - Given: 2 人の空き時間と、共通の禁止時間があります。
/// - When: 片方を 3 時間移動し、共通部分から禁止時間を除きます。
/// - Then: 時間帯内の候補を少ない式で構成し、更新可能な独立した列にできます。
#[test]
fn shifts_and_extracts_shared_availability() {
    // Given
    let alice = interval_bit_set::IntervalBitSet::from_iter([2..12, 20..30]);
    let bob = interval_bit_set::IntervalBitSet::from_iter([0..8, 15..24]);
    let blocked = interval_bit_set::IntervalBitSet::from_iter([6..9, 21..23]);
    // When
    let available = &alice & (&bob << 3_usize) & !blocked;
    let mut sut = available
        .ranges(5..25, true)
        .collect::<interval_bit_set::IntervalBitSet>();
    sut.extend(iter::once(30..32));
    // Then
    assert_eq!(
        vec![5..=5, 9..=10, 20..=20, 23..=24, 30..=31],
        sut.ranges(.., true).collect::<Vec<_>>()
    );
    assert_eq!(8, sut.count(.., true));
    assert_eq!(Some(3), available.find(.., true));
    assert!(!available.get(30));
}

/// Scenario: 各重さを 1 回だけ使う部分和を、シフトと OR で更新します。
/// - Given: 大きな重さと、その前後の重さの品物があります。
/// - When: 各品物を使った到達可能集合をシフトして和集合へ追加します。
/// - Then: 巨大な座標でも隣り合う部分和が区間へ圧縮されます。
#[test]
fn computes_large_subset_sums_with_shifts() {
    // Given
    let weight = usize::MAX / 16;
    let mut sut = interval_bit_set::IntervalBitSet::from_iter([0..=0]);
    // When
    for item in [1, weight, weight + 1] {
        sut |= &sut << item;
    }
    // Then
    assert_eq!(
        vec![0..=1, weight..=weight + 2, 2 * weight + 1..=2 * weight + 2],
        sut.ranges(.., true).collect::<Vec<_>>()
    );
    assert_eq!(7, sut.count(.., true));
    assert_eq!(Some(weight + 2), sut.select(weight.., true, 2));
}

/// Scenario: 連続した空き時間の検索と直前の空き位置を区間列挙で求めます。
/// - Given: 予約で分割された時間帯があります。
/// - When: 長さ 4 の最初の空きを予約し、直前の空きを探します。
/// - Then: 専用の検索メソッドを増やさずに区間の両端を利用できます。
#[test]
fn reserves_a_contiguous_free_block() {
    // Given
    let mut sut = interval_bit_set::IntervalBitSet::from_iter([0..3, 5..10, 16..20]);
    // When
    let first = sut
        .ranges(0..20, false)
        .find(|run| *run.end() as u128 - *run.start() as u128 + 1 >= 4)
        .map(|run| *run.start())
        .unwrap();
    sut.set(first..first + 4, true);
    let previous = sut.ranges(..=13, false).next_back().map(|run| *run.end());
    // Then
    assert_eq!(10, first);
    assert_eq!(Some(4), previous);
    assert_eq!(Some(14), sut.find(first..20, false));
}

/// Scenario: 巨大な座標の多数の区間でも、順序検索と集計は全走査しません。
/// - Given: 等間隔で 20,000 個の長い 1 の区間があります。
/// - When: 部分範囲の集計と順序検索、遠方の区間反転を繰り返します。
/// - Then: 位置を展開せず、結果が等差数列から計算した値と一致します。
#[test]
fn queries_many_runs_at_large_coordinates() {
    // Given
    let size = 20_000;
    let stride = usize::MAX / (4 * size);
    let mut sut = (0..size)
        .map(|x| 2 * x * stride..(2 * x + 1) * stride)
        .collect::<interval_bit_set::IntervalBitSet>();
    // When
    for x in 0..size {
        let start = 2 * x * stride;
        let end = 2 * size * stride;
        // Then
        assert_eq!(
            (size - x) as u128 * stride as u128,
            sut.count(start..end, true)
        );
        assert_eq!(size - x, sut.count_ranges(start..end, true));
        assert_eq!(
            Some(start),
            sut.select(..end, true, x as u128 * stride as u128)
        );
        assert_eq!(
            Some(start + stride),
            sut.select(..end, false, x as u128 * stride as u128)
        );
        // When
        sut.flip(2 * size * stride..3 * size * stride);
        // Then
        assert_eq!(size + usize::from(x % 2 == 0), sut.count_ranges(.., true));
    }
    // When
    sut.flip(2 * size * stride..3 * size * stride);
    // Then
    assert_eq!(size + 1, sut.count_ranges(.., true));
}

/// Scenario: 小さな全パターンで反転、集計、順序検索を素朴解と照合します。
/// - Given: 0 付近と usize::MAX 付近に置いた、8 ビットの全パターンがあります。
/// - When: 各部分範囲を問い合わせ、反転、補集合、シフトを適用します。
/// - Then: 配列で求めた個数、連続区間数、順序、点ごとの移動結果に一致します。
#[test]
fn matches_dense_models_at_both_domain_edges() {
    // Given
    let size = 8;
    for base in [0, usize::MAX - size + 1] {
        for mask in 0..1 << size {
            let expected = (0..size).map(|x| mask & (1 << x) != 0).collect::<Vec<_>>();
            let sut = (0..size)
                .filter(|&x| expected[x])
                .map(|x| base + x..=base + x)
                .collect::<interval_bit_set::IntervalBitSet>();
            for start in 0..size {
                for end in start..size {
                    for value in [false, true] {
                        let positions = (start..=end)
                            .filter(|&x| expected[x] == value)
                            .map(|x| base + x)
                            .collect::<Vec<_>>();
                        let runs = (start..=end)
                            .filter(|&x| {
                                expected[x] == value && (x == start || expected[x - 1] != value)
                            })
                            .count();
                        // When
                        let range = base + start..=base + end;
                        // Then
                        assert_eq!(positions.len() as u128, sut.count(range.clone(), value));
                        assert_eq!(runs, sut.count_ranges(range.clone(), value));
                        for k in 0..=positions.len() {
                            assert_eq!(
                                positions.get(k).copied(),
                                sut.select(range.clone(), value, k as u128)
                            );
                        }
                    }
                    // When
                    let mut flipped = sut.clone();
                    flipped.flip(base + start..=base + end);
                    // Then
                    for (x, &before) in expected.iter().enumerate() {
                        assert_eq!(before ^ (start..=end).contains(&x), flipped.get(base + x));
                    }
                    assert_eq!(
                        flipped.count(.., true),
                        flipped.count(base..=base + (size - 1), true)
                    );
                }
            }
            // When
            let complement = !&sut;
            // Then
            assert_eq!(
                usize::MAX as u128 + 1 - expected.iter().filter(|&&v| v).count() as u128,
                complement.count(.., true)
            );
            for distance in [0, 1, size, usize::MAX] {
                // When
                let left = &sut << distance;
                let right = &sut >> distance;
                let expected_left = expected
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v)
                    .filter_map(|(x, _)| (base + x).checked_add(distance))
                    .collect::<Vec<_>>();
                let expected_right = expected
                    .iter()
                    .enumerate()
                    .filter(|&(_, &v)| v)
                    .filter_map(|(x, _)| (base + x).checked_sub(distance))
                    .collect::<Vec<_>>();
                // Then
                assert_eq!(
                    expected_left,
                    left.ranges(.., true).flatten().collect::<Vec<_>>()
                );
                assert_eq!(
                    expected_right,
                    right.ranges(.., true).flatten().collect::<Vec<_>>()
                );
            }
        }
    }
}

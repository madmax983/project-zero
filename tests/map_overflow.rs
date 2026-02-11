use scale::layer1::map::GridPosition;

#[test]
fn test_distance_chebyshev_overflow_safe() {
    let p1 = GridPosition { x: i32::MIN, y: 0 };
    let p2 = GridPosition { x: i32::MAX, y: 0 };

    // Distance should be u32::MAX (2^32 - 1)
    // -2147483648 to 2147483647 diff is 4294967295
    let dist = p1.distance_chebyshev(p2);
    assert_eq!(dist, u32::MAX);
}

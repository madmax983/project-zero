use scale::layer1::map::GridPosition;

#[test]
fn test_exploit_abs_overflow() {
    let p1 = GridPosition {
        x: i32::MAX,
        y: i32::MAX,
    };
    let p2 = GridPosition { x: -2, y: -2 };

    // With `abs_diff` this does not panic
    let dx = p1.x.abs_diff(p2.x);
    let dy = p1.y.abs_diff(p2.y);
    assert!(dx > 0);
    assert!(dy > 0);
}

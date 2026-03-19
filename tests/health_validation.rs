#![allow(missing_docs)]
use scale::layer1::health::Health;

#[test]
fn test_take_damage_negative_healing() {
    let mut health = Health {
        current: 50.0,
        ..Default::default()
    };
    // Exploit: healing via negative damage
    health.take_damage(-50.0);
    // Should stay 50.0 if fixed, but currently becomes 100.0
    assert_eq!(
        health.current, 50.0,
        "Health should not increase from negative damage"
    );
}

#[test]
fn test_take_damage_nan() {
    let mut health = Health {
        current: 50.0,
        ..Default::default()
    };
    // Exploit: NaN could cause weird behavior
    health.take_damage(f32::NAN);
    // Should probably ignore NaN or handle safely
    assert!(health.is_alive(), "Health became NaN or invalid");
    assert!(!health.current.is_nan(), "Health current is NaN");
}

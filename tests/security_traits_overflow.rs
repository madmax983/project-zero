use scale::layer1::traits::{Trait, Traits};

#[test]
fn test_exploit_traits_overflow_dos() {
    // RED Phase: This will fail if `1 << (t as u8)` is used because `1` is i32, causing a shift overflow panic on variants > 31.
    // It will also fail if `Traits` is `u64` because there are 105 traits now.

    let mut t = Traits::default();

    // Transhumanist is index 82 (> 63, > 31)
    t.add(Trait::Transhumanist);
    t.add(Trait::ExtremeHunger); // Last trait (104)

    assert!(
        t.has(Trait::Transhumanist),
        "Failed to detect Transhumanist due to shift overflow or mask limit"
    );
    assert!(
        t.has(Trait::ExtremeHunger),
        "Failed to detect ExtremeHunger due to shift overflow or mask limit"
    );

    let all: Vec<_> = t.iter().collect();

    assert!(
        all.contains(&Trait::Transhumanist),
        "Trait iterator failed to include variants > 63."
    );
    assert!(
        all.contains(&Trait::ExtremeHunger),
        "Trait iterator failed to include highest variant."
    );
}

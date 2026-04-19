# Spec 1107: Astrological Beliefs

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Our destiny is written in the stars.
**Mechanic:** Factions develop a complex system of "Zodiacs" based on the celestial bodies in their starting Layer 2 system. The movement of these bodies (e.g., planets aligning, eclipses) applies massive, completely arbitrary buffs or debuffs to their Layer 1 productivity based purely on belief, not physics.
**Emergence:** Your enemy is a technological powerhouse, but they refuse to fight because "The Red Eye is in retrograde." You exploit their superstition by attacking during their "unlucky" season.
**Tension:** Do you indulge the delusion to harvest the massive belief-based buffs during "lucky" alignments, or try to enforce rationalism and deal with the constant, grinding unrest of a population who thinks the universe hates them?

## 2. Dependencies
- Layer 1 Morale/Belief Systems
- Layer 2 Celestial Body Positions

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_astrological_buff_applied_during_alignment() {
    // Arrange: Set up a faction with an Astrological Belief and align celestial bodies
    // Act: Run the astrological buff system
    // Assert: Faction's productivity/morale is massively increased
}

#[test]
fn test_astrological_debuff_applied_during_retrograde() {
    // Arrange: Set up a faction and set celestial bodies to a negative alignment
    // Act: Run the astrological debuff system
    // Assert: Faction's productivity/morale is massively decreased
}

#[test]
fn test_rationalist_faction_ignores_astrology() {
    // Arrange: Set up a faction with Rationalist traits and align celestial bodies
    // Act: Run the astrological system
    // Assert: Faction's productivity/morale is unaffected
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component)]
pub struct AstrologicalBelief {
    pub lucky_alignment: bool,
    pub unlucky_alignment: bool,
}

#[derive(Component)]
pub struct Rationalist;

#[derive(Component)]
pub struct Productivity {
    pub multiplier: f32,
}

pub fn astrological_buff_system(
    mut query: Query<(&AstrologicalBelief, &mut Productivity), Without<Rationalist>>,
) {
    for (belief, mut productivity) in query.iter_mut() {
        if belief.lucky_alignment {
            productivity.multiplier = 1.5; // Massive buff
        } else if belief.unlucky_alignment {
            productivity.multiplier = 0.5; // Massive debuff
        } else {
            productivity.multiplier = 1.0; // Normal
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Alignment Calculations:** Instead of simple booleans, calculate angles between Layer 2 celestial bodies to determine continuous alignment phases (waxing/waning luck).
- **Faction Interaction:** Rationalist factions could suffer a small diplomatic penalty with Astrological factions because they "ignore the signs."
- **Rituals:** Pops might perform rituals during unlucky alignments to offset the debuffs, costing resources and time.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Astrological buffs and debuffs correctly modify productivity based on Layer 2 alignments.
- [ ] Rationalist factions are exempt from astrological effects.

## 7. Technical Guidance
- Link the `lucky_alignment` state to actual positions of planets in the Layer 2 simulation. This creates cross-layer dependency.
- This feature works best if the UI clearly displays the current "Zodiac phase" so players can plan around it.

## 8. Questions
*Builder: add questions here if spec is unclear.*

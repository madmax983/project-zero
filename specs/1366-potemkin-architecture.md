# 1366: Potemkin Architecture

## 1. Overview
**Layer:** 1

**Fantasy:** Deception as defense. Looking stronger than you are.

**Mechanic:** Cheap "Fake Buildings" (Inflatable Turrets, Facade Walls) that look real to enemies/inspectors but have 1 HP and zero function. Reduces Raid probability (Intimidation) but fails instantly in combat.

**Emergence:** You surround your base with hundreds of fake turrets. The Pirate Dreadnought scans you, decides you are "Heavily Defended", and leaves. You saved the colony with balloons.

**Tension:** Invest in real defense (expensive) or bluff (cheap but risky)?

## 2. Dependencies
- Base ECS system
- Building construction system
- Threat/Raid calculation system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::{Building, Health};
    use crate::layer2::threat::ThreatSystem;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, calculate_intimidation_system);
        app.insert_resource(ThreatSystem { base_raid_probability: 0.5, current_raid_probability: 0.5 });
        app
    }

    #[test]
    fn test_potemkin_building_has_1_hp_and_increases_intimidation() {
        let mut app = setup_app();

        // Spawn a real turret
        let real_turret = app.world_mut().spawn((
            Building,
            Health { current: 100.0, max: 100.0 },
            IntimidationValue { value: 10.0 },
        )).id();

        // Spawn a fake turret
        let fake_turret = app.world_mut().spawn((
            Building,
            PotemkinArchitecture,
            Health { current: 1.0, max: 1.0 },
            IntimidationValue { value: 10.0 }, // Looks like the real thing
        )).id();

        app.update();

        let threat = app.world().resource::<ThreatSystem>();

        // Raid probability should be reduced by both
        assert!(threat.current_raid_probability < 0.5);
    }

    #[test]
    fn test_potemkin_building_destroys_instantly() {
        let mut app = setup_app();

        // Spawn a fake turret
        let fake_turret = app.world_mut().spawn((
            Building,
            PotemkinArchitecture,
            Health { current: 1.0, max: 1.0 },
        )).id();

        // Take 2 damage
        let mut health = app.world_mut().get_mut::<Health>(fake_turret).unwrap();
        health.current -= 2.0;

        assert!(health.current <= 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer2::threat::ThreatSystem;

#[derive(Component)]
pub struct PotemkinArchitecture;

#[derive(Component)]
pub struct IntimidationValue {
    pub value: f32,
}

pub fn calculate_intimidation_system(
    query: Query<&IntimidationValue, With<Building>>,
    mut threat_system: ResMut<ThreatSystem>,
) {
    let mut total_intimidation = 0.0;
    for intimidation in query.iter() {
        total_intimidation += intimidation.value;
    }

    // Example logic: reduce probability by 1% per intimidation point, max 40% reduction
    let reduction = (total_intimidation * 0.01).min(0.40);
    threat_system.current_raid_probability = (threat_system.base_raid_probability - reduction).max(0.0);
}
```

## 5. REFACTOR Phase: Quality & Design
- Create specific variants of `PotemkinArchitecture` (e.g. `FacadeWall`, `InflatableTurret`) with appropriate costs.
- Ensure inspectors (if applicable) have a small chance to detect the fake structures based on their skill.
- Consider adding a small ongoing maintenance cost (air pumps, paint).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] `PotemkinArchitecture` buildings reduce raid probability equivalent to real buildings.

## 7. Technical Guidance
- Hook into the existing `ThreatSystem` or `RaidManager`.
- Ensure `Health` components for Potemkin structures are initialized correctly with 1 max HP.

## 8. Questions
*Builder: add questions here if spec is unclear.*

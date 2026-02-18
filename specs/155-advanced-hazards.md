# 155: Advanced Workplace Hazards

## Overview

Working on the frontier is dangerous. While `035` introduced basic hazards, this spec implements a dynamic risk system where **Skill** and **Maintenance** matter. A master engineer working on a pristine reactor is safe. A novice working on a crumbling reactor is dead.

This spec supersedes the basic implementation of `035`.

## Dependencies

- `035` — Workplace Hazards (Basic concept)
- `112` — Maintenance Debt (Structure HP)
- `051` — Pop Skills (XP/Level)
- `151` — Cybernetic Augmentation (Amputation/Prosthetics hook)
- `034` — Pop Health (Damage)

## RED Phase: Tests First

Write these tests in `src/layer1/hazards_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::hazards::{calculate_risk, determine_severity, AccidentSeverity, AmputationEvent};
    use crate::layer1::structure::Structure;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::utility_types::ActionType;
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_risk_scaling_skill() {
        // Base risk for Work is 0.001 (0.1%)
        let base_risk = ActionType::Work.danger_level();

        // Level 0 skill
        let skills_0 = Skills::default();
        let risk_0 = calculate_risk(base_risk, &skills_0, SkillType::Construction, &Structure::default());

        // Level 10 skill (Efficiency 2.0)
        let mut skills_10 = Skills::default();
        skills_10.add_xp(SkillType::Construction, 10000.0); // Sufficient for high level
        let risk_10 = calculate_risk(base_risk, &skills_10, SkillType::Construction, &Structure::default());

        assert!(risk_10 < risk_0, "High skill should reduce risk");
        // Check reduction factor (approx half if skill factor is significant)
        assert!(risk_10 < risk_0 * 0.6);
    }

    #[test]
    fn test_risk_scaling_decay() {
        let base_risk = ActionType::Work.danger_level();
        let skills = Skills::default();

        let pristine = Structure { current_hp: 100.0, max_hp: 100.0 };
        let risk_pristine = calculate_risk(base_risk, &skills, SkillType::Construction, &pristine);

        let crumbling = Structure { current_hp: 10.0, max_hp: 100.0 };
        let risk_crumbling = calculate_risk(base_risk, &skills, SkillType::Construction, &crumbling);

        assert!(risk_crumbling > risk_pristine, "Decay should increase risk");
        // Should be at least double (decay factor ~2.0)
        assert!(risk_crumbling > risk_pristine * 2.0);
    }

    #[test]
    fn test_severity_distribution() {
        // Deterministic check or statistical
        // For unit test, we can check the logic of the helper function directly
        // assuming we pass a seed or random value.
        // Let's assume determine_severity takes a float 0.0-1.0

        assert_eq!(determine_severity(0.5), AccidentSeverity::Minor); // 0.0 - 0.8
        assert_eq!(determine_severity(0.85), AccidentSeverity::Major); // 0.8 - 0.95
        assert_eq!(determine_severity(0.96), AccidentSeverity::Critical); // 0.95 - 1.0
    }

    #[test]
    fn test_amputation_event_generation() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            PopName("Lefty".to_string()),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Trigger critical accident logic
        crate::layer1::hazards::trigger_accident(&mut world, pop, AccidentSeverity::Critical);

        // Check for AmputationEvent
        let events = world.resource::<Events<AmputationEvent>>();
        let mut reader = events.get_reader();
        let event = reader.read(events).next();

        assert!(event.is_some());
        assert_eq!(event.unwrap().entity, pop);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ActionType` Danger

Ensure `danger_level()` returns meaningful base values (e.g. 0.001).

### 2. Define Enums and Structs

In `src/layer1/hazards.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AccidentSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Event, Debug, Clone)]
pub struct AmputationEvent {
    pub entity: Entity,
    // potentially which limb? For MVP just "Limb"
}
```

### 3. Implement Risk Calculation

```rust
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::structure::Structure;

pub fn calculate_risk(
    base_risk: f64,
    skills: &Skills,
    skill_type: SkillType,
    structure: &Structure,
) -> f64 {
    // 1. Maintenance Factor
    // 0% HP = 3.0x risk. 100% HP = 1.0x risk.
    // Formula: 1.0 + (1.0 - (current / max)) * 2.0
    let hp_percent = structure.current_hp / structure.max_hp.max(1.0);
    let maintenance_factor = 1.0 + (1.0 - hp_percent) * 2.0;

    // 2. Skill Factor
    // Level 0 = 1.0x. Level 10 = 2.0x denominator (0.5x risk).
    let level = skills.get_level(skill_type);
    let skill_factor = 1.0 + (level as f64 * 0.1);

    base_risk * maintenance_factor / skill_factor
}
```

### 4. Implement Accident Trigger

```rust
pub fn determine_severity(roll: f32) -> AccidentSeverity {
    if roll < 0.80 {
        AccidentSeverity::Minor
    } else if roll < 0.95 {
        AccidentSeverity::Major
    } else {
        AccidentSeverity::Critical
    }
}

pub fn trigger_accident(world: &mut World, entity: Entity, severity: AccidentSeverity) {
    use crate::layer1::health::Health;

    let damage = match severity {
        AccidentSeverity::Minor => 10.0,
        AccidentSeverity::Major => 40.0,
        AccidentSeverity::Critical => 80.0,
    };

    if let Some(mut health) = world.get_mut::<Health>(entity) {
        health.take_damage(damage);
    }

    if severity == AccidentSeverity::Critical {
        world.send_event(AmputationEvent { entity });
        // Also apply bleeding? (Future)
    }

    // Logging
}
```

### 5. Update `work_execution_system`

Inject the logic into `src/layer1/execution.rs`.

```rust
// In the work loop
let risk = calculate_risk(action.danger_level(), skills, skill_type, structure);
if rng.gen_bool(risk) {
    let severity_roll = rng.gen::<f32>();
    let severity = determine_severity(severity_roll);
    trigger_accident(world, pop_entity, severity);
}
```

### 6. Amputation Handling (Integration with 151)

In `src/layer1/cybernetics.rs` or `hazards.rs`:

```rust
pub fn amputation_listener(
    mut events: EventReader<AmputationEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Add "Missing Limb" trait or component?
        // Or directly remove a "NaturalLimb" item?
        // For MVP: Apply a permanent debuff "Amputee" that reduces efficiency
        // until a Prosthetic (151) is installed.
        commands.entity(event.entity).insert(crate::layer1::traits::Trait::Amputee);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Trait Integration**: `Amputee` trait should be removed when `Augmentations` contains a matching prosthetic.
- **Visuals**: Blood particles on accident site.
- **Safety Gear**: Check for `ItemType::Helmet` in `calculate_risk`.

## Acceptance Criteria

- [ ] High skill reduces accident risk significantly.
- [ ] Low building HP increases accident risk.
- [ ] Critical accidents trigger `AmputationEvent`.
- [ ] Events are logged with severity details.
- [ ] All tests pass.

## Technical Guidance

- Ensure `Structure` is available in `work_execution_system`. It might be on a different entity (the building) than the pop. The designation usually points to the target.
- `SkillType` must match the job (Mining vs Construction).

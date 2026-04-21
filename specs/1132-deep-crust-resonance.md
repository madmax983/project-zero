# 1132: The Deep Crust Resonance

## 1. Overview
Uncovering something ancient and terrifying deep underground that drives the colony mad with paranoia.

Mining too deep can encounter "Resonant Ore." Pops working near it gradually gain severe negative mood debuffs, but the ore is extremely valuable. If not mitigated, the "resonance" spreads to their relationships, causing colony-wide paranoia and violent outbursts.

This mechanic creates tension between exploiting maddening wealth and risking social collapse, or sealing off the depths to keep the colony sane.

## 2. Dependencies
- Layer 1 `Mining` and `Excavation` discovery mechanics.
- Layer 1 `Pop` entity, traits, and mood system.
- Layer 1 `Utility AI` (Work assignment, Task priorities).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::mining::{ExcavationEvent, MineableOre};
    use crate::layer1::pop::{Pop, MoodModifier};
    use crate::layer1::social::Relationship;

    #[test]
    fn test_mining_resonant_ore_inflicts_resonance_debuff() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, resonant_ore_exposure_system);

        let miner = app.world_mut().spawn((Pop, MoodModifier::default())).id();
        let resonant_ore = app.world_mut().spawn((MineableOre, ResonantOre)).id();

        app.world_mut().resource_mut::<Events<ExcavationEvent>>().send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner,
            discovery_type: "ResonantOre".to_string(),
            target: resonant_ore,
        });

        app.update();

        let mood = app.world().get::<MoodModifier>(miner).unwrap();
        assert!(mood.modifiers.iter().any(|m| m.label == "Deep Resonance"), "Miner exposed to resonant ore should gain negative resonance mood modifier.");
        assert!(app.world().get::<ResonantInfection>(miner).is_some(), "Miner should be marked as carrying the resonance.");
    }

    #[test]
    fn test_resonance_spreads_paranoia_through_relationships() {
        let mut app = App::new();
        app.add_systems(Update, resonance_social_spread_system);

        let pop_a = app.world_mut().spawn((Pop, ResonantInfection { severity: 1.0 })).id();
        let pop_b = app.world_mut().spawn((Pop, MoodModifier::default())).id();

        app.world_mut().spawn(Relationship {
            from: pop_a,
            to: pop_b,
            trust: 0.5,
        });

        app.update();

        let mood_b = app.world().get::<MoodModifier>(pop_b).unwrap();
        assert!(mood_b.modifiers.iter().any(|m| m.label == "Paranoia"), "Paranoia should spread to related pops.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/deep_crust_resonance.rs
use bevy::prelude::*;
use crate::layer1::mining::ExcavationEvent;
use crate::layer1::pop::{Pop, MoodModifier, MoodModifierEntry};
use crate::layer1::social::Relationship;

#[derive(Component)]
pub struct ResonantOre;

#[derive(Component)]
pub struct ResonantInfection {
    pub severity: f32,
}

pub fn resonant_ore_exposure_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
    mut q_mood: Query<&mut MoodModifier>,
) {
    for event in events.read() {
        if event.discovery_type == "ResonantOre" {
            if let Ok(mut mood) = q_mood.get_mut(event.miner) {
                if !mood.modifiers.iter().any(|m| m.label == "Deep Resonance") {
                    mood.modifiers.push(MoodModifierEntry {
                        label: "Deep Resonance".to_string(),
                        value: -10.0,
                    });
                }
                commands.entity(event.miner).insert(ResonantInfection { severity: 1.0 });
            }
        }
    }
}

pub fn resonance_social_spread_system(
    q_relationships: Query<&Relationship>,
    q_infected: Query<&ResonantInfection>,
    mut q_mood: Query<&mut MoodModifier>,
) {
    for rel in q_relationships.iter() {
        if q_infected.get(rel.from).is_ok() {
            if let Ok(mut mood) = q_mood.get_mut(rel.to) {
                if !mood.modifiers.iter().any(|m| m.label == "Paranoia") {
                    mood.modifiers.push(MoodModifierEntry {
                        label: "Paranoia".to_string(),
                        value: -5.0,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Mining Depth:** The spawning or discovery of `ResonantOre` should be tied directly to the Z-axis or depth level of the colony excavation.
- **Gradual Spread & Decay:** Infection severity and paranoia spread shouldn't be instant binary applications; they should use DeltaTime and scale based on continuous exposure duration or relationship strength.
- **Violent Outbursts:** Add Utility AI hooks so that when `Paranoia` or `Deep Resonance` stacks too high, the Pop selects violent `ActionType` responses.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `ResonantOre` inflicts `Deep Resonance` and `ResonantInfection` upon excavation
- [ ] `ResonantInfection` spreads `Paranoia` to related Pops

## 7. Technical Guidance
- `ResonantOre` should drop a highly valuable resource in the inventory system to incentivize players despite the risks.
- Ensure that the loop applying persistent MoodModifiers explicitly checks if the modifier exists (`!mood.modifiers.iter().any(...)`) to prevent infinite stacking.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 915: Orbital Debris Cult

## 1. Overview
The trash left in orbit becomes a holy relic for those who live among it. Pops living in low-tier orbital stations begin to venerate uncollected space debris (destroyed ships, old satellites). The more debris in orbit, the higher their morale, but the higher the collision risk for your trade routes. Cleaning up the orbit will lower their morale and could cause an uprising.

## 2. Dependencies
- `layer2::orbit` (OrbitalStation, OrbitalDebris)
- `layer1::pops` (Morale, Faction)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::orbit::{OrbitalStation, OrbitalDebris};
    use crate::layer1::pops::{Pop, Morale};

    #[test]
    fn test_debris_cult_activation() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_debris_cult_formation_system);

        // Spawn a lot of debris
        for _ in 0..10 {
            app.world_mut().spawn(OrbitalDebris);
        }

        // Spawn a low-tier orbital pop
        let entity = app.world_mut().spawn((
            Pop,
            OrbitalStation { tier: 1 },
        )).id();

        app.update();

        // Should gain the DebrisCultist component
        assert!(app.world().get::<DebrisCultist>(entity).is_some());
    }

    #[test]
    fn test_debris_cult_morale_boost() {
        let mut app = App::new();
        app.add_systems(Update, apply_debris_cult_morale_system);

        app.world_mut().spawn(OrbitalDebris);
        app.world_mut().spawn(OrbitalDebris);

        let entity = app.world_mut().spawn((
            Pop,
            DebrisCultist,
            Morale { value: 50.0, threshold: 20.0 },
        )).id();

        app.update();

        // Morale should increase based on the amount of debris (e.g. +5 per debris)
        let morale = app.world().get::<Morale>(entity).unwrap();
        assert_eq!(morale.value, 60.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::orbit::{OrbitalStation, OrbitalDebris};
use crate::layer1::pops::{Pop, Morale};

#[derive(Component)]
pub struct DebrisCultist;

pub fn evaluate_debris_cult_formation_system(
    mut commands: Commands,
    debris_query: Query<&OrbitalDebris>,
    pop_query: Query<(Entity, &OrbitalStation), With<Pop>>,
) {
    let debris_count = debris_query.iter().count();

    // Threshold: 5 pieces of debris
    if debris_count >= 5 {
        for (entity, station) in pop_query.iter() {
            if station.tier <= 1 {
                commands.entity(entity).insert(DebrisCultist);
            }
        }
    } else {
        // Lose cult status if debris is cleaned up
        for (entity, _) in pop_query.iter() {
            commands.entity(entity).remove::<DebrisCultist>();
        }
    }
}

pub fn apply_debris_cult_morale_system(
    debris_query: Query<&OrbitalDebris>,
    mut cultist_query: Query<&mut Morale, With<DebrisCultist>>,
) {
    let debris_count = debris_query.iter().count();
    let bonus = (debris_count as f32) * 5.0; // 5 morale per debris

    for mut morale in cultist_query.iter_mut() {
        morale.value += bonus;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Cap the bonus:** Morale bonuses should probably be capped to prevent infinite scaling.
- **Debris types:** Different types of debris (e.g., destroyed dreadnought vs simple trash) could grant different bonus weights.
- **Cleanup penalty:** Instead of just losing the bonus, cleaning up debris might explicitly trigger an `AddChronicleEvent` or an immediate sharp morale penalty/uprising event.

## 6. Acceptance Criteria
- [ ] TDD Tests written and passing.
- [ ] Test coverage >= 85%.
- [ ] Low-tier orbital pops form the cult when debris count is high.
- [ ] Cultists receive morale bonuses proportional to the debris count.

## 7. Technical Guidance
- Implement in `src/layer2/orbit/debris_cult.rs`.
- Ensure interaction with the `layer3::chronicle` system if an uprising triggers due to debris removal.

## 8. Questions
*Builder: add questions here if spec is unclear.*

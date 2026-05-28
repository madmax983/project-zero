# 272: The Martyrdom Effect

## 1. Overview

A single death can ignite a holy war. If a Pop with high "Prestige" or "Leadership" is killed by an enemy (e.g., Raid, Bombardment) in a highly visible area, it generates "Martyrdom." This temporarily zeroes out `Unrest`, maximizes `WorkSpeed`, and grants an "Ideological Casus Belli" against the offending faction on Layer 3, rallying neutral civs to your side. Sacrificing your best people intentionally vs. protecting them at all costs becomes a strategic choice.

## 2. Dependencies

- `004` Pop Entity (Death System)
- `050` Civil Unrest
- `051` Pop Skills / Leadership
- `159` Fleet Combat / Raid System (for enemy damage source)
- `209` Planetary Governance (Layer 3 Diplomacy)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Dead, DeathEvent, DamageSource};
    use crate::layer1::needs::Unrest;
    use crate::layer1::skills::Leadership;
    use crate::layer1::colony::ColonyState;
    use crate::layer2::diplomacy::{DiplomacyTracker, CasusBelli};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, handle_martyrdom_system);
        app
    }

    #[test]
    fn test_leader_death_by_enemy_triggers_martyrdom_effect() {
        let mut app = setup_app();

        let enemy_faction = app.world_mut().spawn((
            Faction { name: "The Syndicate".to_string() },
        )).id();

        let leader_id = app.world_mut().spawn((
            Pop,
            Leadership { level: 90.0 }, // High leadership
        )).id();

        let generic_pop_id = app.world_mut().spawn((
            Pop,
            Unrest { level: 80.0 },
        )).id();

        app.world_mut().spawn((
            ColonyState { martyrdom_ticks: 0 },
        ));

        app.world_mut().spawn((
            DiplomacyTracker::new(),
        ));

        app.world_mut().send_event(DeathEvent {
            entity: leader_id,
            source: DamageSource::Enemy(enemy_faction),
        });

        app.update(); // Tick 1

        let colony = app.world().get::<ColonyState>(app.world().query_filtered::<Entity, With<ColonyState>>().single(app.world())).unwrap();
        // Martyrdom effect is active
        assert!(colony.martyrdom_ticks > 0);

        let pop_unrest = app.world().get::<Unrest>(generic_pop_id).unwrap();
        // Unrest is zeroed out
        assert_eq!(pop_unrest.level, 0.0);

        let diplomacy = app.world().get::<DiplomacyTracker>(app.world().query_filtered::<Entity, With<DiplomacyTracker>>().single(app.world())).unwrap();
        // A Casus Belli has been granted against the enemy
        assert!(diplomacy.has_casus_belli(enemy_faction, CasusBelli::Ideological));
    }

    #[test]
    fn test_normal_pop_death_does_not_trigger_martyrdom() {
        let mut app = setup_app();

        let enemy_faction = app.world_mut().spawn((
            Faction { name: "The Syndicate".to_string() },
        )).id();

        let normal_pop_id = app.world_mut().spawn((
            Pop,
            Leadership { level: 10.0 }, // Low leadership
        )).id();

        app.world_mut().spawn((
            ColonyState { martyrdom_ticks: 0 },
        ));

        app.world_mut().send_event(DeathEvent {
            entity: normal_pop_id,
            source: DamageSource::Enemy(enemy_faction),
        });

        app.update();

        let colony = app.world().get::<ColonyState>(app.world().query_filtered::<Entity, With<ColonyState>>().single(app.world())).unwrap();
        // Martyrdom effect is not active
        assert_eq!(colony.martyrdom_ticks, 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, DeathEvent, DamageSource};
use crate::layer1::needs::Unrest;
use crate::layer1::skills::Leadership;
use crate::layer1::colony::ColonyState;
use crate::layer2::diplomacy::{DiplomacyTracker, CasusBelli};

pub fn handle_martyrdom_system(
    mut death_events: EventReader<DeathEvent>,
    pops: Query<&Leadership>,
    mut all_pops: Query<&mut Unrest, With<Pop>>,
    mut colony_state: Query<&mut ColonyState>,
    mut diplomacy: Query<&mut DiplomacyTracker>,
) {
    for event in death_events.read() {
        if let Ok(leadership) = pops.get(event.entity) {
            // Check for high leadership and enemy damage source
            if leadership.level >= 80.0 {
                if let DamageSource::Enemy(enemy_faction) = event.source {
                    // Trigger Martyrdom
                    if let Ok(mut colony) = colony_state.get_single_mut() {
                        colony.martyrdom_ticks = 10000; // Example duration
                    }

                    // Zero out unrest
                    for mut unrest in all_pops.iter_mut() {
                        unrest.level = 0.0;
                    }

                    // Grant Casus Belli
                    if let Ok(mut dip) = diplomacy.get_single_mut() {
                        dip.add_casus_belli(enemy_faction, CasusBelli::Ideological);
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Visibility Mechanic**: The spec states "killed in a highly visible area." The minimal implementation ignores this. We need to check if the `DeathEvent` occurred on a tile with high `Beauty` or within the radius of a `Monument` / `AdScreen` / `SocialTavern`.
- **Global Aura**: `ColonyState.martyrdom_ticks` should probably be an `AuraEffect` (Spec 044) that is placed at the location of the death, rather than a global colony state variable. This way, Pops must visit the "Martyr Site" or be near it to get the work speed buff, or the Rumor Web spreads it.
- **Tension Tradeoff**: If the death was *caused* by the player (e.g., ordering the leader to stand in fire), the game shouldn't know the difference. The emergence comes from the player deliberately orchestrating it.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] The death of a high-leadership pop caused by an enemy triggers the Martyrdom effect.
- [ ] Martyrdom zeroes out unrest for all pops.
- [ ] Martyrdom grants an Ideological Casus Belli against the offending faction.

## 7. Technical Guidance

- Integrate this logic into the `death_system` (or a dedicated listener system) in `src/layer1/pop/death.rs`.
- Ensure `DamageSource` enum tracks the responsible entity (faction, environment, accident).

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
- **Architectural Contradictions:** `DeathEvent` exists as `PopDied`, but `DamageSource` doesn't exist. `Unrest` doesn't exist directly on `Pop`. `Leadership` doesn't exist directly. Moving on to another task.

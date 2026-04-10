# 944: Cultural Ransom

## Overview

Waging war not for territory, but for the soul of an enemy civilization. During deep strikes into enemy territory, specialized fleets can steal unique "Cultural Artifacts". Holding these items inflicts severe, compounding diplomatic and morale penalties on the victim faction, and they can be traded back in peace negotiations for staggering concessions.

## Dependencies

- Requires existing Layer 3 Diplomatic and Fleet/Warfare systems.

## 1. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer3::diplomacy::{DiplomaticState, Faction, Morale};

    #[test]
    fn test_steal_cultural_artifact() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        app.add_systems(Update, process_artifact_raid_system);

        let victim_faction = app.world_mut().spawn((
            Faction { id: 1 },
            CulturalArtifact { name: "Original Charter".to_string(), held_by: 1 },
        )).id();

        let raider_faction = app.world_mut().spawn(Faction { id: 2 }).id();

        // Act
        app.world_mut().send_event(RaidEvent {
            target_faction: victim_faction,
            raider_faction: raider_faction,
            successful: true,
            is_deep_strike: true,
        });
        app.update();

        // Assert
        // The artifact's held_by value should now be the raider's faction ID
        let artifact = app.world().get::<CulturalArtifact>(victim_faction).unwrap();
        assert_eq!(artifact.held_by, 2, "Artifact should be stolen by raider.");
    }

    #[test]
    fn test_artifact_hostage_penalties() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_hostage_penalties_system);

        let victim_faction = app.world_mut().spawn((
            Faction { id: 1 },
            Morale { value: 100.0 },
            CulturalArtifact { name: "Original Charter".to_string(), held_by: 2 }, // Held by raider
        )).id();

        // Act
        app.update();

        // Assert
        // Victim's morale should be heavily penalized
        let morale = app.world().get::<Morale>(victim_faction).unwrap();
        assert!(morale.value < 100.0, "Morale should decrease when an artifact is held hostage.");
    }

    #[test]
    fn test_trade_artifact_for_concessions() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DiplomaticNegotiationEvent>();
        app.add_systems(Update, handle_ransom_negotiation_system);

        let victim_faction = app.world_mut().spawn((
            Faction { id: 1 },
            CulturalArtifact { name: "Original Charter".to_string(), held_by: 2 },
            ResourcePool { credits: 1000 },
        )).id();

        let raider_faction = app.world_mut().spawn((
            Faction { id: 2 },
            ResourcePool { credits: 0 },
        )).id();

        // Act
        app.world_mut().send_event(DiplomaticNegotiationEvent {
            proposer: raider_faction,
            target: victim_faction,
            offer_artifact_return: true,
            demand_credits: 500,
        });
        app.update();

        // Assert
        let victim_pool = app.world().get::<ResourcePool>(victim_faction).unwrap();
        let raider_pool = app.world().get::<ResourcePool>(raider_faction).unwrap();
        let artifact = app.world().get::<CulturalArtifact>(victim_faction).unwrap();

        assert_eq!(victim_pool.credits, 500, "Victim should have paid the ransom.");
        assert_eq!(raider_pool.credits, 500, "Raider should have received the ransom.");
        assert_eq!(artifact.held_by, 1, "Artifact should be returned to victim.");
    }
}
```

## 2. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Stub structures for dependencies
#[derive(Component)]
pub struct Faction { pub id: u32 }

#[derive(Component)]
pub struct Morale { pub value: f32 }

#[derive(Component)]
pub struct ResourcePool { pub credits: u32 }

#[derive(Event)]
pub struct RaidEvent {
    pub target_faction: Entity,
    pub raider_faction: Entity,
    pub successful: bool,
    pub is_deep_strike: bool,
}

#[derive(Event)]
pub struct DiplomaticNegotiationEvent {
    pub proposer: Entity,
    pub target: Entity,
    pub offer_artifact_return: bool,
    pub demand_credits: u32,
}

// Feature components
#[derive(Component)]
pub struct CulturalArtifact {
    pub name: String,
    pub held_by: u32, // Faction ID
}

// Systems
pub fn process_artifact_raid_system(
    mut events: EventReader<RaidEvent>,
    mut factions: Query<(&Faction, &mut CulturalArtifact)>,
    raiders: Query<&Faction>,
) {
    for event in events.read() {
        if event.successful && event.is_deep_strike {
            if let Ok(raider_faction) = raiders.get(event.raider_faction) {
                if let Ok((target_faction, mut artifact)) = factions.get_mut(event.target_faction) {
                    if artifact.held_by == target_faction.id {
                        artifact.held_by = raider_faction.id; // Stolen!
                    }
                }
            }
        }
    }
}

pub fn apply_hostage_penalties_system(
    mut factions: Query<(&Faction, &CulturalArtifact, &mut Morale)>,
) {
    for (faction, artifact, mut morale) in factions.iter_mut() {
        if artifact.held_by != faction.id {
            morale.value -= 5.0; // Penalty for lost artifact
        }
    }
}

pub fn handle_ransom_negotiation_system(
    mut events: EventReader<DiplomaticNegotiationEvent>,
    mut factions: Query<(Entity, &Faction, &mut ResourcePool, Option<&mut CulturalArtifact>)>,
) {
    for event in events.read() {
        if event.offer_artifact_return {
            // Very simplified logic: find both factions and transfer resources/artifact
            let mut target_paid = false;
            let mut target_id = 0;

            // First pass: deduct from target and get their ID
            for (entity, faction, mut pool, _) in factions.iter_mut() {
                if entity == event.target && pool.credits >= event.demand_credits {
                    pool.credits -= event.demand_credits;
                    target_paid = true;
                    target_id = faction.id;
                }
            }

            if target_paid {
                // Second pass: add to proposer and return artifact
                for (entity, _, mut pool, artifact_opt) in factions.iter_mut() {
                    if entity == event.proposer {
                        pool.credits += event.demand_credits;
                    }
                    if entity == event.target {
                        if let Some(mut artifact) = artifact_opt {
                            artifact.held_by = target_id;
                        }
                    }
                }
            }
        }
    }
}
```

## 3. REFACTOR Phase: Quality & Design

- The `handle_ransom_negotiation_system` is currently doing a double pass which is inefficient; a better design would use `get_many_mut` or structure the event payload to directly contain the necessary faction IDs.
- Expanding negotiations to cover territory (planets) rather than just credits.
- Introduce compounding diplomatic penalties over time rather than a flat morale decrease per tick.

## 4. Acceptance Criteria (Testable)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Deep strikes can steal cultural artifacts.
- [ ] Factions suffer penalties when their artifacts are held by enemies.
- [ ] Artifacts can be traded back in negotiations.

## 5. Technical Guidance

- Implement in `src/layer3/diplomacy/ransom.rs` or similar.
- Consider utilizing Bevy's relational components if factions and artifacts have a complex ownership structure in Layer 3.

## 6. Questions

*Builder: add questions here if spec is unclear.*

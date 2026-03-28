# Specification: The Orphanage of Stars

## 1. Overview
**Layer:** Cross-layer (3 -> 1)
**Title:** The Orphanage of Stars
**Description:** Post-war refugee ships can drop off "Orphans" (children with no parents). They consume resources without working until adulthood. Once adults, they gain a massive "Indebted Loyalty" buff. Later, their original factions might demand their return, offering bounties or threatening war.

## 2. Dependencies
- `Pop` and Needs system (Layer 1)
- Age/Maturation system for Pops (Layer 1)
- Faction and Diplomacy system (Layer 3)
- Event/Chronicle system for diplomacy demands

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orphan_maturation_grants_loyalty() {
        let mut app = App::new();
        app.add_systems(Update, process_orphan_maturation);

        // Arrange
        let orphan_entity = app.world_mut().spawn((
            Pop,
            Age { current: 17, threshold: 18 },
            Orphan { original_faction_id: 42 },
        )).id();

        // Act
        // Simulate time passing to age the orphan
        app.world_mut().entity_mut(orphan_entity).get_mut::<Age>().unwrap().current = 18;
        app.update();

        // Assert
        let pop = app.world().entity(orphan_entity);
        assert!(!pop.contains::<Orphan>(), "Orphan component should be removed upon adulthood");
        assert!(pop.contains::<IndebtedLoyalty>(), "Adult orphans should gain Indebted Loyalty");
        assert_eq!(
            pop.get::<IndebtedLoyalty>().unwrap().original_faction_id,
            42,
            "Loyalty should remember the original faction"
        );
    }

    #[test]
    fn test_faction_demands_loyal_orphans() {
        let mut app = App::new();
        app.add_event::<FactionDemandEvent>();
        app.add_systems(Update, evaluate_orphan_repatriation_demands);

        // Arrange
        let adult_orphan = app.world_mut().spawn((
            Pop,
            IndebtedLoyalty { original_faction_id: 99 },
        )).id();

        // Act
        // Manually trigger a check for faction 99 looking for their lost citizens
        app.world_mut().insert_resource(RepatriationCheckTimer(Timer::from_seconds(0.0, TimerMode::Once)));
        app.update();

        // Assert
        let events = app.world().resource::<Events<FactionDemandEvent>>();
        let mut reader = events.get_cursor();
        let demands: Vec<_> = reader.read(events).collect();

        assert_eq!(demands.len(), 1, "A faction demand event should be fired");
        assert_eq!(demands[0].faction_id, 99);
        assert_eq!(demands[0].target_pops.len(), 1);
        assert!(demands[0].target_pops.contains(&adult_orphan));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Age {
    pub current: u32,
    pub threshold: u32,
}

#[derive(Component)]
pub struct Orphan {
    pub original_faction_id: u32,
}

#[derive(Component)]
pub struct IndebtedLoyalty {
    pub original_faction_id: u32,
}

#[derive(Event)]
pub struct FactionDemandEvent {
    pub faction_id: u32,
    pub target_pops: Vec<Entity>,
}

#[derive(Resource)]
pub struct RepatriationCheckTimer(pub Timer);

pub fn process_orphan_maturation(
    mut commands: Commands,
    query: Query<(Entity, &Age, &Orphan)>,
) {
    for (entity, age, orphan) in query.iter() {
        if age.current >= age.threshold {
            commands.entity(entity).remove::<Orphan>();
            commands.entity(entity).insert(IndebtedLoyalty {
                original_faction_id: orphan.original_faction_id,
            });
        }
    }
}

pub fn evaluate_orphan_repatriation_demands(
    mut timer: Option<ResMut<RepatriationCheckTimer>>,
    query: Query<(Entity, &IndebtedLoyalty)>,
    mut demand_events: EventWriter<FactionDemandEvent>,
) {
    if let Some(mut t) = timer {
        if t.0.finished() {
            // Group loyal orphans by their original faction
            let mut faction_groups: std::collections::HashMap<u32, Vec<Entity>> = std::collections::HashMap::new();

            for (entity, loyalty) in query.iter() {
                faction_groups.entry(loyalty.original_faction_id).or_default().push(entity);
            }

            for (faction_id, target_pops) in faction_groups {
                demand_events.send(FactionDemandEvent {
                    faction_id,
                    target_pops,
                });
            }

            // Reset timer (for testing, we just remove it or let it be)
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** The `IndebtedLoyalty` trait should explicitly negate the effects of negative morale modifiers or drastically reduce the chance of the Pop engaging in strikes or riots.
- **Performance:** `evaluate_orphan_repatriation_demands` should not run every frame. It should be triggered periodically (e.g., via a daily or weekly tick).
- **Design:** Faction demands should tie into the Layer 3 diplomacy system, presenting the player with a choice: surrender the pops for a relationship boost/bounty, or refuse and incur a massive relationship penalty/war declaration.

## 6. Acceptance Criteria (Testable!)
- [ ] `test_orphan_maturation_grants_loyalty` passes.
- [ ] `test_faction_demands_loyal_orphans` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- **Components:** Create the `Orphan` and `IndebtedLoyalty` components in `src/layer1/social/traits.rs` or a dedicated `src/layer1/social/orphan.rs` module.
- **Diplomacy Bridge:** The `FactionDemandEvent` should be handled by a system in `src/layer3/diplomacy.rs` which queues up an interactive chronicle/event for the player to resolve.
- **Consumption:** Ensure that Pops with the `Orphan` component consume food and housing but cannot be assigned to Jobs.

## 8. Questions
*Builder: add questions here if spec is unclear.*

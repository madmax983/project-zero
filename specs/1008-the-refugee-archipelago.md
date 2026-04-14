# 1008: The Refugee Archipelago

## 1. Overview
When a Layer 1 colony fails due to starvation, unrest, or disaster, the survivors launch in whatever jury-rigged ships they can find. These "Refugee Flotillas" roam the Layer 2 system, slowly depleting their own fuel and food. They will dock at functional colonies demanding asylum, instantly dumping massive populations and extreme needs onto infrastructure. If denied, they turn into desperate pirates or strip-mine orbital resources.

## 2. Dependencies
- Layer 1 `Colony` collapse/failure conditions.
- Layer 2 `Fleet` entities (creating a special `RefugeeFlotilla` type).
- Layer 2 `Navigation` to move flotillas towards other colonies.
- Layer 1 `Diplomacy` (Asylum requests) and `Population` transfer mechanics.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::colony::{Colony, CollapseEvent};
    use crate::layer2::fleet::{Fleet, RefugeeFlotilla, Starving};
    use crate::layer1::diplomacy::{AsylumRequestEvent, Faction};

    #[test]
    fn test_colony_collapse_spawns_refugee_flotilla() {
        let mut app = App::new();
        app.add_event::<CollapseEvent>();
        app.add_systems(Update, spawn_refugees_on_collapse_system);

        let collapsed_colony = app.world_mut().spawn((
            Colony,
            Faction::Independent,
        )).id();

        app.world_mut().resource_mut::<Events<CollapseEvent>>().send(CollapseEvent {
            colony: collapsed_colony,
            survivor_count: 5000,
        });

        app.update();

        // Verify Flotilla spawned
        let mut flotilla_query = app.world_mut().query::<(&Fleet, &RefugeeFlotilla)>();
        let mut found = false;
        for (_fleet, flotilla) in flotilla_query.iter(app.world()) {
            if flotilla.population == 5000 {
                found = true;
            }
        }

        assert!(found, "A refugee flotilla should spawn with the survivor population upon colony collapse.");
    }

    #[test]
    fn test_refugee_flotilla_requests_asylum_at_colony() {
        let mut app = App::new();
        app.add_event::<AsylumRequestEvent>();
        app.add_systems(Update, refugee_arrival_system);

        let target_colony = app.world_mut().spawn((
            Colony,
            Faction::Player,
        )).id();

        // Spawn a flotilla that has 'arrived' at the colony
        let flotilla = app.world_mut().spawn((
            Fleet,
            RefugeeFlotilla { population: 2000, target_colony: Some(target_colony) },
        )).id();

        app.update();

        // Verify AsylumRequest event fired
        let asylum_events = app.world().resource::<Events<AsylumRequestEvent>>();
        let mut reader = asylum_events.get_reader();
        let mut found = false;
        for event in reader.read(asylum_events) {
            if event.flotilla == flotilla && event.target == target_colony {
                found = true;
            }
        }

        assert!(found, "Refugee flotilla arriving at a colony should trigger an Asylum Request event.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/refugee_archipelago.rs
use bevy::prelude::*;
use crate::layer1::colony::{Colony, CollapseEvent};
use crate::layer2::fleet::Fleet;
use crate::layer1::diplomacy::AsylumRequestEvent;

#[derive(Component)]
pub struct RefugeeFlotilla {
    pub population: u32,
    pub target_colony: Option<Entity>,
}

pub fn spawn_refugees_on_collapse_system(
    mut commands: Commands,
    mut collapse_events: EventReader<CollapseEvent>,
) {
    for event in collapse_events.read() {
        if event.survivor_count > 0 {
            commands.spawn((
                Fleet,
                RefugeeFlotilla {
                    population: event.survivor_count,
                    target_colony: None, // Will be set by navigation AI later
                },
            ));
        }
    }
}

pub fn refugee_arrival_system(
    query: Query<(Entity, &RefugeeFlotilla)>,
    mut asylum_events: EventWriter<AsylumRequestEvent>,
) {
    for (entity, flotilla) in query.iter() {
        // If they have a target, assume they've arrived for MVP
        if let Some(target) = flotilla.target_colony {
            asylum_events.send(AsylumRequestEvent {
                flotilla: entity,
                target,
                population: flotilla.population,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Navigation AI:** The flotillas need an AI system to actually find the nearest functioning colony in Layer 2 space and move towards it.
- **Resource Depletion:** Flotillas should lose population over time as they starve in transit.
- **Piracy:** If `AsylumRequestEvent` is rejected (needs a response system), the `RefugeeFlotilla` component should be swapped with a `PirateFleet` component, making them hostile.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_colony_collapse_spawns_refugee_flotilla` passes.
- [ ] Test `test_refugee_flotilla_requests_asylum_at_colony` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- Integration with Layer 1 `CollapseEvent` is key. Ensure this fires when unrest or starvation reaches the defined terminal threshold.
- Accepting asylum should instantly create `Pop` entities in Layer 1 with extremely high `Hunger` and `Stress` values to reflect their journey.

## 8. Questions
*Builder: add questions here if spec is unclear.*

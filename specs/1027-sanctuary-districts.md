# 1027: Sanctuary Districts

## 1. Overview
Sanctuary Districts are designated zones where "Police" or automated security cannot enter to make arrests. "Wanted" Pops flock to these zones. While this causes localized spikes in Crime and Vice, it keeps criminals contained and productive via a black market. This introduces emergence where players must venture into these dangerous zones to hire specialized rogue talent, bypassing their own laws.

## 2. Dependencies
- Layer 1 `TerrainGrid`/`Zone` system.
- Layer 1 `Pop` entity (`Traits`, `WantedStatus`).
- Layer 1 `Utility AI` (Police logic, pathfinding restriction).
- Layer 1 `Economy` (Black Market generation).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::terrain::{Zone, ZoneType};
    use crate::layer1::pop::{Pop, WantedStatus, CurrentZone};
    use crate::layer1::utility_ai::{ActionType, TargetEntity};
    use crate::layer1::security::PoliceComponent;

    #[test]
    fn test_wanted_pops_migrate_to_sanctuary_zones() {
        let mut app = App::new();
        app.add_systems(Update, wanted_pop_migration_scoring_system);

        let sanctuary_zone = app.world_mut().spawn((Zone { id: 1 }, ZoneType::Sanctuary)).id();
        let normal_zone = app.world_mut().spawn((Zone { id: 2 }, ZoneType::Residential)).id();

        let wanted_pop = app.world_mut().spawn((
            Pop,
            WantedStatus { active: true },
            CurrentZone { zone: normal_zone },
        )).id();

        app.update();

        // In a real Utility AI, this would evaluate a score. We'll simulate a move order being generated.
        let mut found_move = false;
        let mut query = app.world_mut().query::<(&TargetEntity, &ActionType)>();
        for (target, action) in query.iter(app.world()) {
            if *action == ActionType::MoveToZone && target.entity == Some(sanctuary_zone) {
                found_move = true;
                break;
            }
        }

        assert!(found_move, "Wanted Pops should generate high-priority move actions towards Sanctuary zones.");
    }

    #[test]
    fn test_police_cannot_target_wanted_pops_in_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, filter_police_targets_system);

        let sanctuary_zone = app.world_mut().spawn((Zone { id: 1 }, ZoneType::Sanctuary)).id();
        let safe_wanted_pop = app.world_mut().spawn((
            Pop,
            WantedStatus { active: true },
            CurrentZone { zone: sanctuary_zone },
        )).id();

        let police = app.world_mut().spawn((
            Pop,
            PoliceComponent,
            TargetEntity { entity: Some(safe_wanted_pop) },
            ActionType::Arrest,
        )).id();

        app.update();

        // System should clear the target because they are in a sanctuary
        let target = app.world().get::<TargetEntity>(police).unwrap();
        assert!(target.entity.is_none(), "Police should not be able to target or arrest Pops inside a Sanctuary Zone.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/sanctuary_districts.rs
use bevy::prelude::*;
use crate::layer1::terrain::{Zone, ZoneType};
use crate::layer1::pop::{Pop, WantedStatus, CurrentZone};
use crate::layer1::utility_ai::{ActionType, TargetEntity};
use crate::layer1::security::PoliceComponent;

pub fn wanted_pop_migration_scoring_system(
    mut commands: Commands,
    wanted_query: Query<(Entity, &CurrentZone, &WantedStatus), With<Pop>>,
    zone_query: Query<(Entity, &ZoneType), With<Zone>>,
) {
    let mut sanctuary_entity = None;
    for (entity, zone_type) in zone_query.iter() {
        if *zone_type == ZoneType::Sanctuary {
            sanctuary_entity = Some(entity);
            break;
        }
    }

    if let Some(sanctuary) = sanctuary_entity {
        for (pop_entity, current_zone, status) in wanted_query.iter() {
            if status.active && current_zone.zone != sanctuary {
                // MVP: Force a move command
                commands.spawn((
                    TargetEntity { entity: Some(sanctuary) },
                    ActionType::MoveToZone,
                    // Typically this would attach to the pop or be a WorkOrder
                ));
            }
        }
    }
}

pub fn filter_police_targets_system(
    mut police_query: Query<&mut TargetEntity, With<PoliceComponent>>,
    target_query: Query<&CurrentZone>,
    zone_query: Query<&ZoneType>,
) {
    for mut target in police_query.iter_mut() {
        if let Some(target_ent) = target.entity {
            if let Ok(current_zone) = target_query.get(target_ent) {
                if let Ok(zone_type) = zone_query.get(current_zone.zone) {
                    if *zone_type == ZoneType::Sanctuary {
                        // Invalid target, clear it
                        target.entity = None;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding Blocks:** The `filter_police_targets_system` only prevents *targeting*. The A* pathfinding system needs to treat Sanctuary zone borders as impassable walls for entities with `PoliceComponent` to prevent them from just walking through.
- **Black Market Economy:** Add a system that calculates the total number of `WantedStatus` pops in a Sanctuary zone and generates illegal goods/wealth proportional to their population.
- **Utility Scoring:** The migration system forces a command instead of feeding into the Utility AI. Wanted pops should evaluate "Flee to Sanctuary" against other immediate needs like "Eat" or "Sleep".

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_wanted_pops_migrate_to_sanctuary_zones` passes.
- [ ] Test `test_police_cannot_target_wanted_pops_in_sanctuary` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `ZoneType::Sanctuary` must be added to the zone configuration enums.
- Be cautious of "Flee" logic creating ping-ponging behavior if a wanted pop is hungry but the only food is outside the sanctuary.

## 8. Questions
*Builder: add questions here if spec is unclear.*

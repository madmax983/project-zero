# 974: Leviathan Tapping

## Overview

Harvesting energy from a creature so large it has its own gravity well. Spaceborn "Leviathans" occasionally drift through star systems. Instead of fighting them, you can launch specialized "Tether Harvesters" that latch onto their armored hide. These tethers siphon massive amounts of exotic energy, providing a huge boost to system-wide power, as long as the Leviathan remains in the system. However, when the Leviathan eventually leaves the system by entering FTL, the unbreakable tethers drag any connected orbital structures (like shipyards) out of orbit and across the galaxy, losing them forever.

## Dependencies

- None (Base Layer 2 Simulation)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tether_siphons_energy_from_leviathan() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(SystemPower { current: 100, max: 1000 });
        app.add_systems(Update, process_leviathan_siphoning_system);

        let leviathan = app.world_mut().spawn(SpaceLeviathan { energy_reserves: 5000 }).id();
        app.world_mut().spawn(TetherHarvester { attached_to: Some(leviathan), siphon_rate: 50 });

        // Act
        app.update();

        // Assert
        let power = app.world().resource::<SystemPower>();
        assert_eq!(power.current, 150, "Tether should siphon energy into SystemPower.");

        let lev_data = app.world().get::<SpaceLeviathan>(leviathan).unwrap();
        assert_eq!(lev_data.energy_reserves, 4950, "Leviathan should lose siphoned energy.");
    }

    #[test]
    fn test_leviathan_departure_destroys_tethered_structures() {
        // Arrange
        let mut app = App::new();
        app.add_event::<LeviathanDepartEvent>();
        app.add_systems(Update, handle_leviathan_departure_system);

        let leviathan = app.world_mut().spawn(SpaceLeviathan { energy_reserves: 1000 }).id();
        let shipyard = app.world_mut().spawn((
            OrbitalStructure,
            TetherHarvester { attached_to: Some(leviathan), siphon_rate: 50 }
        )).id();

        // Act: Trigger Departure
        app.world_mut().send_event(LeviathanDepartEvent { leviathan_entity: leviathan });
        app.update();

        // Assert
        assert!(app.world().get_entity(shipyard).is_err(), "Tethered structure should be destroyed on departure.");
    }

    #[test]
    fn test_tether_disconnect_prevents_destruction() {
         // Arrange
         let mut app = App::new();
         app.add_event::<LeviathanDepartEvent>();
         app.add_systems(Update, handle_leviathan_departure_system);

         let leviathan = app.world_mut().spawn(SpaceLeviathan { energy_reserves: 1000 }).id();
         let shipyard = app.world_mut().spawn((
             OrbitalStructure,
             TetherHarvester { attached_to: None, siphon_rate: 50 } // Not attached
         )).id();

         // Act: Trigger Departure
         app.world_mut().send_event(LeviathanDepartEvent { leviathan_entity: leviathan });
         app.update();

         // Assert
         assert!(app.world().get_entity(shipyard).is_ok(), "Disconnected structure should survive departure.");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct SystemPower {
    pub current: u32,
    pub max: u32,
}

#[derive(Component)]
pub struct SpaceLeviathan {
    pub energy_reserves: u32,
}

#[derive(Component)]
pub struct TetherHarvester {
    pub attached_to: Option<Entity>,
    pub siphon_rate: u32,
}

#[derive(Component)]
pub struct OrbitalStructure;

#[derive(Event)]
pub struct LeviathanDepartEvent {
    pub leviathan_entity: Entity,
}

pub fn process_leviathan_siphoning_system(
    mut power: ResMut<SystemPower>,
    mut leviathan_query: Query<&mut SpaceLeviathan>,
    tether_query: Query<&TetherHarvester>,
) {
    for tether in tether_query.iter() {
        if let Some(target) = tether.attached_to {
            if let Ok(mut leviathan) = leviathan_query.get_mut(target) {
                let siphoned = tether.siphon_rate.min(leviathan.energy_reserves);
                leviathan.energy_reserves -= siphoned;
                power.current = (power.current + siphoned).min(power.max);
            }
        }
    }
}

pub fn handle_leviathan_departure_system(
    mut commands: Commands,
    mut depart_events: EventReader<LeviathanDepartEvent>,
    tether_query: Query<(Entity, &TetherHarvester)>,
) {
    for event in depart_events.read() {
        for (entity, tether) in tether_query.iter() {
            if tether.attached_to == Some(event.leviathan_entity) {
                commands.entity(entity).despawn_recursive();
                // Alternatively, could spawn a "DraggedIntoFTL" component or event
            }
        }

        // Despawn the leviathan itself
        commands.entity(event.leviathan_entity).despawn_recursive();
    }
}
```

## REFACTOR Phase: Quality & Design

- `handle_leviathan_departure_system` directly despawns entities. This should probably fire a `StructureLostEvent` or `ChronicleEvent` to notify the player that their shipyard was dragged into hyperspace, rather than just silently deleting it.
- Siphoning logic might need to scale with `Time` if the tick rate isn't fixed, to avoid frame-rate dependent resource generation.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/leviathan.rs` (or equivalent module).

## Technical Guidance

- Ensure `LeviathanDepartEvent` is registered in `app.add_event()`.
- Consider placing this in a new file `src/layer2/leviathan.rs` and registering the systems in the appropriate Layer 2 simulation schedule.

## Questions

*Builder: add questions here if spec is unclear.*

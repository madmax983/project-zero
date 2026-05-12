//! The Hidden Harbors.
//!
//! When colony authority weakens, the black market finds physical space to operate.
//! Smuggler's coves spontaneously spawn in unmonitored map tiles, offering illegal
//! trades until they naturally decay or are busted by enforcers.

use crate::layer1::map::GridPosition;
use crate::layer1::TerrainGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Measures the strictness and enforcement level of the current administration.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::economy::smugglers_cove::ColonyAuthority;
///
/// let authority = ColonyAuthority { level: 25.0 };
/// assert!(authority.level < 30.0); // Ripe for smuggling
/// ```
#[derive(Resource, Default)]
pub struct ColonyAuthority {
    pub level: f32, // 0.0 to 100.0
}

/// A physical location representing an illegal black market hub.
///
/// ## Examples
///
/// ```rust
/// use scale::layer1::economy::smugglers_cove::SmugglersCove;
///
/// let mut cove = SmugglersCove { lifespan: 100 };
/// cove.lifespan = cove.lifespan.saturating_sub(1);
/// assert_eq!(cove.lifespan, 99);
/// ```
#[derive(Component)]
pub struct SmugglersCove {
    pub lifespan: u32,
}

/// Spawns a [`SmugglersCove`] if colony authority is sufficiently low.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::economy::smugglers_cove::{spawn_smugglers_cove_system, ColonyAuthority, SmugglersCove};
/// use scale::layer1::nature::terrain::generate_terrain;
///
/// let mut world = World::new();
/// world.insert_resource(ColonyAuthority { level: 10.0 }); // Low authority
/// world.insert_resource(generate_terrain(10, 10)); // Provide terrain
///
/// // In a real game, RNG decides spawning. Here we just show setup.
/// let mut schedule = Schedule::default();
/// schedule.add_systems(spawn_smugglers_cove_system);
/// schedule.run(&mut world);
/// ```
pub fn spawn_smugglers_cove_system(
    mut commands: Commands,
    authority: Res<ColonyAuthority>,
    grid: Res<TerrainGrid>,
) {
    // Only spawn if authority is low
    if authority.level > 30.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Small chance to spawn a cove each tick
    if rng.gen::<f32>() < 0.02 {
        // Pick a random unobserved location (simplified logic)
        let x = rng.gen_range(0..grid.width);
        let y = rng.gen_range(0..grid.height);

        commands.spawn((
            SmugglersCove { lifespan: 100 }, // Cove lasts for 100 ticks
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
        ));
    }
}

/// Slowly decays the lifespan of active coves, despawning them when they reach zero.
///
/// ## Examples
///
/// ```rust
/// use bevy_ecs::prelude::*;
/// use scale::layer1::economy::smugglers_cove::{process_smuggler_decay_system, SmugglersCove};
///
/// let mut world = World::new();
/// let cove = world.spawn(SmugglersCove { lifespan: 1 }).id();
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(process_smuggler_decay_system);
///
/// schedule.run(&mut world); // Lifespan drops to 0, entity despawns
/// assert!(world.get::<SmugglersCove>(cove).is_none());
/// ```
pub fn process_smuggler_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SmugglersCove)>,
) {
    for (entity, mut cove) in query.iter_mut() {
        cove.lifespan = cove.lifespan.saturating_sub(1);
        if cove.lifespan == 0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::time::SimulationTime;
    use bevy::prelude::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.insert_resource(ColonyAuthority { level: 20.0 }); // Low authority
        app.insert_resource(crate::layer1::nature::terrain::generate_terrain(100, 100)); // 100x100 map

        // Add minimal required systems
        app.add_systems(Update, spawn_smugglers_cove_system);

        app
    }

    #[test]
    fn test_smuggler_cove_spawns_in_low_authority() {
        let mut app = setup_app();
        app.add_systems(Update, spawn_smugglers_cove_system);

        // Force deterministic spawn by setting rng probability check to 1.0
        // in test environment, or run enough ticks to guarantee it.
        // For simplicity in this test, we run for 1000 ticks to make failure statistically impossible.
        for _ in 0..1000 {
            app.update();
        }

        // Verify a smuggler's cove has spawned
        let mut cove_query = app.world_mut().query::<&SmugglersCove>();
        assert!(
            cove_query.iter(app.world()).count() > 0,
            "A smuggler's cove should spawn when authority is low"
        );
    }

    #[test]
    fn test_smuggler_cove_does_not_spawn_in_high_authority() {
        let mut app = setup_app();
        app.add_systems(Update, spawn_smugglers_cove_system);

        // High authority should prevent smuggler coves from spawning
        app.world_mut().resource_mut::<ColonyAuthority>().level = 90.0;

        for _ in 0..100 {
            app.update();
        }

        let mut cove_query = app.world_mut().query::<&SmugglersCove>();
        assert_eq!(
            cove_query.iter(app.world()).count(),
            0,
            "A smuggler's cove should not spawn when authority is high"
        );
    }

    #[test]
    fn test_smuggler_cove_decays_over_time() {
        let mut app = setup_app();
        app.add_systems(Update, spawn_smugglers_cove_system);
        app.add_systems(Update, process_smuggler_decay_system);

        // Manually spawn a cove
        let cove_entity = app
            .world_mut()
            .spawn((SmugglersCove { lifespan: 5 }, GridPosition { x: 50, y: 50 }))
            .id();

        // Advance time and check lifespan
        for _ in 0..6 {
            app.update();
        }

        // The cove should be despawned after its lifespan expires
        assert!(
            app.world().get_entity(cove_entity).is_err(),
            "Cove should despawn after lifespan"
        );
    }
}

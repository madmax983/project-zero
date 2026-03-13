use std::collections::{HashMap, HashSet};
use crate::layer1::energy::load_limits::PowerCable;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::WeatherState;
use bevy_ecs::prelude::*;

/// Type of shadow fauna entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowType {
    /// Consumes power.
    StaticMite,
    /// Corrupts data.
    DataRot,
    /// Passive high-beauty entity.
    VoidEel,
}

/// Shadow Ecosystems entity spawned in high energy or data zones.
#[derive(Component)]
pub struct ShadowEntity {
    /// Sub-type of the shadow entity.
    pub entity_type: ShadowType,
    /// Amount of hunger accumulated.
    pub hunger: f32,
    /// Whether the entity is currently visible.
    pub visible: bool,
}

/// Spawns `ShadowEntity` (e.g. `StaticMite`) on tiles where a `PowerCable` has very high load.
pub fn spawn_shadow_fauna_system(
    mut commands: Commands,
    cables: Query<(&PowerCable, &GridPosition)>,
    existing: Query<&GridPosition, With<ShadowEntity>>,
) {
    let mut spawned_this_frame = HashSet::new();

    for (cable, pos) in cables.iter() {
        if cable.current_load > 100.0 {
            // Check if there is already a shadow entity here, and avoid spawning duplicates per tick
            let has_shadow = existing
                .iter()
                .any(|e_pos| e_pos.x == pos.x && e_pos.y == pos.y)
                || spawned_this_frame.contains(&(pos.x, pos.y));
            if !has_shadow {
                spawned_this_frame.insert((pos.x, pos.y));
                commands.spawn((
                    ShadowEntity {
                        entity_type: ShadowType::StaticMite,
                        hunger: 0.0,
                        visible: false,
                    },
                    *pos,
                ));
            }
        }
    }
}

/// Updates visibility of `ShadowEntity` based on `WeatherState` (visible during `MagneticStorm`).
pub fn shadow_visibility_system(
    mut query: Query<&mut ShadowEntity>,
    weather: Option<Res<WeatherState>>,
) {
    let is_magnetic_storm = weather.is_some_and(|w| {
        w.current_weather == crate::layer1::nature::weather::WeatherType::MagneticStorm
    });

    for mut shadow in query.iter_mut() {
        shadow.visible = is_magnetic_storm;
    }
}

/// Handles feeding logic for `ShadowEntity` (`StaticMite`), increasing demand on `PowerConsumer`.
pub fn shadow_feed_system(
    mut mites: Query<(&mut ShadowEntity, &GridPosition)>,
    mut consumers: Query<(Entity, &mut PowerConsumer, &GridPosition)>,
) {
    // Build lookup for consumers by position to avoid O(N*M) iterations
    let mut consumers_by_pos: HashMap<(i32, i32), Vec<Entity>> = HashMap::new();
    for (entity, _consumer, c_pos) in consumers.iter() {
        consumers_by_pos.entry((c_pos.x, c_pos.y)).or_default().push(entity);
    }

    for (mut mite, mite_pos) in mites.iter_mut() {
        if mite.entity_type == ShadowType::StaticMite {
            mite.hunger += 1.0;

            if let Some(consumer_entities) = consumers_by_pos.get(&(mite_pos.x, mite_pos.y)) {
                for &entity in consumer_entities {
                    if let Ok((_, mut consumer, _)) = consumers.get_mut(entity) {
                        consumer.demand += 5.0; // Consume power to feed
                        mite.hunger -= 10.0;
                        mite.hunger = mite.hunger.max(0.0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::energy::load_limits::PowerCable;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::fauna::shadow_ecosystems::{
        shadow_feed_system, shadow_visibility_system, spawn_shadow_fauna_system, ShadowEntity,
        ShadowType,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::weather::{WeatherState, WeatherType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_high_energy_spawns_static_mites() {
        let mut world = World::new();
        // Setup high energy cable cell at (5,5)
        world.spawn((
            PowerCable {
                capacity: 100.0,
                current_load: 105.0, // High load > 100.0
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run spawn system
        let _ = world.run_system_once(spawn_shadow_fauna_system);

        // Check for entity
        let mut query = world.query::<(&ShadowEntity, &GridPosition)>();
        let found = query
            .iter(&world)
            .any(|(e, pos)| e.entity_type == ShadowType::StaticMite && pos.x == 5 && pos.y == 5);
        assert!(found);
    }

    #[test]
    fn test_shadow_feed_reduces_power_efficiency() {
        let mut world = World::new();

        world.spawn((
            ShadowEntity {
                entity_type: ShadowType::StaticMite,
                hunger: 50.0,
                visible: false,
            },
            GridPosition { x: 5, y: 5 },
        ));

        let consumer = world
            .spawn((
                PowerConsumer {
                    demand: 100.0,
                    active: true,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let _ = world.run_system_once(shadow_feed_system);

        let c = world.get::<PowerConsumer>(consumer).unwrap();
        // The Mite should increase the local demand
        assert!(c.demand > 100.0);
    }

    #[test]
    fn test_visibility_toggle() {
        let mut world = World::new();

        let e = world
            .spawn(ShadowEntity {
                entity_type: ShadowType::StaticMite,
                hunger: 0.0,
                visible: false,
            })
            .id();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        let _ = world.run_system_once(shadow_visibility_system);

        let shadow = world.get::<ShadowEntity>(e).unwrap();
        assert!(shadow.visible);
    }
}

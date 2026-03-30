use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Represents the type of shadow fauna.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowType {
    /// Feeds on power grid energy, reducing output efficiency.
    StaticMite,
    /// Corrupts data, causing research or blueprint loss.
    DataRot,
    /// Passive entity, mostly for visual flavor.
    VoidEel,
}

/// A shadow fauna entity that usually spawns around high-tech infrastructure.
#[derive(Component)]
pub struct ShadowEntity {
    /// The specific type of this shadow fauna.
    pub entity_type: ShadowType,
    /// Is the shadow currently visible to the player?
    pub visible: bool,
}

/// Spawns `ShadowEntity` (e.g. `StaticMite`) around high power output sources.
pub fn spawn_shadow_fauna_system(
    mut commands: Commands,
    query: Query<(&GridPosition, &PowerSource)>,
) {
    let mut rng = rand::thread_rng();

    for (pos, source) in query.iter() {
        if source.output >= 1000.0 && rng.gen_bool(0.1) {
            commands.spawn((
                ShadowEntity {
                    entity_type: ShadowType::StaticMite,
                    visible: false,
                },
                *pos,
            ));
        }
    }
}

/// Reduces the power output of sources that share a tile with a `StaticMite`.
pub fn shadow_feed_system(
    shadow_query: Query<(&ShadowEntity, &GridPosition)>,
    mut power_query: Query<(&mut PowerSource, &GridPosition)>,
) {
    for (shadow, shadow_pos) in shadow_query.iter() {
        if shadow.entity_type == ShadowType::StaticMite {
            for (mut source, source_pos) in power_query.iter_mut() {
                if shadow_pos.x == source_pos.x && shadow_pos.y == source_pos.y {
                    source.output *= 0.9; // Reduce efficiency
                }
            }
        }
    }
}

pub fn shadow_visibility_system(mut query: Query<&mut ShadowEntity>, weather: Res<WeatherState>) {
    let is_magnetic_storm = weather.current_weather == WeatherType::MagneticStorm;

    for mut shadow in query.iter_mut() {
        // Simplified: visible during magnetic storms
        shadow.visible = is_magnetic_storm;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_energy_spawns_static_mites() {
        let mut world = World::new();

        world.spawn((
            GridPosition { x: 5, y: 5 },
            PowerSource {
                output: 1500.0,
                active: true,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_shadow_fauna_system);

        for _ in 0..100 {
            schedule.run(&mut world);
        }

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
            GridPosition { x: 5, y: 5 },
            PowerSource {
                output: 1000.0,
                active: true,
            },
        ));

        world.spawn((
            GridPosition { x: 5, y: 5 },
            ShadowEntity {
                entity_type: ShadowType::StaticMite,
                visible: false,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(shadow_feed_system);
        schedule.run(&mut world);

        let mut query = world.query::<&PowerSource>();
        let source = query.single(&world);
        assert_eq!(source.output, 900.0); // 1000 * 0.9
    }

    #[test]
    fn test_visibility_toggle() {
        let mut world = World::new();

        let entity = world
            .spawn((ShadowEntity {
                entity_type: ShadowType::StaticMite,
                visible: false,
            },))
            .id();

        world.insert_resource(WeatherState {
            is_extreme: false,
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(shadow_visibility_system);
        schedule.run(&mut world);

        let shadow = world.get::<ShadowEntity>(entity).unwrap();
        assert!(shadow.visible);

        // Turn off storm
        world.insert_resource(WeatherState {
            is_extreme: false,
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        schedule.run(&mut world);

        let shadow = world.get::<ShadowEntity>(entity).unwrap();
        assert!(!shadow.visible);
    }
}

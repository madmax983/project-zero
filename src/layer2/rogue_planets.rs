
use bevy_ecs::prelude::*;
use bevy_time::Time;
use bevy_time::Timer;
use crate::layer2::fleet::InOrbit;

/// Configuration for Rogue Planets.
#[derive(Resource, Debug, Clone)]
pub struct RoguePlanetConfig {
    /// The default duration a rogue planet stays in the system before despawning.
    pub default_drift_duration_seconds: f32,
}

impl Default for RoguePlanetConfig {
    fn default() -> Self {
        Self {
            default_drift_duration_seconds: 6.0 * 30.0 * 24.0, // Rough 6 months in "hours"
        }
    }
}

/// Marker component for a Rogue Planet.
#[derive(Component)]
pub struct RoguePlanet;

/// Component managing the despawn timer for a drifting Rogue Planet.
#[derive(Component)]
pub struct RoguePlanetDrift {
    pub timer: Timer,
}

/// System that ticks the drift timer of Rogue Planets and despawns them when they drift away.
/// Also removes the `InOrbit` component from any fleets currently orbiting it.
pub fn rogue_planet_drift_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RoguePlanetDrift), With<RoguePlanet>>,
    mut fleet_query: Query<(Entity, &InOrbit)>,
) {
    for (planet_entity, mut drift) in query.iter_mut() {
        drift.timer.tick(time.delta());

        if drift.timer.just_finished() {
            // Clean up fleets orbiting the rogue planet
            for (fleet_entity, orbit) in fleet_query.iter_mut() {
                if orbit.parent == planet_entity {
                    commands.entity(fleet_entity).remove::<InOrbit>();
                }
            }

            // Despawn the rogue planet
            commands.entity(planet_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::SystemBody;
    use crate::layer1::solar::SolarPower;
    use crate::layer2::mining::MiningTarget;
    use crate::layer1::economy::resources::ResourceType;

    #[test]
    fn test_rogue_planet_drift_and_despawn() {
        let mut app = bevy::app::App::new();
        app.insert_resource(bevy::time::Time::default() as bevy::time::Time);
        app.insert_resource(RoguePlanetConfig::default());

        let entity = app.world_mut().spawn((
            RoguePlanet,
            RoguePlanetDrift {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        )).id();

        // Spawn a dummy fleet in orbit
        let fleet_entity = app.world_mut().spawn(InOrbit {
            parent: entity,
        }).id();

        app.add_systems(Update, rogue_planet_drift_system);

        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1.5));
        app.update();

        // Verify the Rogue Planet entity is despawned
        assert!(app.world().get_entity(entity).is_err());

        // Verify the fleet is no longer in orbit
        assert!(app.world().get::<InOrbit>(fleet_entity).is_none());
    }

    #[test]
    fn test_rogue_planet_solar_power_and_resources() {
        let mut app = bevy::app::App::new();

        let entity = app.world_mut().spawn((
            RoguePlanet,
            SystemBody,
            SolarPower { base_output: 0.0 }, // 0 Solar Power
            MiningTarget {
                resource_type: ResourceType::Scrap, // Rare resource
                amount: 10000.0,
                mining_difficulty: 1.0,
            },
        )).id();

        let solar = app.world().get::<SolarPower>(entity).unwrap();
        assert_eq!(solar.base_output, 0.0);

        let target = app.world().get::<MiningTarget>(entity).unwrap();
        assert!(matches!(target.resource_type, ResourceType::Scrap));
        assert!(target.amount > 0.0);
    }
}

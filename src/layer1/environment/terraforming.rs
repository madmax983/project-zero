// src/layer1/terraforming.rs

use crate::layer1::atmosphere::AtmosphereGrid;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::PowerConsumer;
use crate::layer1::health::Health;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// Global resource tracking planetary toxicity and temperature.
#[derive(Resource, Debug, Clone)]
pub struct PlanetaryAtmosphere {
    /// 0.0 = Earth-like, 1.0 = Toxic Wasteland
    pub toxicity: f32,
    /// Global average temperature offset (Celcius)
    pub temperature: f32,
}

impl Default for PlanetaryAtmosphere {
    fn default() -> Self {
        Self {
            toxicity: 0.8,      // Start hostile
            temperature: -20.0, // Start cold
        }
    }
}

/// System to update planetary atmosphere based on active processors.
#[allow(clippy::explicit_iter_loop)]
pub fn update_planetary_atmosphere_system(
    mut atmosphere: ResMut<PlanetaryAtmosphere>,
    query: Query<(&Building, &PowerConsumer)>,
) {
    let mut toxicity_change = 0.0;
    let mut temp_change = 0.0;

    for (building, power) in query.iter() {
        if building.building_type == BuildingType::AtmosphericProcessor && power.active {
            // Reduce toxicity slowly, raise temperature slowly
            toxicity_change -= 0.0001;
            temp_change += 0.001;
        }
    }

    atmosphere.toxicity = (atmosphere.toxicity + toxicity_change).clamp(0.0, 1.0);
    atmosphere.temperature = (atmosphere.temperature + temp_change).clamp(-100.0, 100.0);
}

/// System to apply planetary effects (toxicity) to local atmosphere and pops.
#[allow(clippy::suboptimal_flops, clippy::explicit_iter_loop)]
pub fn apply_planetary_effects_system(
    atmosphere: Res<PlanetaryAtmosphere>,
    mut grid: ResMut<AtmosphereGrid>,
    mut query: Query<&mut Health, With<Pop>>,
) {
    // 1. Modify Atmosphere Grid Decay
    let toxicity_factor = atmosphere.toxicity;
    let target_rate = 0.90 + (toxicity_factor * 0.099);

    // Respect existing high rates (e.g. from Thermal Inversion which sets it to 1.0)
    // We assume update_weather_diffusion_system runs before this.
    grid.diffusion_rate = grid.diffusion_rate.max(target_rate);

    // 2. Global Health Damage (if toxicity is high)
    if atmosphere.toxicity > 0.5 {
        let damage = (atmosphere.toxicity - 0.5) * 0.05;
        for mut health in query.iter_mut() {
            if !health.has_condition(crate::layer1::biology::health::HealthCondition::RustLung) {
                health.take_damage(damage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::atmosphere::DiffusionConfig;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_planetary_atmosphere_initialization() {
        let atmosphere = PlanetaryAtmosphere::default();
        // Default planet is hostile
        assert!(atmosphere.toxicity > 0.5);
        assert!(atmosphere.temperature < 20.0); // Assuming cold planet start
    }

    #[test]
    fn test_processor_reduces_toxicity() {
        let mut world = World::new();
        let atmosphere = PlanetaryAtmosphere {
            toxicity: 1.0,
            temperature: 0.0,
        };
        world.insert_resource(atmosphere);

        // Spawn powered Processor in "Detoxify" mode
        world.spawn((
            Building {
                building_type: BuildingType::AtmosphericProcessor,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 500.0,
                active: true,
            }, // Fully powered
        ));

        // Run update multiple times to simulate time passing
        let mut schedule = Schedule::default();
        schedule.add_systems(update_planetary_atmosphere_system);

        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let new_atmosphere = world.resource::<PlanetaryAtmosphere>();
        assert!(
            new_atmosphere.toxicity < 1.0,
            "Toxicity should decrease with active processor"
        );
    }

    #[test]
    fn test_processor_requires_power() {
        let mut world = World::new();
        let atmosphere = PlanetaryAtmosphere {
            toxicity: 1.0,
            temperature: 0.0,
        };
        world.insert_resource(atmosphere);

        // Spawn unpowered Processor
        world.spawn((
            Building {
                building_type: BuildingType::AtmosphericProcessor,
            },
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                demand: 500.0,
                active: false,
            }, // No power
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_planetary_atmosphere_system);
        schedule.run(&mut world);

        let new_atmosphere = world.resource::<PlanetaryAtmosphere>();
        assert_eq!(
            new_atmosphere.toxicity, 1.0,
            "Toxicity should NOT change without power"
        );
    }

    #[test]
    fn test_global_toxicity_affects_local_diffusion() {
        let mut world = World::new();
        // High toxicity planet
        world.insert_resource(PlanetaryAtmosphere {
            toxicity: 0.9,
            temperature: 0.0,
        });
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(DiffusionConfig::default());
        // Add Pop just to satisfy query
        world.spawn((Pop, Health::default()));

        // Initialize diffusion rate (normally done by weather system)
        world.resource_mut::<AtmosphereGrid>().diffusion_rate = 0.99; // Default

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_planetary_effects_system);
        schedule.run(&mut world);

        let grid = world.resource::<AtmosphereGrid>();
        // High global toxicity means local smog stays longer (lower decay/diffusion rate)
        assert!(
            grid.diffusion_rate > 0.98,
            "High toxicity should increase retention (slower decay). Got: {}",
            grid.diffusion_rate
        );
    }

    #[test]
    fn test_low_toxicity_accelerates_decay() {
        let mut world = World::new();
        // Clean planet
        world.insert_resource(PlanetaryAtmosphere {
            toxicity: 0.1,
            temperature: 0.0,
        });
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(DiffusionConfig::default());
        world.spawn((Pop, Health::default()));

        // Reset rate to default lowish value
        world.resource_mut::<AtmosphereGrid>().diffusion_rate = 0.90;

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_planetary_effects_system);
        schedule.run(&mut world);

        let grid = world.resource::<AtmosphereGrid>();
        // 0.90 + (0.1 * 0.099) = 0.9099.
        // It should be close to 0.91.
        assert!(
            grid.diffusion_rate < 0.95,
            "Low toxicity should decrease retention (faster decay). Got: {}",
            grid.diffusion_rate
        );
    }

    #[test]
    fn test_global_toxicity_damages_pops() {
        let mut world = World::new();
        world.insert_resource(PlanetaryAtmosphere {
            toxicity: 1.0,
            temperature: 0.0,
        }); // Max toxicity
        world.insert_resource(AtmosphereGrid::new(10, 10)); // Required by system

        let pop = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_planetary_effects_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(
            health.current < 100.0,
            "Global toxicity should damage exposed pops"
        );
    }
}

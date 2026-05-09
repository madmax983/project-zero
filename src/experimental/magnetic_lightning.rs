#![allow(clippy::type_complexity)]
//! Magnetic Lightning (Nova Feature).
//!
//! # The Spark
//! We have a `MagneticStorm` weather type in `WeatherType`, a `PowerGrid` system (`PowerSource`, `PowerConsumer`, `EnergyGrid`, `GridOverloadEvent` in `src/layer1/energy/mod.rs`), and a `Health` system.
//!
//! # The Feature
//! During a `MagneticStorm`, there's a chance a "Lightning Strike" hits a random building or pop.
//! If the building is a `PowerSource`, it temporarily gains a massive surge in `output` (a "Supercharge" component),
//! but risks taking structural damage. If it hits an entity with `Health` (like a Pop) outside, they take immediate damage.
//!
//! # The Potential
//! Turns a hazardous weather event into a risk-reward scenario. Players might try to harness the storm
//! for massive free power, but risk grid overload or injury to their Pops.

use crate::layer1::biology::health::Health;
use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use rand::Rng;

const MAGNETIC_LIGHTNING_CHANCE: f64 = 0.05;

/// Component indicating a power source is supercharged by lightning.
#[derive(Component)]
pub struct Supercharge {
    pub duration: u32,
    pub multiplier: f32,
    pub base_output: f32,
}

pub fn magnetic_lightning_system(
    mut commands: Commands,
    weather_state: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(Entity, &GridPosition, &mut Health), With<Pop>>,
    mut power_sources: Query<(
        Entity,
        &GridPosition,
        &PowerSource,
        Option<&mut Supercharge>,
    )>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    if let Some(state) = weather_state {
        if state.current_weather != WeatherType::MagneticStorm {
            return;
        }
    } else {
        return;
    }

    let mut rng = rand::thread_rng();
    let has_roof = |x: i32, y: i32| -> bool {
        if let Some(ref grid) = roof_grid {
            grid.has_roof(x, y)
        } else {
            false
        }
    };

    // Damage Pops outside
    for (_entity, pos, mut health) in pops.iter_mut() {
        if !has_roof(pos.x, pos.y) && rng.gen_bool(MAGNETIC_LIGHTNING_CHANCE) {
            health.current -= 20.0;
            if health.current < 0.0 {
                health.current = 0.0;
            }
            if let Some(ref mut l) = log {
                l.add_colored(
                    format!(
                        "A pop was struck by magnetic lightning at ({}, {})!",
                        pos.x, pos.y
                    ),
                    ratatui::style::Color::Magenta,
                );
            }
        }
    }

    // Supercharge PowerSources
    for (entity, pos, power_source, supercharge_opt) in power_sources.iter_mut() {
        if !has_roof(pos.x, pos.y) && rng.gen_bool(MAGNETIC_LIGHTNING_CHANCE) {
            if let Some(mut sc) = supercharge_opt {
                sc.duration = 10;
            } else {
                commands.entity(entity).insert(Supercharge {
                    duration: 10,
                    multiplier: 5.0,
                    base_output: power_source.output,
                });
            }
            if let Some(ref mut l) = log {
                l.add_colored(
                    format!(
                        "A power source was supercharged by magnetic lightning at ({}, {})!",
                        pos.x, pos.y
                    ),
                    ratatui::style::Color::Yellow,
                );
            }
        }
    }
}

pub fn apply_supercharge_system(mut query: Query<(&mut PowerSource, &Supercharge)>) {
    for (mut power_source, supercharge) in query.iter_mut() {
        // We set the output to the base multiplied by the supercharge.
        // We use absolute value difference to prevent constant event triggers on change detection.
        let target_output = supercharge.base_output * supercharge.multiplier;
        if (power_source.output - target_output).abs() > f32::EPSILON {
            power_source.output = target_output;
        }
    }
}

pub fn decay_supercharge_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Supercharge, &mut PowerSource)>,
) {
    for (entity, mut supercharge, mut power_source) in query.iter_mut() {
        supercharge.duration = supercharge.duration.saturating_sub(1);
        if supercharge.duration == 0 {
            power_source.output = supercharge.base_output;
            commands.entity(entity).remove::<Supercharge>();
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        magnetic_lightning_system,
        apply_supercharge_system,
        decay_supercharge_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_lightning_strikes_pop_damage() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        let grid = RoofGrid::new(10, 10);
        // No roof at (5, 5)
        world.insert_resource(grid);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        // Run multiple times to trigger the lightning strike reliably
        for _ in 0..1000 {
            world.run_system_once(magnetic_lightning_system).unwrap();
            world.flush();
        }

        let health = world.get::<Health>(pop).unwrap();
        assert!(
            health.current < 100.0,
            "Pop outside during magnetic storm should take lightning damage"
        );
    }

    #[test]
    fn test_lightning_strikes_generator_supercharge() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        let generator = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                PowerSource {
                    output: 10.0,
                    active: true,
                },
            ))
            .id();

        // Run multiple times to trigger the lightning strike reliably
        for _ in 0..1000 {
            world.run_system_once(magnetic_lightning_system).unwrap();
            world.flush();
        }

        let supercharge = world.get::<Supercharge>(generator);
        assert!(
            supercharge.is_some(),
            "PowerSource struck by lightning should receive Supercharge component"
        );

        world.run_system_once(apply_supercharge_system).unwrap();
        let power = world.get::<PowerSource>(generator).unwrap();
        assert!(
            power.output > 10.0,
            "PowerSource output should be multiplied by supercharge"
        );
    }
}

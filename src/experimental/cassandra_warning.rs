#![allow(clippy::type_complexity)]
//! Cassandra's Warning (Nova Feature).
//!
//! # The Spark
//! We have a `Dazing` mental break (`ActionType::Daze`) and hazardous procedural disasters like `MagneticStorm` and `MutagenicRain`.
//!
//! # The Feature
//! A new mental break effect. When a Pop enters a Daze and severe weather is active, they become a Cassandra,
//! shouting prophetic warnings about the doom upon the colony. This provides narrative flavor but causes panic
//! (stress damage) to nearby Pops who hear their screams.
//!
//! # Boundaries
//! We piggyback on `ActionType::Daze` rather than modifying core enums to ensure this module is completely additive
//! and can be toggled via the `nova` feature flag.

use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

const PANIC_RADIUS_SQ: i32 = 25; // 5 tile radius
const STRESS_PENALTY_PER_TICK: f32 = 0.5;

/// Component added to pops that have already shrieked a prophecy recently,
/// to prevent log spamming every single tick.
#[derive(Component)]
pub struct ProphesyCooldown(pub u32);

pub fn cassandra_prophesy_system(
    mut commands: Commands,
    weather_state: Option<Res<WeatherState>>,
    mut log: Option<ResMut<MessageLog>>,
    dazing_pops: Query<(Entity, &GridPosition, &PopAction), (With<Pop>, Without<ProphesyCooldown>)>,
    mut all_pops: Query<(Entity, &GridPosition, &mut StressTracker), With<Pop>>,
) {
    let Some(weather) = weather_state else {
        return;
    };

    // Check if the current weather is hazardous enough to trigger prophecies
    let is_hazardous = matches!(
        weather.current_weather,
        WeatherType::MagneticStorm | WeatherType::MutagenicRain | WeatherType::SporeStorm
    );

    if !is_hazardous {
        return;
    }

    let mut rng = rand::thread_rng();
    let mut panic_centers = Vec::new();

    for (entity, pos, action) in dazing_pops.iter() {
        if action.current == ActionType::Daze {
            // Add a cooldown so they don't scream every single tick.
            // Say they scream once every 50-100 ticks while dazing.
            if rng.gen_bool(0.1) {
                commands.entity(entity).insert(ProphesyCooldown(60));
                panic_centers.push(*pos);

                if let Some(ref mut l) = log {
                    let text = format!(
                        "A dazing pop screams: 'The {} will consume us all!'",
                        weather.current_weather.name()
                    );
                    l.add_colored(text, ratatui::style::Color::Magenta);
                }
            }
        }
    }

    // Apply stress to nearby pops (who are not the prophet)
    for center in panic_centers {
        for (_, pos, mut stress) in all_pops.iter_mut() {
            let dx = pos.x - center.x;
            let dy = pos.y - center.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= PANIC_RADIUS_SQ && dist_sq > 0 {
                // > 0 to exclude self, though the self might not have StressTracker
                stress.accumulated_stress += STRESS_PENALTY_PER_TICK;
            }
        }
    }
}

pub fn update_prophesy_cooldown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ProphesyCooldown)>,
) {
    for (entity, mut cooldown) in query.iter_mut() {
        if cooldown.0 > 0 {
            cooldown.0 -= 1;
        }
        if cooldown.0 == 0 {
            commands.entity(entity).remove::<ProphesyCooldown>();
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((cassandra_prophesy_system, update_prophesy_cooldown_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_cassandra_warning_triggers_on_hazardous_weather() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        // The prophet
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Daze,
                    ..Default::default()
                },
            ))
            .id();

        // The innocent bystander
        let bystander = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 }, // Within radius
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        // Force the RNG outcome (probabilistic tests can be flaky, but we can just run it multiple times to ensure it hits)
        // For simplicity, we will run the system enough times to statistically ensure it triggers
        for _ in 0..100 {
            world.run_system_once(cassandra_prophesy_system).unwrap();
        }

        // Verify that the prophet gained a cooldown
        assert!(
            world.get::<ProphesyCooldown>(pop).is_some(),
            "Prophet should have gained a cooldown"
        );

        // Verify that the bystander gained stress
        let stress = world.get::<StressTracker>(bystander).unwrap();
        assert!(
            stress.accumulated_stress > 0.0,
            "Bystander should have gained stress from the panic aura"
        );
    }

    #[test]
    fn test_cassandra_warning_ignores_clear_weather() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 10,
        });

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Daze,
                    ..Default::default()
                },
            ))
            .id();

        for _ in 0..100 {
            world.run_system_once(cassandra_prophesy_system).unwrap();
        }

        assert!(
            world.get::<ProphesyCooldown>(pop).is_none(),
            "Prophet should not trigger in clear weather"
        );
    }
}

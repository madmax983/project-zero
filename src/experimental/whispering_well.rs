#![allow(clippy::type_complexity)]
//! The Whispering Well (Nova Feature).
//!
//! # The Spark
//! We have a `BuildingType::Well` and a system for tracking `Morale` and `Knowledge`.
//! What if pops whisper their darkest thoughts into the deep, and the deep whispers back?
//!
//! # The Feature
//! If a `Pop` with extremely low morale (`< 0.1`) is near a `BuildingType::Well`, they inadvertently awaken it.
//! The well becomes a `WhisperingWell`. It acts as a permanent `NoiseSource` and slowly drains the morale of anyone passing nearby.
//! However, it also passively generates `Knowledge` for the colony as it echoes ancient secrets back from the crust.

use crate::layer1::architecture::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::physics::acoustic::NoiseSource;
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

const WHISPERING_AWAKEN_RADIUS: u32 = 2;
const WHISPERING_EFFECT_RADIUS: u32 = 5;

/// Component indicating a Well has awakened and is whispering.
#[derive(Component)]
pub struct WhisperingWell;

/// Detects if a very depressed Pop is near a Well, and awakens it.
pub fn detect_whispering_well_system(
    mut commands: Commands,
    pops: Query<(&GridPosition, &Morale), With<Pop>>,
    wells: Query<(Entity, &GridPosition, &Building), Without<WhisperingWell>>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    for (well_entity, well_pos, building) in wells.iter() {
        if building.building_type == BuildingType::Well {
            for (pop_pos, morale) in pops.iter() {
                if morale.value < 0.1
                    && pop_pos.distance_chebyshev(*well_pos) <= WHISPERING_AWAKEN_RADIUS
                {
                    // Awaken the well!
                    commands.entity(well_entity).insert((
                        WhisperingWell,
                        NoiseSource {
                            radius: 10.0,
                            intensity: 0.6, // An unsettling, low hum
                        },
                    ));

                    if let Some(ref mut log_res) = log {
                        log_res.add_colored(
                            "WARNING: A pop's despair has awakened a Whispering Well. The deep speaks.".to_string(),
                            ratatui::style::Color::Red,
                        );
                    }

                    // A single pop awakening it is enough
                    break;
                }
            }
        }
    }
}

/// The effects of an active Whispering Well.
pub fn whispering_well_effects_system(
    wells: Query<&GridPosition, With<WhisperingWell>>,
    mut pops: Query<(&GridPosition, &mut Morale), With<Pop>>,
    mut resources: ResMut<ColonyResources>,
) {
    if wells.is_empty() {
        return;
    }

    for well_pos in wells.iter() {
        // Passively generates knowledge
        resources.add_knowledge(0.05);

        // Degrades morale of nearby pops
        for (pop_pos, mut morale) in pops.iter_mut() {
            if pop_pos.distance_chebyshev(*well_pos) <= WHISPERING_EFFECT_RADIUS {
                // We don't want to instantly crush them, just add a modifier or slowly drain.
                // Modifiers are time-based, let's just drain their base value slightly so it acts like an aura.
                morale.value = (morale.value - 0.005).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        detect_whispering_well_system,
        whispering_well_effects_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_whispering_well_awakens_and_generates_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        let well = world
            .spawn((
                Building {
                    building_type: BuildingType::Well,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        // Spawn a happy pop, shouldn't trigger
        world.spawn((
            Pop,
            GridPosition { x: 11, y: 10 },
            Morale {
                value: 1.0,
                ..Default::default()
            },
        ));

        world
            .run_system_once(detect_whispering_well_system)
            .unwrap();
        assert!(world.get::<WhisperingWell>(well).is_none());

        // Spawn a depressed pop
        world.spawn((
            Pop,
            GridPosition { x: 11, y: 10 },
            Morale {
                value: 0.05,
                ..Default::default()
            },
        ));

        world
            .run_system_once(detect_whispering_well_system)
            .unwrap();
        assert!(world.get::<WhisperingWell>(well).is_some());
        assert!(world.get::<NoiseSource>(well).is_some());

        // Test effects
        world
            .run_system_once(whispering_well_effects_system)
            .unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!((resources.knowledge - 0.05).abs() < f32::EPSILON);
    }
}

//! Technological Rituals system.
//!
//! Implements the mechanics for "Machine Spirits" in Heirloom technology.
//! Ancient machines (Reactors, Fabricators) have a spirit that gets angry over time.
//! If too angry, they develop "Quirks" that hinder their operation (e.g., stopping production).
//!
//! Pops must perform rituals to appease the spirit (reduce anger) and remove quirks.
//! This adds a maintenance loop for high-value ancient tech.

use bevy_ecs::prelude::*;

/// Component tracking the anger of a machine spirit.
///
/// Anger increases over time via [`spirit_decay_system`] and decreases via [`perform_ritual`].
#[derive(Component, Debug, Clone, Default)]
pub struct MachineSpirit {
    /// The current anger level (0.0 to 100.0).
    pub anger: f32,
}

/// Types of quirks a machine can develop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuirkType {
    /// Stops production randomly or entirely.
    Glitchy,
    /// Risk of fire (not yet implemented).
    Overheating,
    /// Increases anger faster (not yet implemented).
    Demanding,
}

/// Component representing a negative trait on a machine.
#[derive(Component, Debug, Clone)]
pub struct Quirk {
    /// The type of quirk.
    pub quirk_type: QuirkType,
}

impl Quirk {
    /// Returns true if this quirk stops production.
    #[must_use]
    pub const fn stops_production(&self) -> bool {
        matches!(self.quirk_type, QuirkType::Glitchy)
    }
}

/// System that increases machine spirit anger over time.
///
/// Runs every tick. Anger caps at 100.0.
pub fn spirit_decay_system(mut query: Query<&mut MachineSpirit>) {
    const ANGER_RATE: f32 = 0.1;
    for mut spirit in &mut query {
        spirit.anger = (spirit.anger + ANGER_RATE).min(100.0);
    }
}

/// System that spawns quirks on angry machines.
///
/// If anger exceeds 80.0, a Quirk is added (if not already present).
pub fn quirk_generation_system(
    mut commands: Commands,
    query: Query<(Entity, &MachineSpirit), Without<Quirk>>,
) {
    const ANGER_THRESHOLD: f32 = 80.0;
    for (entity, spirit) in query.iter() {
        if spirit.anger > ANGER_THRESHOLD {
            // Manifest Quirk
            commands.entity(entity).insert(Quirk {
                quirk_type: QuirkType::Glitchy, // Default for MVP
            });
        }
    }
}

/// Performs a ritual on the target entity, reducing anger and potentially removing quirks.
///
/// *   Reduces anger by 50.0 (clamped to 0.0).
/// *   Removes [`Quirk`] if anger drops below 50.0.
pub fn perform_ritual(world: &mut World, target: Entity) {
    const ANGER_REDUCTION: f32 = 50.0;
    const QUIRK_REMOVAL_THRESHOLD: f32 = 50.0;

    if let Some(mut spirit) = world.get_mut::<MachineSpirit>(target) {
        spirit.anger = (spirit.anger - ANGER_REDUCTION).max(0.0);
    }

    // Check if we should remove the quirk
    let should_remove_quirk = world
        .get::<MachineSpirit>(target)
        .is_some_and(|s| s.anger < QUIRK_REMOVAL_THRESHOLD);

    if should_remove_quirk {
        world.entity_mut(target).remove::<Quirk>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::heirloom::AncientStructure;
    use bevy_ecs::{
        prelude::World,
        schedule::Schedule,
    };

    #[test]
    fn test_machine_spirit_initialization() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                MachineSpirit::default(), // Should start Happy/Neutral
            ))
            .id();

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger <= 0.0);
    }

    #[test]
    fn test_spirit_decay_increases_anger() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                MachineSpirit {
                    anger: 0.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run decay system
        let mut schedule = Schedule::default();
        schedule.add_systems(spirit_decay_system);
        schedule.run(&mut world);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger > 0.0, "Spirit should get angry over time");
    }

    #[test]
    fn test_high_anger_causes_quirk() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                MachineSpirit {
                    anger: 100.0,
                    ..Default::default()
                }, // Furious
            ))
            .id();

        // Run quirk generation system
        let mut schedule = Schedule::default();
        schedule.add_systems(quirk_generation_system);
        schedule.run(&mut world);

        let quirk = world.get::<Quirk>(entity);
        assert!(quirk.is_some(), "High anger should manifest a Quirk");
    }

    #[test]
    fn test_ritual_reduces_anger() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                MachineSpirit {
                    anger: 50.0,
                    ..Default::default()
                },
                Quirk {
                    quirk_type: QuirkType::Glitchy,
                },
            ))
            .id();

        // Perform ritual
        perform_ritual(&mut world, entity);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!(spirit.anger < 50.0, "Ritual should reduce anger");

        // Quirk might be removed or suppressed
        let quirk = world.get::<Quirk>(entity);
        assert!(
            quirk.is_none(),
            "Ritual should remove/suppress active Quirk"
        );
    }

    #[test]
    fn test_glitchy_quirk_stops_production() {
        // This test requires integrating with production systems,
        // but unit test can check the flag.
        let quirk = Quirk {
            quirk_type: QuirkType::Glitchy,
        };
        assert!(
            quirk.stops_production(),
            "Glitchy quirk should stop production"
        );
    }

    #[test]
    fn test_quirk_not_generated_low_anger() {
        let mut world = World::new();
        let entity = world
            .spawn((MachineSpirit {
                anger: 50.0,
                ..Default::default()
            },))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(quirk_generation_system);
        schedule.run(&mut world);

        let quirk = world.get::<Quirk>(entity);
        assert!(quirk.is_none(), "Low anger should not manifest a Quirk");
    }

    #[test]
    fn test_ritual_clamped_at_zero() {
        let mut world = World::new();
        let entity = world
            .spawn((MachineSpirit {
                anger: 10.0,
                ..Default::default()
            },))
            .id();

        perform_ritual(&mut world, entity);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!((spirit.anger - 0.0).abs() < f32::EPSILON, "Anger should clamp to 0.0");
    }

    #[test]
    fn test_ritual_keeps_quirk_if_anger_high() {
        let mut world = World::new();
        // Start with very high anger (100)
        // Ritual reduces by 50 -> 50.
        // Threshold is < 50. So 50 is NOT < 50. Quirk stays?
        let entity = world
            .spawn((
                MachineSpirit {
                    anger: 100.0,
                    ..Default::default()
                },
                Quirk {
                    quirk_type: QuirkType::Glitchy,
                },
            ))
            .id();

        perform_ritual(&mut world, entity);

        let spirit = world.get::<MachineSpirit>(entity).unwrap();
        assert!((spirit.anger - 50.0).abs() < f32::EPSILON);

        let quirk = world.get::<Quirk>(entity);
        assert!(
            quirk.is_some(),
            "Quirk should remain if anger is still high (>= 50)"
        );

        // Another ritual should remove it (0.0 < 50.0)
        perform_ritual(&mut world, entity);
        let quirk = world.get::<Quirk>(entity);
        assert!(quirk.is_none(), "Second ritual should remove Quirk");
    }
}

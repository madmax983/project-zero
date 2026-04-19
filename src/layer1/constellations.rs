//! The `Constellations` module manages the procedurally generated night sky and its effects on the colony.
//!
//! # The Story
//! The night sky of the frontier is not static. Procedurally generated constellations rotate into view,
//! each bringing its own mythological resonance (`myth`). Observatory workers who study the sky have a
//! chance to be affected by the ascendant constellation, gaining temporary emotional modifiers such as
//! `Cosmic Inspiration` or `Existential Dread`.
//!
//! # Mechanics
//! - A `Sky` resource initializes with a set of generated `Constellation`s.
//! - `update_sky_system` rotates the ascendant constellation over time.
//! - `observe_constellations_system` allows observatory workers to randomly acquire the active constellation's effect.
//!

#![allow(clippy::cast_precision_loss)]
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::observatory::Observatory;
use crate::shared::narrative::NarrativeGenerator;
use bevy_ecs::prelude::*;
use rand::{thread_rng, Rng};

/// The effect a constellation has on those who observe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstellationEffect {
    /// Inspires creativity and hope.
    Inspiration,
    /// Induces fear of the vast unknown.
    Dread,
    /// Calms the mind and reduces stress.
    Calm,
    /// Invigorates the body and spirit.
    Vigor,
}

impl ConstellationEffect {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inspiration => "Cosmic Inspiration",
            Self::Dread => "Existential Dread",
            Self::Calm => "Starlit Calm",
            Self::Vigor => "Celestial Vigor",
        }
    }

    const fn value(self) -> f32 {
        match self {
            Self::Inspiration => 0.15,
            Self::Dread => -0.10,
            Self::Calm => 0.10,
            Self::Vigor => 0.12,
        }
    }

    #[allow(dead_code)]
    const fn description(self) -> &'static str {
        match self {
            Self::Inspiration => "Feeling inspired by the stars.",
            Self::Dread => "Feeling small in the vast universe.",
            Self::Calm => "Feeling soothed by the night sky.",
            Self::Vigor => "Feeling energized by the cosmos.",
        }
    }
}

/// A procedurally generated constellation.
#[derive(Debug, Clone)]
pub struct Constellation {
    /// The name of the constellation (e.g., "The Weeping Gear").
    pub name: String,
    /// Flavor text describing the constellation.
    pub myth: String,
    /// The effect it bestows.
    pub effect: ConstellationEffect,
    /// Star positions (normalized 0.0-1.0) for potential UI rendering.
    pub stars: Vec<(f32, f32)>,
}

/// Resource holding the state of the night sky.
#[derive(Resource)]
pub struct Sky {
    /// The list of generated constellations.
    pub constellations: Vec<Constellation>,
    /// The index of the currently ascendant constellation.
    pub current_index: usize,
}

impl FromWorld for Sky {
    fn from_world(world: &mut World) -> Self {
        let mut constellations = Vec::new();
        let mut rng = thread_rng();

        // Try to get the generator to make cool names
        let generator = world.get_resource::<NarrativeGenerator>();

        let count = 12; // Zodiac style
        for i in 0..count {
            let effect = match i % 4 {
                0 => ConstellationEffect::Inspiration,
                1 => ConstellationEffect::Dread,
                2 => ConstellationEffect::Calm,
                _ => ConstellationEffect::Vigor,
            };

            let name = generator.map_or_else(
                || format!("Constellation {i}"),
                |narrator| {
                    // Try to use fragments if available, else fallback
                    let prefix = narrator
                        .get_random_fragment("STAR_PREFIX")
                        .cloned()
                        .unwrap_or_else(|| "The".to_string());
                    let suffix = narrator
                        .get_random_fragment("STAR_SUFFIX")
                        .cloned()
                        .unwrap_or_else(|| "Star".to_string());
                    format!("{prefix} {suffix}")
                },
            );

            // Generate random stars
            let star_count = rng.gen_range(3..8);
            let mut stars = Vec::new();
            for _ in 0..star_count {
                stars.push((rng.r#gen::<f32>(), rng.r#gen::<f32>()));
            }

            constellations.push(Constellation {
                name,
                myth: format!("A collection of stars resembling {}.", effect.label()),
                effect,
                stars,
            });
        }

        Self {
            constellations,
            current_index: 0,
        }
    }
}

/// Rotates the `Sky` resource over time, changing the currently ascendant constellation.
///
/// The sky advances its `current_index` every 1,000 simulation ticks, wrapping around
/// once it reaches the end of the `constellations` list.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::constellations::{Sky, Constellation, ConstellationEffect, update_sky_system};
/// use scale::shared::time::SimulationTime;
///
/// let mut world = World::new();
///
/// // Create a sky with two constellations
/// let sky = Sky {
///     constellations: vec![
///         Constellation { name: "Star A".into(), myth: "".into(), effect: ConstellationEffect::Calm, stars: vec![] },
///         Constellation { name: "Star B".into(), myth: "".into(), effect: ConstellationEffect::Dread, stars: vec![] },
///     ],
///     current_index: 0,
/// };
/// world.insert_resource(sky);
/// world.insert_resource(SimulationTime { tick: 1500, ..Default::default() });
///
/// // Run the system
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_sky_system);
/// schedule.run(&mut world);
///
/// // Tick 1500 / 1000 = 1. The index should have rotated to 1.
/// assert_eq!(world.resource::<Sky>().current_index, 1);
/// ```
pub fn update_sky_system(mut sky: ResMut<Sky>, time: Res<crate::shared::time::SimulationTime>) {
    if sky.constellations.is_empty() {
        return;
    }

    // Rotate every 1000 ticks (approx 1 day in some timescales, or just a shift)
    // To make it deterministic but changing, we can use tick / 1000
    let index = (time.tick / 1000) as usize % sky.constellations.len();
    sky.current_index = index;
}

/// Applies the current constellation's effect to `Pop`s working at an `Observatory`.
///
/// Each tick, `ObservatoryWorker`s have a 5% chance to be struck by the mythological
/// resonance of the currently ascendant constellation in the `Sky`. If triggered, a
/// `MoodModifier` matching the constellation's effect is added to the `Pop`'s `Morale`.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::constellations::{Sky, Constellation, ConstellationEffect, observe_constellations_system};
/// use scale::layer1::actions::{AssignedTo, AssignmentType};
/// use scale::layer1::observatory::Observatory;
/// use scale::layer1::morale::Morale;
///
/// let mut world = World::new();
///
/// // Setup a sky with a known effect
/// let sky = Sky {
///     constellations: vec![Constellation {
///         name: "The Great Gear".into(),
///         myth: "".into(),
///         effect: ConstellationEffect::Vigor,
///         stars: vec![]
///     }],
///     current_index: 0,
/// };
/// world.insert_resource(sky);
///
/// // Spawn an observatory
/// let observatory_id = world.spawn(Observatory { efficiency: 1.0 }).id();
///
/// // Spawn a Pop assigned to the observatory
/// let pop_id = world.spawn((
///     AssignedTo { entity: observatory_id, assignment_type: AssignmentType::ObservatoryWorker },
///     Morale::default(),
/// )).id();
///
/// // Run the system repeatedly to ensure the 5% chance triggers
/// let mut schedule = Schedule::default();
/// schedule.add_systems(observe_constellations_system);
/// for _ in 0..100 {
///     schedule.run(&mut world);
/// }
///
/// // The Pop should have received the Vigor modifier
/// let morale = world.get::<Morale>(pop_id).expect("Component should exist or System should run");
/// assert!(morale.modifiers.iter().any(|m| m.label == ConstellationEffect::Vigor.label()));
/// ```
pub fn observe_constellations_system(
    sky: Res<Sky>,
    mut pops: Query<(&AssignedTo, &mut Morale)>,
    observatories: Query<&Observatory>,
) {
    if sky.constellations.is_empty() {
        return;
    }

    let current = &sky.constellations[sky.current_index];
    let mut rng = thread_rng();

    for (assignment, mut morale) in &mut pops {
        // Only affects Observatory workers
        if assignment.assignment_type == AssignmentType::ObservatoryWorker
            && observatories.get(assignment.entity).is_ok()
        {
            // 5% chance per tick to notice the specific constellation and get its specific buff
            // This is separate from the generic "Overview Effect" in observatory.rs
            if rng.gen_bool(0.05) {
                // Check if they already have this specific buff to avoid stacking spam
                if !morale
                    .modifiers
                    .iter()
                    .any(|m| m.label == current.effect.label())
                {
                    morale.add_modifier(MoodModifier {
                        label: current.effect.label().to_string(),
                        value: current.effect.value(),
                        duration: 800, // Lasts a while
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::actions::AssignmentType;
    use crate::layer1::observatory::Observatory;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_sky_generation() {
        let mut world = World::new();
        // Insert empty generator to test fallback or basic gen
        world.insert_resource(NarrativeGenerator::default());

        // Manual init since FromWorld is used via init_resource usually
        // But we can call it manually if we want to test logic
        let sky = Sky::from_world(&mut world);
        assert_eq!(sky.constellations.len(), 12);
        assert_eq!(sky.current_index, 0);
    }

    #[test]
    fn test_sky_rotation() {
        let mut world = World::new();
        let sky = Sky {
            constellations: vec![
                Constellation {
                    name: "A".into(),
                    myth: "".into(),
                    effect: ConstellationEffect::Calm,
                    stars: vec![],
                },
                Constellation {
                    name: "B".into(),
                    myth: "".into(),
                    effect: ConstellationEffect::Dread,
                    stars: vec![],
                },
            ],
            current_index: 0,
        };

        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sky_system);

        // Tick 0 -> Index 0
        world.insert_resource(sky);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Sky>().current_index, 0);

        // Tick 1000 -> Index 1
        world.resource_mut::<SimulationTime>().tick = 1000;
        schedule.run(&mut world);
        assert_eq!(world.resource::<Sky>().current_index, 1);

        // Tick 2000 -> Index 0 (wrap)
        world.resource_mut::<SimulationTime>().tick = 2000;
        schedule.run(&mut world);
        assert_eq!(world.resource::<Sky>().current_index, 0);
    }

    #[test]
    fn test_observe_constellation_buff() {
        let mut world = World::new();
        let sky = Sky {
            constellations: vec![Constellation {
                name: "Test Star".into(),
                myth: "".into(),
                effect: ConstellationEffect::Inspiration,
                stars: vec![],
            }],
            current_index: 0,
        };
        world.insert_resource(sky);

        let observatory = world.spawn(Observatory { efficiency: 100.0 }).id();
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: observatory,
                    assignment_type: AssignmentType::ObservatoryWorker,
                },
                Morale::default(),
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(observe_constellations_system);

        // Run enough times to trigger 5% chance
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let morale = world.get::<Morale>(pop).expect("Component should exist or System should run");
        // It's probabilistic, but 100 trials for 5% is ~99.4% chance.
        // Assert at least one modifier exists
        assert!(morale
            .modifiers
            .iter()
            .any(|m| m.label == "Cosmic Inspiration"));
    }
}

//! The Panopticon Glitch Feature (Nova Expansion)
//!
//! # The Spark
//! We have `StressTracker` and `Morale` components. What if a building perfectly
//! manages these needs by imposing total surveillance, but occasionally malfunctions
//! and broadcasts everyone's private data, causing a massive social collapse?
//!
//! # The Feature
//! `PanopticonNode` component. It continuously lowers stress for nearby Pops (order),
//! but accumulates `instability`. When instability peaks, it triggers a "glitch,"
//! massively spiking stress and dropping morale as secrets are broadcast to the colony.
//!
//! # Potential
//! Forces players to weigh the benefits of perfectly stable (but artificial) society
//! against the devastating risk of a single point of failure exposing all secrets.

use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

/// A surveillance node that suppresses stress but builds instability.
#[derive(Component, Debug, Clone)]
pub struct PanopticonNode {
    /// Range of the surveillance grid.
    pub radius: i32,
    /// Current instability level.
    pub instability: f32,
    /// Threshold at which the node glitches.
    pub glitch_threshold: f32,
}

impl Default for PanopticonNode {
    fn default() -> Self {
        Self {
            radius: 5,
            instability: 0.0,
            glitch_threshold: 100.0,
        }
    }
}

/// System to operate the Panopticon grid.
pub fn panopticon_surveillance_system(
    mut nodes: Query<(&GridPosition, &mut PanopticonNode)>,
    mut pops: Query<(&GridPosition, &mut Morale, &mut StressTracker), With<Pop>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (node_pos, mut node) in nodes.iter_mut() {
        // Increment instability every tick
        node.instability += 1.0;

        let is_glitching = node.instability >= node.glitch_threshold;

        if is_glitching {
            // Glitch! Broadcast secrets.
            node.instability = 0.0;

            chronicle_events.send(AddChronicleEvent {
                text: "The Panopticon grid glitched, broadcasting everyone's secrets!".to_string(),
                importance: EventImportance::Major,
            ..Default::default()});

            // Apply massive penalties to all pops in range
            for (pop_pos, mut morale, mut stress) in pops.iter_mut() {
                if pop_pos.distance_chebyshev(*node_pos) <= node.radius as u32 {
                    morale.value = (morale.value - 0.5).max(0.0);
                    stress.accumulated_stress = (stress.accumulated_stress + 50.0).min(100.0);
                }
            }
        } else {
            // Normal operation: suppress stress slightly, keep morale somewhat stable
            for (pop_pos, mut morale, mut stress) in pops.iter_mut() {
                if pop_pos.distance_chebyshev(*node_pos) <= node.radius as u32 {
                    stress.accumulated_stress = (stress.accumulated_stress - 0.5).max(0.0);
                    // Surveillance is slightly dehumanizing, keeps morale from peaking
                    if morale.value > 0.8 {
                        morale.value = (morale.value - 0.01).max(0.8);
                    }
                }
            }
        }
    }
}

/// Registers the Panopticon Glitch system into the Bevy schedule.
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(panopticon_surveillance_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_panopticon_order() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();

        // Spawn a Panopticon node
        world.spawn((
            PanopticonNode {
                radius: 5,
                instability: 0.0,
                glitch_threshold: 100.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn a pop in range
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 2 },
                Morale {
                    modifiers: vec![],
                    value: 0.9,
                },
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        world
            .run_system_once(panopticon_surveillance_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();

        // Stress should be reduced by 0.5
        assert!((stress.accumulated_stress - 49.5).abs() < f32::EPSILON);
        // Morale > 0.8 should be reduced slightly by surveillance
        assert!((morale.value - 0.89).abs() < f32::EPSILON);
    }

    #[test]
    fn test_panopticon_glitch() {
        let mut world = World::new();
        world.init_resource::<Events<AddChronicleEvent>>();

        // Spawn a Panopticon node about to glitch
        world.spawn((
            PanopticonNode {
                radius: 5,
                instability: 99.0, // Will hit 100 on first tick
                glitch_threshold: 100.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn a pop in range
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 2, y: 2 },
                Morale {
                    modifiers: vec![],
                    value: 0.9,
                },
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        world
            .run_system_once(panopticon_surveillance_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();

        // Glitch! Stress spikes +50, Morale drops -0.5
        assert!((stress.accumulated_stress - 100.0).abs() < f32::EPSILON);
        assert!((morale.value - 0.4).abs() < f32::EPSILON);

        let events = world.resource::<Events<AddChronicleEvent>>();
        assert_eq!(events.len(), 1);
    }
}

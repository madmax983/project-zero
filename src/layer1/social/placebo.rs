use bevy_ecs::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceboProtocol {
    FakeReinforcements, // Reduces Panic
    VitaminX,           // Reduces Sickness Fear
    SafetyInspection,   // Reduces Collapse Fear
}

#[derive(Component)]
pub struct ActivePlacebo {
    pub protocol: PlaceboProtocol,
    pub duration: f32,
    pub stress_relief: f32,
    pub revealed: bool,
}

pub fn placebo_tick_system(
    mut placebos: Query<(Entity, &mut ActivePlacebo)>,
    mut pops: Query<&mut StressTracker>,
) {
    for (_, mut placebo) in placebos.iter_mut() {
        if placebo.duration > 0.0 {
            // Apply relief continuously (suppress stress)
            placebo.duration -= 1.0;

            for mut tracker in pops.iter_mut() {
                tracker.accumulated_stress -= 20.0;
            }
        } else {
            // Expire and set revealed
            placebo.revealed = true;
        }
    }
}

pub fn reveal_betrayal_system(
    mut commands: Commands,
    query: Query<(Entity, &ActivePlacebo)>,
    mut pops: Query<(&mut StressTracker, &mut Traits)>,
) {
    for (entity, placebo) in query.iter() {
        if placebo.revealed {
            for (mut tracker, mut traits) in pops.iter_mut() {
                tracker.accumulated_stress += placebo.stress_relief * 2.0;
                traits.0.insert(Trait::Distrustful);
            }
            commands.entity(entity).despawn();
        }
    }
}

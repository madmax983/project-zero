//! Echo Chamber Feature (Nova Expansion)
//!
//! # The Spark
//! We have a `Morale` and `StressTracker` system. What if a Pop's extreme emotional
//! state creates a temporary Aura that amplifies that same emotion in nearby Pops?
//!
//! # The Feature
//! `EchoingState` component. Pops with extreme stress or morale act as localized
//! sources of that emotion, creating cascading behavioral pockets.

use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct EchoingState {
    pub is_positive: bool,
    pub ticks_remaining: u32,
}

const ECHO_THRESHOLD_HIGH: f32 = 0.9;
const ECHO_THRESHOLD_LOW: f32 = 0.1;
const STRESS_ECHO_THRESHOLD: f32 = 90.0;
const ECHO_DURATION: u32 = 100;
const ECHO_RADIUS: u32 = 3;

/// System to detect extreme emotional states and turn Pops into Echo Chambers
pub fn detect_echo_chamber_system(
    mut commands: Commands,
    pops: Query<(Entity, &Morale, &StressTracker), (With<Pop>, Without<EchoingState>)>,
) {
    for (entity, morale, stress) in pops.iter() {
        if morale.value >= ECHO_THRESHOLD_HIGH {
            // High morale echo
            commands.entity(entity).insert(EchoingState {
                is_positive: true,
                ticks_remaining: ECHO_DURATION,
            });
        } else if morale.value <= ECHO_THRESHOLD_LOW || stress.accumulated_stress >= STRESS_ECHO_THRESHOLD {
            // Low morale / high stress echo
            commands.entity(entity).insert(EchoingState {
                is_positive: false,
                ticks_remaining: ECHO_DURATION,
            });
        }
    }
}

/// System to apply the Echo Chamber effect to nearby Pops
pub fn apply_echo_chamber_system(
    mut commands: Commands,
    mut echo_pops: Query<(Entity, &GridPosition, &mut EchoingState)>,
    mut all_pops: Query<(Entity, &GridPosition, &mut Morale, &mut StressTracker), With<Pop>>,
) {
    // Collect active echo chambers
    let mut echo_sources = Vec::new();
    for (entity, pos, mut echo) in echo_pops.iter_mut() {
        if echo.ticks_remaining > 0 {
            echo.ticks_remaining -= 1;
            echo_sources.push((*pos, echo.is_positive));
        } else {
            commands.entity(entity).remove::<EchoingState>();
        }
    }

    if echo_sources.is_empty() {
        return;
    }

    // Apply effects
    for (_, pos, mut morale, mut stress) in all_pops.iter_mut() {
        // Find if this pop is influenced by any echo
        for (echo_pos, is_positive) in &echo_sources {
            if pos.distance_chebyshev(*echo_pos) <= ECHO_RADIUS.try_into().unwrap_or(0) {
                if *is_positive {
                    morale.value = (morale.value + 0.01).min(1.0);
                    stress.accumulated_stress = (stress.accumulated_stress - 0.5).max(0.0);
                } else {
                    morale.value = (morale.value - 0.01).max(0.0);
                    stress.accumulated_stress = (stress.accumulated_stress + 0.5).min(100.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        detect_echo_chamber_system,
        apply_echo_chamber_system.after(detect_echo_chamber_system),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_detect_echo_chamber() {
        let mut world = World::new();

        // High morale pop
        let p1 = world.spawn((
            Pop,
            Morale { value: 0.95, ..Default::default() },
            StressTracker { accumulated_stress: 0.0, ..Default::default() },
        )).id();

        // High stress pop
        let p2 = world.spawn((
            Pop,
            Morale { value: 0.5, ..Default::default() },
            StressTracker { accumulated_stress: 95.0, ..Default::default() },
        )).id();

        // Normal pop
        let p3 = world.spawn((
            Pop,
            Morale { value: 0.5, ..Default::default() },
            StressTracker { accumulated_stress: 0.0, ..Default::default() },
        )).id();

        world.run_system_once(detect_echo_chamber_system).unwrap();

        assert!(world.get::<EchoingState>(p1).is_some());
        assert!(world.get::<EchoingState>(p1).unwrap().is_positive);

        assert!(world.get::<EchoingState>(p2).is_some());
        assert!(!world.get::<EchoingState>(p2).unwrap().is_positive);

        assert!(world.get::<EchoingState>(p3).is_none());
    }

    #[test]
    fn test_apply_echo_chamber() {
        let mut world = World::new();

        // Positive echo source
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            EchoingState { is_positive: true, ticks_remaining: 10 },
        ));

        // Target pop in range
        let target = world.spawn((
            Pop,
            GridPosition { x: 1, y: 1 }, // Distance 1 (<= 3)
            Morale { value: 0.5, ..Default::default() },
            StressTracker { accumulated_stress: 50.0, ..Default::default() },
        )).id();

        // Target pop out of range
        let far_target = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Distance 5 (> 3)
            Morale { value: 0.5, ..Default::default() },
            StressTracker { accumulated_stress: 50.0, ..Default::default() },
        )).id();

        world.run_system_once(apply_echo_chamber_system).unwrap();

        let t_morale = world.get::<Morale>(target).unwrap().value;
        let t_stress = world.get::<StressTracker>(target).unwrap().accumulated_stress;
        assert!(t_morale > 0.5);
        assert!(t_stress < 50.0);

        let ft_morale = world.get::<Morale>(far_target).unwrap().value;
        let ft_stress = world.get::<StressTracker>(far_target).unwrap().accumulated_stress;
        // Strict clippy settings prohibit direct float comparisons
        assert!((ft_morale - 0.5).abs() < f32::EPSILON);
        assert!((ft_stress - 50.0).abs() < f32::EPSILON);
    }
}

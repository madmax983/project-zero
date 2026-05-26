//! # Cryptids
//!
//! Mysterious, unseen creatures that roam the edges of the colony. Cryptids leave behind
//! traces (like strange slimes) that pops can observe, filling them with awe or dread.

use crate::layer1::map::GridPosition;
use crate::layer1::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct Cryptid {
    pub trace_timer: Timer,
}

#[derive(Component)]
pub struct TraceItem {
    pub is_slime: bool,
}

#[derive(Component)]
pub struct PopMood {
    pub awe: f32,
    pub dread: f32,
}

#[derive(Component)]
pub struct VisionRadius(pub f32);

pub fn cryptid_trace_system(
    mut commands: Commands,
    mut query: Query<(&mut Cryptid, &GridPosition)>,
    time: Res<Time>,
) {
    for (mut cryptid, pos) in query.iter_mut() {
        cryptid.trace_timer.tick(time.delta());
        if cryptid.trace_timer.just_finished() {
            commands.spawn((
                TraceItem { is_slime: true },
                GridPosition { x: pos.x, y: pos.y },
            ));
        }
    }
}

pub fn cryptid_observation_system(
    cryptid_query: Query<&GridPosition, With<Cryptid>>,
    mut pop_query: Query<(&mut PopMood, &GridPosition, &VisionRadius), With<Pop>>,
    time: Res<Time>,
) {
    for cryptid_pos in cryptid_query.iter() {
        for (mut mood, pop_pos, vision) in pop_query.iter_mut() {
            let dist = (cryptid_pos
                .x
                .abs_diff(pop_pos.x)
                .saturating_add(cryptid_pos.y.abs_diff(pop_pos.y))) as f32;
            if dist <= vision.0 {
                mood.awe += 1.0 * time.delta_secs();
            }
        }
    }
}

pub fn cryptid_plugin(app: &mut App) {
    app.add_systems(Update, (cryptid_trace_system, cryptid_observation_system));
}



use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
pub fn cryptid_chronicle_bridge_system(
    cryptid_query: Query<&GridPosition, With<Cryptid>>,
    mut pop_query: Query<(&mut PopMood, &GridPosition, &VisionRadius), With<Pop>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    time: Res<Time>,
) {
    for cryptid_pos in cryptid_query.iter() {
        for (mut mood, pop_pos, vision) in pop_query.iter_mut() {
            let dist = (cryptid_pos
                .x
                .abs_diff(pop_pos.x)
                .saturating_add(cryptid_pos.y.abs_diff(pop_pos.y))) as f32;
            if dist <= vision.0 {
                // If a pop has seen the cryptid and has just acquired awe...
                let was_zero = mood.awe == 0.0;
                mood.awe += 1.0 * time.delta_secs();
                if was_zero && mood.awe > 0.0 {
                    chronicle_events.send(AddChronicleEvent {
                        text:
                            "A colonist reported seeing a strange, elusive creature in the wilds."
                                .to_string(),
                        importance: EventImportance::Major,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cryptid_spawns_trace() {
        let mut app = App::new();
        app.add_systems(Update, cryptid_trace_system);
        app.init_resource::<Time>();

        let _cryptid = app
            .world_mut()
            .spawn((
                Cryptid {
                    trace_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        app.update();

        // Fast forward timer
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(6));

        app.update();

        let mut query = app.world_mut().query::<(&TraceItem, &GridPosition)>();
        let mut trace_count = 0;
        for (trace, pos) in query.iter(app.world()) {
            trace_count += 1;
            assert_eq!(pos.x, 10);
            assert_eq!(pos.y, 10);
            assert!(trace.is_slime);
        }

        assert_eq!(trace_count, 1, "Cryptid should drop a trace");
    }

    #[test]
    fn test_pop_observes_cryptid() {
        let mut app = App::new();
        app.add_systems(Update, cryptid_observation_system);
        app.init_resource::<Time>();

        let _cryptid = app
            .world_mut()
            .spawn((
                Cryptid {
                    trace_timer: Timer::default(),
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                GridPosition { x: 2, y: 0 },
                PopMood {
                    awe: 0.0,
                    dread: 0.0,
                },
                VisionRadius(5.0),
            ))
            .id();

        app.update();

        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));

        app.update();

        let mood = app.world().get::<PopMood>(pop).unwrap();
        assert!(
            mood.awe > 0.0 || mood.dread > 0.0,
            "Pop should feel awe or dread after seeing a cryptid"
        );
    }
}
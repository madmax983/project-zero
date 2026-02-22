use bevy_ecs::prelude::*;
use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;
use crate::shared::log::MessageLog;

/// Duration of a festival in ticks.
pub const FESTIVAL_DURATION: u64 = 100;

/// Represents an active festival.
#[derive(Debug, Clone)]
pub struct Festival {
    /// Name of the festival (e.g., "Founding Day Festival").
    pub name: String,
    /// Tick when the original event occurred.
    pub original_event_tick: u64,
    /// Tick when the festival ends.
    pub end_tick: u64,
}

/// Resource to track festival state.
#[derive(Resource, Default)]
pub struct FestivalState {
    /// The currently active festival, if any.
    pub active_festival: Option<Festival>,
}

/// Helper to get current morale modifier from festivals.
#[must_use]
pub const fn get_festival_morale_modifier(state: &FestivalState) -> f32 {
    if state.active_festival.is_some() {
        10.0
    } else {
        0.0
    }
}

/// System to check for festival triggers (anniversaries).
#[allow(clippy::manual_is_multiple_of)] // Standard library method might be unstable
pub fn check_for_festivals_system(
    mut state: ResMut<FestivalState>,
    chronicle: Res<Chronicle>,
    time: Res<SimulationTime>,
    mut log: ResMut<MessageLog>,
) {
    let current_tick = time.tick;

    // Do not override existing festival
    if state.active_festival.is_some() {
        return;
    }

    for event in &chronicle.events {
        if event.importance == EventImportance::Minor {
            continue;
        }

        // Check if today is the anniversary
        // (current_tick - event.tick) % TICKS_PER_YEAR == 0
        // AND current_tick > event.tick (it's in the past)
        if current_tick > event.tick && (current_tick - event.tick) % TICKS_PER_YEAR == 0 {
            // Found one!
            let name = format!("{} Festival", event.text.chars().take(20).collect::<String>().trim());

            state.active_festival = Some(Festival {
                name: name.clone(),
                original_event_tick: event.tick,
                end_tick: current_tick + FESTIVAL_DURATION,
            });

            log.add(format!("Today we celebrate {name}!"));
            break; // Only one festival at a time
        }
    }
}

/// System to manage festival duration and ending.
#[allow(clippy::collapsible_if)] // Avoid unstable `let_chains`
pub fn festival_lifecycle_system(
    mut state: ResMut<FestivalState>,
    time: Res<SimulationTime>,
    mut log: ResMut<MessageLog>,
) {
    if let Some(festival) = &state.active_festival {
        if time.tick >= festival.end_tick {
            log.add(format!("The {} has ended.", festival.name));
            state.active_festival = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::{Chronicle, EventImportance};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_festival_state_default() {
        let state = FestivalState::default();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_check_for_festivals_no_events() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        // Add log resource if needed by system
        world.insert_resource(crate::shared::log::MessageLog::default());

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, check_for_festivals_system).unwrap();

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_check_for_festivals_anniversary() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let mut chronicle = Chronicle::default();
        // Event happened at tick 100 (Year 1)
        chronicle.add_event(100, "Founding Day".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        // Current time: Tick 100 + TICKS_PER_YEAR (Year 2, same day)
        world.insert_resource(SimulationTime {
            tick: 100 + TICKS_PER_YEAR,
            ..Default::default()
        });

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, check_for_festivals_system).unwrap();

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_some(), "Festival should be active on anniversary");
        let festival = state.active_festival.as_ref().unwrap();
        assert!(festival.name.contains("Founding Day"));
        assert_eq!(festival.end_tick, 100 + TICKS_PER_YEAR + FESTIVAL_DURATION);
    }

    #[test]
    fn test_check_for_festivals_ignores_minor_events() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let mut chronicle = Chronicle::default();
        chronicle.add_event(100, "Ate a berry".to_string(), EventImportance::Minor);
        world.insert_resource(chronicle);

        world.insert_resource(SimulationTime {
            tick: 100 + TICKS_PER_YEAR,
            ..Default::default()
        });

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, check_for_festivals_system).unwrap();

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none(), "Minor events should not trigger festivals");
    }

    #[test]
    fn test_festival_ends_after_duration() {
        let mut world = World::new();

        // Festival ending at tick 200
        let festival = Festival {
            name: "Test Festival".to_string(),
            original_event_tick: 0,
            end_tick: 200,
        };

        world.insert_resource(FestivalState {
            active_festival: Some(festival),
        });

        // Current time: 201
        world.insert_resource(SimulationTime { tick: 201, ..Default::default() });
        world.insert_resource(crate::shared::log::MessageLog::default());

        bevy_ecs::system::RunSystemOnce::run_system_once(&mut world, festival_lifecycle_system).unwrap();

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_apply_festival_morale_effect() {
        let mut world = World::new();

        // Active festival
        world.insert_resource(FestivalState {
            active_festival: Some(Festival {
                name: "Party".to_string(),
                original_event_tick: 0,
                end_tick: 1000,
            }),
        });

        let bonus = get_festival_morale_modifier(&world.resource::<FestivalState>());
        assert!(bonus > 0.0);
    }
}

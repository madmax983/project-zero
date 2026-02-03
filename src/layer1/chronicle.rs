use bevy_ecs::prelude::*;
use crate::layer1::building::{Building, BuildingType};
use crate::shared::time::SimulationTime;

/// Importance level for chronicle events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventImportance {
    /// Flavor text or minor notifications.
    Minor,
    /// Standard gameplay events.
    Standard,
    /// Significant achievements or milestones.
    Major,
    /// World-altering events or game start.
    Legendary,
}

/// A single chronicle event.
#[derive(Clone, Debug)]
pub struct ChronicleEvent {
    /// The simulation tick when the event occurred.
    pub tick: u64,
    /// The year when the event occurred (derived from tick).
    pub year: u32,
    /// The event description.
    pub text: String,
    /// The importance level of the event.
    pub importance: EventImportance,
}

/// Chronicle resource - stores the colony's historical record.
#[derive(Resource, Default)]
pub struct Chronicle {
    /// List of events in the chronicle.
    pub events: Vec<ChronicleEvent>,
}

impl Chronicle {
    /// Add a new event to the chronicle.
    pub fn add_event(&mut self, tick: u64, text: String, importance: EventImportance) {
        self.events.push(ChronicleEvent {
            tick,
            year: 1 + u32::try_from(tick / 1000).unwrap_or(u32::MAX), // Rough "year" approximation
            text,
            importance,
        });
    }
}

/// UI state for chronicle window.
#[derive(Resource, Default)]
pub struct ChronicleUiState {
    /// Whether the chronicle window is currently open.
    pub is_open: bool,
}

/// Tracks which building types have been built (for milestones).
#[derive(Resource, Default)]
pub struct BuildingTracker {
    /// Whether housing has been built at least once.
    pub has_built_housing: bool,
    /// Whether a farm has been built at least once.
    pub has_built_farm: bool,
}

/// Creates the initial "colony founded" event.
pub fn initial_chronicle_event(world: &mut World) {
    world.resource_mut::<Chronicle>().add_event(
        0,
        "Colony founded. The journey begins.".to_string(),
        EventImportance::Legendary,
    );
}

/// Checks for building milestones and records them in the chronicle.
pub fn check_milestones_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;

    // Optimization: Check if all milestones are already met
    let (need_housing, need_farm) = {
        let tracker = world.resource::<BuildingTracker>();
        if tracker.has_built_housing && tracker.has_built_farm {
            return;
        }
        (!tracker.has_built_housing, !tracker.has_built_farm)
    };

    let mut found_housing = false;
    let mut found_farm = false;

    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        if need_housing && building.building_type == BuildingType::Housing {
            found_housing = true;
        }
        if need_farm && building.building_type == BuildingType::Farm {
            found_farm = true;
        }
        // Stop early if we found everything we needed
        if (found_housing || !need_housing) && (found_farm || !need_farm) {
            break;
        }
    }

    if found_housing || found_farm {
        world.resource_scope(|world, mut tracker: Mut<BuildingTracker>| {
            let mut chronicle = world.resource_mut::<Chronicle>();

            if found_housing && !tracker.has_built_housing {
                tracker.has_built_housing = true;
                chronicle.add_event(
                    current_tick,
                    "First Housing constructed. A shelter from the void.".to_string(),
                    EventImportance::Major,
                );
            }
            if found_farm && !tracker.has_built_farm {
                tracker.has_built_farm = true;
                chronicle.add_event(
                    current_tick,
                    "First Farm operational. We shall not starve.".to_string(),
                    EventImportance::Major,
                );
            }
        });
    }
}

/// Format the importance prefix for display.
#[must_use]
pub const fn format_event_prefix(importance: EventImportance) -> &'static str {
    match importance {
        EventImportance::Legendary => "!!!",
        EventImportance::Major => "!",
        _ => " ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::GridPosition;

    #[test]
    fn test_event_importance_variants() {
        // Just verify all variants exist
        let _ = EventImportance::Minor;
        let _ = EventImportance::Standard;
        let _ = EventImportance::Major;
        let _ = EventImportance::Legendary;
    }

    #[test]
    fn test_chronicle_event_creation() {
        let event = ChronicleEvent {
            tick: 100,
            year: 1,
            text: "Test event".to_string(),
            importance: EventImportance::Standard,
        };

        assert_eq!(event.tick, 100);
        assert_eq!(event.year, 1);
        assert_eq!(event.text, "Test event");
        assert_eq!(event.importance, EventImportance::Standard);
    }

    #[test]
    fn test_chronicle_default() {
        let chronicle = Chronicle::default();
        assert!(chronicle.events.is_empty());
    }

    #[test]
    fn test_chronicle_add_event() {
        let mut chronicle = Chronicle::default();

        chronicle.add_event(0, "First event".to_string(), EventImportance::Legendary);
        chronicle.add_event(100, "Second event".to_string(), EventImportance::Standard);

        assert_eq!(chronicle.events.len(), 2);
        assert_eq!(chronicle.events[0].text, "First event");
        assert_eq!(chronicle.events[1].text, "Second event");
    }

    #[test]
    fn test_chronicle_year_calculation() {
        let mut chronicle = Chronicle::default();

        chronicle.add_event(0, "Year 1".to_string(), EventImportance::Standard);
        chronicle.add_event(1000, "Year 2".to_string(), EventImportance::Standard);
        chronicle.add_event(5000, "Year 6".to_string(), EventImportance::Standard);

        assert_eq!(chronicle.events[0].year, 1);
        assert_eq!(chronicle.events[1].year, 2);
        assert_eq!(chronicle.events[2].year, 6);
    }

    #[test]
    fn test_chronicle_ui_state_default() {
        let ui_state = ChronicleUiState::default();
        assert!(!ui_state.is_open);
    }

    #[test]
    fn test_chronicle_ui_state_toggle() {
        let mut ui_state = ChronicleUiState::default();
        assert!(!ui_state.is_open);

        ui_state.is_open = true;
        assert!(ui_state.is_open);

        ui_state.is_open = !ui_state.is_open;
        assert!(!ui_state.is_open);
    }

    #[test]
    fn test_building_tracker_default() {
        let tracker = BuildingTracker::default();
        assert!(!tracker.has_built_housing);
        assert!(!tracker.has_built_farm);
    }

    #[test]
    fn test_check_milestones_system_housing() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place housing
        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 5, y: 5 },
        ));

        check_milestones_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Housing"));

        let tracker = world.resource::<BuildingTracker>();
        assert!(tracker.has_built_housing);
    }

    #[test]
    fn test_check_milestones_system_farm() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place farm
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));

        check_milestones_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Farm"));

        let tracker = world.resource::<BuildingTracker>();
        assert!(tracker.has_built_farm);
    }

    #[test]
    fn test_check_milestones_system_only_once() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());
        world.insert_resource(BuildingTracker::default());
        world.insert_resource(SimulationTime::default());

        // Place two farms
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 5, y: 5 },
        ));
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 10, y: 10 },
        ));

        check_milestones_system(&mut world);
        check_milestones_system(&mut world); // Run twice

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1, "Should only record first farm");
    }

    #[test]
    fn test_initial_chronicle_event() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default());

        initial_chronicle_event(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("founded"));
        assert_eq!(chronicle.events[0].importance, EventImportance::Legendary);
    }

    #[test]
    fn test_format_event_prefix() {
        assert_eq!(format_event_prefix(EventImportance::Legendary), "!!!");
        assert_eq!(format_event_prefix(EventImportance::Major), "!");
        assert_eq!(format_event_prefix(EventImportance::Standard), " ");
        assert_eq!(format_event_prefix(EventImportance::Minor), " ");
    }
}

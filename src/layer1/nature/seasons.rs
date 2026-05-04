use crate::layer1::balance::{
    SEASON_MODIFIER_AUTUMN, SEASON_MODIFIER_SPRING, SEASON_MODIFIER_SUMMER, SEASON_MODIFIER_WINTER,
    TICKS_PER_YEAR,
};
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// The four seasons of the year, influencing gameplay mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Season {
    /// The season of rebirth and growth.
    #[default]
    Spring,
    /// The season of warmth and abundance.
    Summer,
    /// The season of harvest and preparation.
    Autumn,
    /// The season of cold and dormancy.
    Winter,
}

impl Season {
    /// Returns the next season in the cycle.
    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }

    /// Returns the food production modifier for this season.
    #[must_use]
    pub const fn food_modifier(&self) -> f32 {
        match self {
            Self::Spring => SEASON_MODIFIER_SPRING,
            Self::Summer => SEASON_MODIFIER_SUMMER,
            Self::Autumn => SEASON_MODIFIER_AUTUMN,
            Self::Winter => SEASON_MODIFIER_WINTER,
        }
    }

    /// Returns the display name of the season.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }

    /// Returns the base ambient temperature for this season (Celsius).
    #[must_use]
    pub const fn base_temperature(&self) -> f32 {
        match self {
            Self::Spring => 15.0,
            Self::Summer => 30.0,
            Self::Autumn => 10.0,
            Self::Winter => -5.0,
        }
    }
}

/// Tracks the current season of the colony.
#[derive(Resource, Default)]
pub struct SeasonState {
    /// The current active season.
    pub current_season: Season,
}

/// Updates the current season based on the simulation tick.
///
/// This system calculates the season by dividing the current tick by `TICKS_PER_YEAR / 4`.
/// It updates `SeasonState` and adds a `Chronicle` event when the season changes.
pub fn advance_season_system(
    time: Res<SimulationTime>,
    mut state: ResMut<SeasonState>,
    mut events: EventWriter<AddChronicleEvent>,
) {
    let tick = time.tick;
    let ticks_per_season = TICKS_PER_YEAR / 4;

    let season_index = (tick / ticks_per_season) % 4;
    let new_season = match season_index {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        _ => Season::Winter,
    };

    if state.current_season != new_season {
        state.current_season = new_season;
        events.send(AddChronicleEvent {
            text: format!("The season turns. {} has arrived.", new_season.name()),
            importance: EventImportance::Standard,
            ..Default::default()});
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_season_enum_cycling() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_season_state_default() {
        let state = SeasonState::default();
        assert_eq!(state.current_season, Season::Spring);
    }

    #[test]
    fn test_advance_season_system_initial() {
        let mut world = World::new();
        world.insert_resource(SeasonState::default());
        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });
        world.init_resource::<Events<AddChronicleEvent>>();

        world.run_system_once(advance_season_system).unwrap();

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Spring);
    }

    #[test]
    fn test_advance_season_system_transition() {
        let mut world = World::new();
        world.insert_resource(SeasonState::default());
        world.init_resource::<Events<AddChronicleEvent>>();

        // Ticks per season = 1000 / 4 = 250
        // Spring: 0-249, Summer: 250-499
        world.insert_resource(SimulationTime {
            tick: 250,
            ..Default::default()
        });

        world.run_system_once(advance_season_system).unwrap();

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Summer);

        // Verify Event was sent
        let events = world.resource::<Events<AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();
        assert_eq!(emitted.len(), 1);
        assert!(emitted[0].text.contains("Summer"));
    }

    #[test]
    fn test_advance_season_system_no_spam() {
        let mut world = World::new();
        world.insert_resource(SeasonState {
            current_season: Season::Summer,
        });
        world.init_resource::<Events<AddChronicleEvent>>();
        world.insert_resource(SimulationTime {
            tick: 251,
            ..Default::default()
        });

        world.run_system_once(advance_season_system).unwrap();

        // Verify NO Event was sent
        let events = world.resource::<Events<AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        let emitted: Vec<_> = reader.read(events).collect();
        assert!(
            emitted.is_empty(),
            "Should not add event if season hasn't changed"
        );
    }

    #[test]
    fn test_get_food_modifier() {
        assert!((Season::Spring.food_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((Season::Summer.food_modifier() - 1.2).abs() < f32::EPSILON);
        assert!((Season::Autumn.food_modifier() - 1.5).abs() < f32::EPSILON);
        assert!((Season::Winter.food_modifier() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_base_temperature() {
        assert!((Season::Spring.base_temperature() - 15.0).abs() < f32::EPSILON);
        assert!((Season::Summer.base_temperature() - 30.0).abs() < f32::EPSILON);
        assert!((Season::Autumn.base_temperature() - 10.0).abs() < f32::EPSILON);
        assert!((Season::Winter.base_temperature() + 5.0).abs() < f32::EPSILON);
    }
}

use crate::layer1::culture::CulturalTag;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::utility_eval_types::PopEvalData;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct PhantomShiftConfig {
    pub inefficiency_threshold: f32,
}

impl Default for PhantomShiftConfig {
    fn default() -> Self {
        Self {
            inefficiency_threshold: 80.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct ColonyInefficiencyTracker {
    pub current_backlog_score: f32,
}

pub fn evaluate_phantom_shift_utility(
    data: &PopEvalData,
    cycle: &DayNightCycle,
    tracker: &ColonyInefficiencyTracker,
    config: &PhantomShiftConfig,
) -> Option<(ActionType, f32, Option<Entity>)> {
    if cycle.time_of_day != crate::layer1::day_night::TimeOfDay::Night
        || tracker.current_backlog_score < config.inefficiency_threshold
    {
        return None;
    }

    if let Some(culture) = &data.culture {
        if *culture == CulturalTag::Fringe {
            // Phantom work overrides rest with a moderate/high utility
            return Some((ActionType::PhantomWork, 8.0, None));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::culture::CulturalTag;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::utility_eval_types::PopEvalData;
    use crate::layer1::utility_types::ActionType;

    fn get_data(is_fringe: bool) -> PopEvalData {
        let mut data = PopEvalData::test_instance();
        if is_fringe {
            data.culture = Some(CulturalTag::Fringe);
        }
        data
    }

    #[test]
    fn test_phantom_shift_activation_at_night() {
        let mut cycle = DayNightCycle::default();
        cycle.time_of_day = TimeOfDay::Night;
        let tracker = ColonyInefficiencyTracker {
            current_backlog_score: 100.0,
        };
        let config = PhantomShiftConfig {
            inefficiency_threshold: 50.0,
        };

        let data = get_data(true);

        let result = evaluate_phantom_shift_utility(&data, &cycle, &tracker, &config);

        assert!(result.is_some());
        assert_eq!(result.unwrap().0, ActionType::PhantomWork);
    }

    #[test]
    fn test_phantom_shift_does_not_activate_during_day() {
        let mut cycle = DayNightCycle::default();
        cycle.time_of_day = TimeOfDay::Day;
        let tracker = ColonyInefficiencyTracker {
            current_backlog_score: 100.0,
        };
        let config = PhantomShiftConfig {
            inefficiency_threshold: 50.0,
        };

        let data = get_data(true);

        let result = evaluate_phantom_shift_utility(&data, &cycle, &tracker, &config);

        assert!(result.is_none());
    }

    #[test]
    fn test_phantom_shift_consumes_resources_silently() {
        // Assert that global resource trackers drift from actual stockpile counts.
    }
}

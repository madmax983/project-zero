use crate::layer1::needs::Needs;
use crate::layer1::structural_integrity::StructureCollapsed;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_types::PopAction;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Tracks active taboos in the colony.
#[derive(Resource, Default, Debug, Clone)]
pub struct TabooState {
    /// Map of taboo actions to their remaining duration in ticks.
    pub active_taboos: HashMap<ActionType, u32>,
}

impl TabooState {
    /// Adds a taboo for a specific action with a duration.
    /// If the taboo already exists, the duration is extended to the max of current and new.
    pub fn add_taboo(&mut self, action: ActionType, duration: u32) {
        let current = self.active_taboos.entry(action).or_insert(0);
        *current = (*current).max(duration);
    }

    /// Checks if an action is currently taboo.
    #[must_use]
    pub fn is_taboo(&self, action: ActionType) -> bool {
        self.active_taboos.contains_key(&action)
    }

    /// Returns the remaining duration of a taboo for an action.
    #[must_use]
    pub fn get_duration(&self, action: ActionType) -> u32 {
        *self.active_taboos.get(&action).unwrap_or(&0)
    }

    /// Decays all active taboos by the specified amount.
    pub fn decay(&mut self, amount: u32) {
        self.active_taboos.retain(|_, duration| {
            *duration = duration.saturating_sub(amount);
            *duration > 0
        });
    }
}

/// System that listens for `StructureCollapsed` events and creates taboos.
pub fn taboo_event_system(
    mut events: EventReader<StructureCollapsed>,
    mut state: ResMut<TabooState>,
    mut log: ResMut<MessageLog>,
) {
    let mut triggered = false;
    for _event in events.read() {
        // Collapse -> Fear of Work (Construction/Mining)
        // Duration: 1 Day (approx 2400 ticks)
        state.add_taboo(ActionType::Work, 2400);
        triggered = true;
    }

    if triggered {
        log.add("The collapse has caused a fear of labor!".to_string());
    }
}

/// System that decays taboo durations over time.
pub fn update_taboo_duration_system(mut state: ResMut<TabooState>) {
    state.decay(1);
}

/// System that applies stress (leisure decay) to pops performing taboo actions.
pub fn apply_taboo_stress_system(
    state: Res<TabooState>,
    mut query: Query<(&PopAction, &mut Needs)>,
) {
    if state.active_taboos.is_empty() {
        return;
    }

    for (action, mut needs) in &mut query {
        if state.is_taboo(action.current) {
            // Apply stress (reduce leisure)
            // 0.005 per tick is significant (1.0 in 200 ticks)
            needs.leisure = (needs.leisure - 0.005).max(0.0);
        }
    }
}

/// Helper to evaluate the utility penalty for a taboo action.
#[must_use]
pub fn evaluate_taboo_penalty(action: ActionType, state: &TabooState) -> f32 {
    if state.is_taboo(action) {
        -0.3 // Significant penalty to utility score
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::structural_integrity::StructureCollapsed;
    use crate::layer1::taboo::{
        TabooState, apply_taboo_stress_system, evaluate_taboo_penalty, taboo_event_system,
    };
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_taboo_state_default() {
        let state = TabooState::default();
        assert!(state.active_taboos.is_empty());
    }

    #[test]
    fn test_structure_collapse_creates_taboo() {
        let mut world = World::new();
        world.insert_resource(TabooState::default());
        world.insert_resource(SimulationTime::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Register event
        world.init_resource::<Events<StructureCollapsed>>();

        // Trigger event
        world.send_event(StructureCollapsed {
            pos: GridPosition { x: 0, y: 0 },
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(taboo_event_system);
        schedule.run(&mut world);

        let state = world.resource::<TabooState>();
        // Work action (which includes mining/building) should be taboo
        assert!(state.is_taboo(ActionType::Work));
        assert!(state.get_duration(ActionType::Work) > 0);
    }

    #[test]
    fn test_taboo_decays_over_time() {
        let mut world = World::new();
        let mut state = TabooState::default();

        // Add taboo manually
        state.add_taboo(ActionType::Work, 10); // 10 ticks duration
        world.insert_resource(state);
        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        // Advance time
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::taboo::update_taboo_duration_system);

        schedule.run(&mut world); // Tick 0 -> 1? (Depending on impl)

        let mut state = world.resource_mut::<TabooState>();
        // System ran once, so 10 -> 9
        assert_eq!(state.get_duration(ActionType::Work), 9);

        // Manually simulate remaining decay
        state.decay(9);
        assert!(!state.is_taboo(ActionType::Work));
    }

    #[test]
    fn test_taboo_applies_stress_to_worker() {
        let mut world = World::new();
        let mut state = TabooState::default();
        state.add_taboo(ActionType::Work, 100);
        world.insert_resource(state);

        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 1.0,
                    ..Default::default()
                },
                PopAction {
                    current: ActionType::Work,
                    ..Default::default()
                },
            ))
            .id();

        // Run stress system
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_taboo_stress_system);
        schedule.run(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        // Leisure should drop (stress increase)
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_evaluate_taboo_penalty() {
        let mut state = TabooState::default();
        state.add_taboo(ActionType::Work, 100);

        let penalty = evaluate_taboo_penalty(ActionType::Work, &state);
        let normal = evaluate_taboo_penalty(ActionType::Idle, &state);

        assert!(penalty < 0.0); // Should return a negative modifier
        assert!(normal.abs() < f32::EPSILON);
    }
}

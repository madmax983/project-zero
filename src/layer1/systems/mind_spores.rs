use crate::layer1::mind_spores::{
    process_mind_spore_infection_system, process_sabotage_events_system,
    trigger_symbiont_sabotage_system,
};
use crate::layer1::systems::Layer1SystemSet;
use bevy_ecs::prelude::*;

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            process_mind_spore_infection_system,
            trigger_symbiont_sabotage_system,
            process_sabotage_events_system,
        )
            .in_set(Layer1SystemSet::Execution),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let mut schedule = Schedule::default();
        register(&mut schedule);
    }
}

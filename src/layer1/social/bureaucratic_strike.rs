use bevy_ecs::prelude::*;
use crate::layer1::designation::DesignationType;
use crate::layer1::morale::Morale;

#[derive(Resource, Clone, Debug)]
pub struct RedTapeEvent {
    pub active: bool,
    pub severity: u32,
}

#[derive(Component, Clone, Debug)]
pub struct AdminPop;

#[derive(Component, Clone, Debug)]
pub struct RedTapeCost {
    pub current_cost: f32,
    pub max_cost: f32,
    pub designation_type: DesignationType,
    pub penalized: bool,
}

#[derive(Component, Clone, Debug)]
pub struct PaperworkDelay {
    pub ticks_remaining: u32,
}

use crate::layer1::utility_types::AssignmentType;
use crate::layer1::actions::AssignedTo;

pub fn update_bureaucratic_strike_status_system(
    mut red_tape: ResMut<RedTapeEvent>,
    query: Query<(&Morale, &AssignedTo)>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut total_morale = 0.0;
    let mut admin_count = 0;

    for (morale, assigned_to) in query.iter() {
        if assigned_to.assignment_type == AssignmentType::Administrator {
            total_morale += morale.value;
            admin_count += 1;
        }
    }

    if admin_count > 0 {
        let avg_morale = total_morale / admin_count as f32;
        if avg_morale < 30.0 && !red_tape.active { // Low morale threshold
            red_tape.active = true;
            red_tape.severity = 10;
            chronicle_events.send(AddChronicleEvent {
                text: "The administrators have gone on a bureaucratic strike!".to_string(),
                importance: crate::layer1::chronicle::EventImportance::Major,
            });
        } else if avg_morale > 60.0 && red_tape.active {
            red_tape.active = false;
            chronicle_events.send(AddChronicleEvent {
                text: "The administrators have ended their bureaucratic strike.".to_string(),
                importance: crate::layer1::chronicle::EventImportance::Major,
            });
        }
    }
}

use crate::layer1::chronicle::{AddChronicleEvent};

pub fn process_red_tape_designations_system(
    red_tape: Res<RedTapeEvent>,
    mut designation_query: Query<&mut RedTapeCost>,
    mut delay_query: Query<&mut PaperworkDelay>,
) {
    if red_tape.active {
        for mut cost in designation_query.iter_mut() {
            if !cost.penalized {
                if cost.max_cost == 0.0 {
                    cost.max_cost = 100.0;
                }
                cost.max_cost *= red_tape.severity as f32;
                cost.penalized = true;
            }
        }

        // In tests, jobs with ticks_remaining = 0 are spawned and initialized here.
        // In the real game, utility_ai.rs initializes them to 50 * severity directly.
        for mut delay in delay_query.iter_mut() {
            if delay.ticks_remaining == 0 {
                delay.ticks_remaining = 50 * red_tape.severity;
            }
        }
    }
}

pub fn tick_paperwork_delay_system(
    mut commands: Commands,
    mut delay_query: Query<(Entity, &mut PaperworkDelay)>,
) {
    for (entity, mut delay) in delay_query.iter_mut() {
        if delay.ticks_remaining > 0 {
            delay.ticks_remaining -= 1;
        }

        if delay.ticks_remaining == 0 {
            commands.entity(entity).remove::<PaperworkDelay>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::designation::DesignationType;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            update_bureaucratic_strike_status_system,
            process_red_tape_designations_system,
            tick_paperwork_delay_system,
        ));
        app.insert_resource(RedTapeEvent { active: false, severity: 1 });
        app
    }

    #[test]
    fn test_red_tape_increases_designation_cost() {
        let mut app = setup_app();
        app.add_event::<AddChronicleEvent>();

        let designation = app.world_mut().spawn(RedTapeCost { current_cost: 0.0, max_cost: 100.0, designation_type: DesignationType::Mine, penalized: false }).id();

        app.world_mut().resource_mut::<RedTapeEvent>().active = true;
        app.world_mut().resource_mut::<RedTapeEvent>().severity = 10;

        app.update();

        // Cost should be multiplied by severity
        let cost = app.world().get::<RedTapeCost>(designation).unwrap();
        assert_eq!(cost.max_cost, 1000.0);
    }

    #[test]
    fn test_red_tape_prevents_instant_job_reassignment() {
        let mut app = setup_app();
        app.add_event::<AddChronicleEvent>();

        // This simulates a pop trying to take a job but being delayed by paperwork
        let job = app.world_mut().spawn(PaperworkDelay { ticks_remaining: 1 }).id();

        app.update(); // Tick remaining hits 0, which also triggers `commands.remove` on the exact same tick with the updated logic

        app.cleanup(); // Force a full flush to ensure the entity drops the component

        // Component should be removed because it hits 0 and `commands.entity(entity).remove` is called.
        assert!(app.world().get::<PaperworkDelay>(job).is_none());
    }

    #[test]
    fn test_strike_ends_when_admin_morale_recovers() {
        let mut app = setup_app();
        app.add_event::<AddChronicleEvent>();

        // Add a happy Admin Pop
        app.world_mut().spawn((
            AssignedTo { entity: Entity::PLACEHOLDER, assignment_type: AssignmentType::Administrator },
            Morale { value: 90.0, modifiers: Vec::new() }, // High morale
        ));

        app.world_mut().resource_mut::<RedTapeEvent>().active = true;

        app.update();

        // The active state should flip back to false
        let event = app.world().resource::<RedTapeEvent>();
        assert_eq!(event.active, false);
    }
}

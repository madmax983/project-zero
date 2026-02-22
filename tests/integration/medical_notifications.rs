use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::integration::{
    hospitalization_notification_system, medical_treatment_notification_system,
    pop_death_notification_system,
};
use scale::layer1::medical::PatientTreated;
use scale::layer1::notifications::{NotificationQueue, NotificationSeverity};
use scale::layer1::pop::{Pop, PopDied, PopName};
use scale::layer1::utility_types::{ActionType, PopAction};
use scale::shared::time::SimulationTime;

fn setup() -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.init_resource::<NotificationQueue>();
    world.init_resource::<SimulationTime>();
    world.init_resource::<Events<PatientTreated>>();
    world.init_resource::<Events<PopDied>>();
    world
}

#[test]
fn test_treatment_triggers_notification() {
    let mut world = setup();

    let patient = world.spawn((Pop, PopName("TestPatient".to_string()))).id();

    // Send event
    world.send_event(PatientTreated {
        patient,
        hospital: Entity::PLACEHOLDER,
        amount: 10.0,
    });

    // Run system
    world
        .run_system_once(medical_treatment_notification_system)
        .unwrap();

    // Verify
    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should have 1 notification");
    let notif = &queue.active[0];
    assert!(notif.text.contains("TestPatient"));
    // Format is {:.1} -> 10.0
    assert!(notif.text.contains("10.0 HP"));
    assert_eq!(notif.severity, NotificationSeverity::Success);
}

#[test]
fn test_hospitalization_triggers_notification() {
    let mut world = setup();

    // Spawn pop with Initial Action
    let pop = world
        .spawn((
            Pop,
            PopName("SicklySid".to_string()),
            PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },
        ))
        .id();

    // Change action to SeekMedicalCare
    if let Some(mut action) = world.get_mut::<PopAction>(pop) {
        action.current = ActionType::SeekMedicalCare;
    }

    // Run system (needs Changed<PopAction> trigger)
    // RunSystemOnce executes once, effectively handling the change if it happened "recently" enough for query
    // In Bevy `RunSystemOnce`, change detection works if changes happened since last run of THIS system or since creation.
    // Since we just changed it, it should trigger.
    world
        .run_system_once(hospitalization_notification_system)
        .unwrap();

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should notify on hospitalization");
    let notif = &queue.active[0];
    assert!(notif.text.contains("SicklySid"));
    assert!(notif.text.contains("hospitalized"));
    assert_eq!(notif.severity, NotificationSeverity::Warning);
}

#[test]
fn test_hospitalization_ignores_other_actions() {
    let mut world = setup();

    let pop = world
        .spawn((
            Pop,
            PopName("WorkerWill".to_string()),
            PopAction {
                current: ActionType::Idle,
                ..Default::default()
            },
        ))
        .id();

    // Change to Work
    if let Some(mut action) = world.get_mut::<PopAction>(pop) {
        action.current = ActionType::Work;
    }

    world
        .run_system_once(hospitalization_notification_system)
        .unwrap();

    let queue = world.resource::<NotificationQueue>();
    assert!(queue.active.is_empty(), "Should NOT notify for Work");
}

#[test]
fn test_death_triggers_notification() {
    let mut world = setup();

    let pop = world.spawn((Pop, PopName("DeadDave".to_string()))).id();

    world.send_event(PopDied {
        entity: pop,
        name: "DeadDave".to_string(),
        tick: 100,
        reason: "Starvation".to_string(),
    });

    world
        .run_system_once(pop_death_notification_system)
        .unwrap();

    let queue = world.resource::<NotificationQueue>();
    assert_eq!(queue.active.len(), 1, "Should notify on death");
    let notif = &queue.active[0];
    assert!(notif.text.contains("DeadDave"));
    assert!(notif.text.contains("Starvation"));
    assert_eq!(notif.severity, NotificationSeverity::Error); // Or Critical if available, likely Error/Warning
}

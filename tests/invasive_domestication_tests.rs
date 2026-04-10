use bevy::prelude::*;
use scale::layer1::invasive_domestication::*;
use scale::layer1::needs::Needs;
use scale::layer1::mind::utility_types::{ActionType, PopAction};
use scale::layer1::pop::Pop;

#[test]
fn test_bliss_weed_boosts_needs_and_causes_dependency() {
    let mut app = App::new();
    // Register systems
    app.add_systems(Update, apply_bliss_weed_aura);

    // Spawn BlissWeed
    let weed_entity = app.world_mut().spawn((BlissWeed { aura_radius: 5.0, dependency_threshold: 10.0 }, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    // Spawn Pop nearby
    let pop_entity = app.world_mut().spawn((
        Pop,
        Needs { leisure: 0.0, ..Default::default() },
        BlissExposure { accumulated: 0.0 },
        Transform::from_xyz(1.0, 0.0, 0.0)
    )).id();

    // Run systems to accumulate exposure
    for _ in 0..15 {
        app.update();
    }

    // Needs should be fulfilled, and Dependency trait acquired
    assert!(app.world().get::<Needs>(pop_entity).unwrap().leisure > 50.0);
    assert!(app.world().get::<DependencyTrait>(pop_entity).is_some());
}

#[test]
fn test_dependency_blocks_work_actions() {
    let mut app = App::new();
    app.add_systems(Update, dependency_work_blocker_system);

    // Pop with Dependency trait
    let pop_entity = app.world_mut().spawn((
        Pop,
        DependencyTrait,
        PopAction { current: ActionType::Work, ..Default::default() }
    )).id();

    app.update();

    // Action should be changed to Basking or Idle
    let action = app.world().get::<PopAction>(pop_entity).unwrap();
    assert_ne!(action.current, ActionType::Work);
}

#[test]
fn test_dependency_defends_flora() {
    let mut app = App::new();
    app.add_systems(Update, defend_bliss_weed_system);

    let weed_entity = app.world_mut().spawn((BlissWeed { aura_radius: 5.0, dependency_threshold: 10.0 }, Transform::from_xyz(0.0, 0.0, 0.0))).id();

    let pop_entity = app.world_mut().spawn((
        Pop,
        DependencyTrait,
        Transform::from_xyz(1.0, 0.0, 0.0),
        CurrentTarget { entity: None }
    )).id();

    let hostile_worker = app.world_mut().spawn((
        Pop,
        PopAction { current: ActionType::Vandalize, ..Default::default() },
        CurrentTarget { entity: Some(weed_entity) },
        Transform::from_xyz(2.0, 0.0, 0.0)
    )).id();

    app.update();

    // Dependent Pop should target the hostile worker
    let target = app.world().get::<CurrentTarget>(pop_entity).unwrap();
    assert_eq!(target.entity, Some(hostile_worker));
}

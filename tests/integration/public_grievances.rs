use bevy::prelude::*;
use scale::layer1::social::grievances::{
    apply_grievance_system, post_grievance_system, BulletinBoard, PostGrievanceEvent, SocialStanding,
};
use scale::layer1::needs::Needs;
use scale::layer1::social::Relationships;
use scale::layer1::pop::Pop;
use scale::shared::time::SimulationTime;
use scale::layer1::map::GridPosition;

#[test]
fn test_post_grievance_emits_event_and_updates_standing() {
    let mut app = App::new();
    app.add_event::<PostGrievanceEvent>();
    app.insert_resource(SimulationTime {
        tick: 1000,
        ..Default::default()
    });

    app.add_systems(Update, (post_grievance_system, apply_grievance_system).chain());

    let _board_ent = app.world_mut()
        .spawn((BulletinBoard::default(), GridPosition { x: 0, y: 0 }))
        .id();

    let target_pop = app.world_mut().spawn((
        Pop,
        SocialStanding { value: 50.0 },
    )).id();

    // Create a pop with negative relationships and low morale to trigger a negative post
    let mut rel = Relationships::default();
    rel.affinities.insert(target_pop, -10.0);

    let _poster_pop = app.world_mut().spawn((
        Pop,
        Needs {
            hunger: 0.1,
            rest: 0.1,
            leisure: 0.1,
            hygiene: 0.1,
        },
        rel,
        GridPosition { x: 0, y: 0 },
        SocialStanding { value: 50.0 },
    )).id();

    // Force post by running it enough times to bypass 1% probability
    for _ in 0..200 {
        app.update();
    }

    let target_standing = app.world().get::<SocialStanding>(target_pop).unwrap().value;
    assert!(target_standing < 50.0, "Target standing should decrease after negative grievance!");
}

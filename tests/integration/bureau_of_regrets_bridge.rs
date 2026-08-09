use scale::layer1::entities::pop::PopDied;
use scale::layer1::social::bureau_of_regrets::AtrocityScore;
use scale::layer1::core::integration::starvation_atrocity_bridge;
use bevy_ecs::prelude::*;

#[test]
fn starvation_increases_atrocity_score() {
    let mut app = bevy::prelude::App::new();
    app.add_event::<PopDied>();
    app.init_resource::<AtrocityScore>();
    app.add_systems(bevy::prelude::Update, starvation_atrocity_bridge);

    app.world_mut().send_event(PopDied {
        entity: Entity::PLACEHOLDER,
        name: "Test Pop".to_string(),
        tick: 1,
        reason: "Starvation".to_string(),
    });

    app.update();

    let atrocity = app.world().resource::<AtrocityScore>();
    assert_eq!(atrocity.score, 10.0);
}

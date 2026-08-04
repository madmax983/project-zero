use bevy_ecs::prelude::*;
use scale::layer1::entities::pop::{Pop, PopBundle};
use scale::layer1::social::sartorial_rebellion::Appearance;

#[test]
fn test_spawned_pops_have_appearance() {
    let mut world = World::new();
    world.spawn(PopBundle::random(0, 0, &mut rand::thread_rng()));

    let mut query = world.query_filtered::<&Appearance, With<Pop>>();
    let count = query.iter(&world).count();
    assert_eq!(count, 1, "Pops should spawn with an Appearance component");
}

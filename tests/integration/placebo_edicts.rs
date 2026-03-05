use bevy_ecs::prelude::*;
use scale::layer1::edicts::{ColonyPolicies, Policy};
use scale::layer1::integration::issue_placebo_from_edict_system;
use scale::layer1::social::placebo::{ActivePlacebo, PlaceboProtocol};

#[test]
fn test_placebo_issued_from_edict() {
    let mut world = World::new();
    let mut policies = ColonyPolicies::default();
    policies.toggle(Policy::Placebo(PlaceboProtocol::FakeReinforcements));
    world.insert_resource(policies);

    let mut schedule = Schedule::default();
    schedule.add_systems(issue_placebo_from_edict_system);
    schedule.run(&mut world);

    // We expect an ActivePlacebo to be spawned
    let mut count = 0;
    for placebo in world.query::<&ActivePlacebo>().iter(&world) {
        assert_eq!(placebo.protocol, PlaceboProtocol::FakeReinforcements);
        count += 1;
    }
    assert_eq!(count, 1, "Should spawn one ActivePlacebo");
}

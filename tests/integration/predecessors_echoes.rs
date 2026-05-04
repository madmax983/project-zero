use scale::layer1::core::integration::predecessor_weather_array_bridge_system;
use scale::layer2::integration::predecessor_orbital_shield_bridge_system;
use scale::layer1::nature::seasons::{Season, SeasonState};
use scale::layer1::predecessors::{PredecessorOrbitalShield, PredecessorWeatherArray};
use scale::layer2::fleet::{Fleet, FleetOrder};
use scale::layer2::generation::ColonyLocation;
use bevy_ecs::prelude::*;

#[test]
fn test_predecessor_weather_array_forces_spring() {
    let mut world = World::new();

    // Start in Winter
    world.insert_resource(SeasonState {
        current_season: Season::Winter,
    });

    // Spawn the array
    world.spawn(PredecessorWeatherArray);

    let mut schedule = Schedule::default();
    schedule.add_systems(predecessor_weather_array_bridge_system);
    schedule.run(&mut world);

    // Verify forced to Spring
    let season_state = world.resource::<SeasonState>();
    assert_eq!(season_state.current_season, Season::Spring, "Season should be forced to Spring");
}

#[test]
fn test_predecessor_shield_blocks_fleets() {
    let mut world = World::new();

    // Spawn colony location
    let colony = world.spawn(ColonyLocation).id();

    // Spawn orbital shield
    world.spawn(PredecessorOrbitalShield);

    // Spawn a fleet trying to move to the colony
    let fleet = world.spawn((
        Fleet,
        FleetOrder::MoveTo(colony),
    )).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(predecessor_orbital_shield_bridge_system);
    schedule.run(&mut world);

    // Verify order was removed
    assert!(world.get::<FleetOrder>(fleet).is_none(), "FleetOrder should be removed due to shield");
}

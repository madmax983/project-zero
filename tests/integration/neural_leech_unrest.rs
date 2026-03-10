use bevy::prelude::*;
use scale::layer1::integration::apply_neural_shock_system;
use scale::layer1::tech::neural_leech::NeuralShock;
use scale::layer1::unrest::{MentalBreakType, MentalState};

#[test]
fn test_neural_shock_causes_breakdown() {
    let mut world = World::new();
    let pop = world.spawn((NeuralShock, MentalState::Normal)).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(apply_neural_shock_system);
    schedule.run(&mut world);

    let state = world.get::<MentalState>(pop).unwrap();
    assert_eq!(*state, MentalState::Broken(MentalBreakType::Daze));
    assert!(world.get::<NeuralShock>(pop).is_none());
}

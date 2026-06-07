use bevy::prelude::*;
use scale::layer1::infrastructure::subconscious_grid::{
    Machine, SubconsciousGridEffect
};
use scale::layer1::core::integration::{apply_subconscious_grid_machine_effects_system, BaseMachineStats};

#[test]
fn test_subconscious_grid_machine_effects_bridge() {
    let mut app = App::new();

    app.add_systems(Update, apply_subconscious_grid_machine_effects_system);

    let machine_entity = app.world_mut().spawn((
        Machine {
            efficiency: 1.0,
            burnout_risk: 0.05,
        },
        SubconsciousGridEffect {
            efficiency_multiplier: 1.5,
            burnout_risk_modifier: 0.02,
        },
    )).id();

    // Run system once
    app.update();

    let machine = app.world().get::<Machine>(machine_entity).unwrap();
    assert_eq!(machine.efficiency, 1.5);
    assert_eq!(machine.burnout_risk, 0.07);

    // Verify BaseMachineStats was inserted
    let base_stats = app.world().get::<BaseMachineStats>(machine_entity).unwrap();
    assert_eq!(base_stats.base_efficiency, 1.0);
    assert_eq!(base_stats.base_burnout_risk, 0.05);

    // Run system again to ensure non-accumulating
    app.update();

    let machine = app.world().get::<Machine>(machine_entity).unwrap();
    assert_eq!(machine.efficiency, 1.5);
    assert_eq!(machine.burnout_risk, 0.07);

    // Update effect
    app.world_mut().get_mut::<SubconsciousGridEffect>(machine_entity).unwrap().efficiency_multiplier = 0.5;

    app.update();

    let machine = app.world().get::<Machine>(machine_entity).unwrap();
    assert_eq!(machine.efficiency, 0.5);
    assert_eq!(machine.burnout_risk, 0.07);
}

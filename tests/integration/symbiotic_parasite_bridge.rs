use bevy::app::App;
use scale::layer1::agriculture::gastronomy::WorkSpeedBuff;
use scale::layer1::biology::health::Health;
use scale::layer1::biology::symbiotic_parasite::{
    apply_parasite_buffs_system, apply_parasite_health_drain_system, SymbioticParasite,
};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::psychology::needs::Needs;

#[test]
fn test_symbiotic_parasite_bridge() {
    let mut app = App::new();

    app.add_systems(bevy::app::Update, (apply_parasite_buffs_system, apply_parasite_health_drain_system));

    let infected_pop = app
        .world_mut()
        .spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.5,
                ..Default::default()
            },
            WorkSpeedBuff {
                multiplier: 1.0,
                duration: 1,
            },
            SymbioticParasite {
                drain_amount: 5.0,
                drain_radius: 5.0,
            },
            GridPosition { x: 0, y: 0 },
        ))
        .id();

    let healthy_pop = app
        .world_mut()
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            GridPosition { x: 3, y: 0 },
        ))
        .id();

    app.update();

    let needs = app.world().get::<Needs>(infected_pop).unwrap();
    assert_eq!(needs.hunger, 1.0, "Hunger should be frozen at 1.0");
    assert_eq!(needs.rest, 1.0, "Rest should be frozen at 1.0");

    let speed = app.world().get::<WorkSpeedBuff>(infected_pop).unwrap();
    assert_eq!(speed.multiplier, 1.5, "Work speed should be boosted to 1.5");

    let hp = app.world().get::<Health>(healthy_pop).unwrap().current;
    assert_eq!(hp, 95.0, "Healthy pop should be drained by 5.0 HP");
}

use scale::layer1::environment::atmosphere::HeavyIndustry;
use scale::layer1::geology::tectonic::TectonicStress;
use scale::layer1::core::integration::heavy_industry_tectonic_stress_bridge;

#[test]
fn test_heavy_industry_increases_tectonic_stress() {
    let mut app = bevy_app::App::new();
    app.add_plugins(bevy::MinimalPlugins);

    app.init_resource::<TectonicStress>();

    // Spawn an entity with HeavyIndustry
    app.world_mut().spawn(HeavyIndustry { smog_output: 10.0 });

    app.add_systems(bevy_app::Update, heavy_industry_tectonic_stress_bridge);

    app.update(); // Tick 1

    let stress = app.world().resource::<TectonicStress>();
    // Stress should increase by 10.0 * 0.05 = 0.5
    assert_eq!(stress.current, 0.5);
}

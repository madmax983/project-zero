import re

with open("src/layer1/biology/rust_lung.rs", "r") as f:
    content = f.read()

extra_test = """
    #[test]
    fn test_rust_lung_takes_toxic_gas_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Setup pop without Rust-Lung
        let pop_id = app.world_mut().spawn((Pop, Health::default())).id();

        // Apply toxic gas damage
        apply_toxic_gas_damage(&mut app.world_mut(), pop_id, 10.0);

        let current_health = app.world().get::<Health>(pop_id).unwrap();
        assert!((current_health.current - (current_health.max - 10.0)).abs() < f32::EPSILON, "Pop without Rust-Lung should take toxic gas damage");
    }
"""

content = content.replace("    // Accumulation from mining tested in mining.rs since the logic is there.\n}", extra_test + "\n    // Accumulation from mining tested in mining.rs since the logic is there.\n}")

with open("src/layer1/biology/rust_lung.rs", "w") as f:
    f.write(content)

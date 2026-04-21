use crate::layer1::biology::health::Health;
use bevy_ecs::prelude::*;

/// Component indicating the pop is suffering from Rust-Lung.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct RustLung;

/// Applies toxic gas damage to a pop, unless they have immunity from Rust-Lung.
pub fn apply_toxic_gas_damage(world: &mut World, pop_id: Entity, damage: f32) {
    let has_immunity = world.get::<RustLung>(pop_id).is_some();
    if !has_immunity {
        let mut health = world.get_mut::<Health>(pop_id).unwrap();
        health.take_damage(damage);
    }
}

// System is not needed since the logic is integrated into mining.rs
// pub fn mine_low_purity_ore ...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy::prelude::{App, MinimalPlugins};

    #[test]
    fn test_rust_lung_provides_toxic_gas_immunity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Setup pop with Rust-Lung
        let pop_id = app
            .world_mut()
            .spawn((Pop, Health::default(), RustLung))
            .id();

        // Apply toxic gas damage
        apply_toxic_gas_damage(&mut app.world_mut(), pop_id, 10.0);

        let current_health = app.world().get::<Health>(pop_id).unwrap();
        assert!(
            (current_health.current - current_health.max).abs() < f32::EPSILON,
            "Pop with Rust-Lung should not take toxic gas damage"
        );
    }

    #[test]
    fn test_rust_lung_takes_toxic_gas_damage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Setup pop without Rust-Lung
        let pop_id = app.world_mut().spawn((Pop, Health::default())).id();

        // Apply toxic gas damage
        apply_toxic_gas_damage(&mut app.world_mut(), pop_id, 10.0);

        let current_health = app.world().get::<Health>(pop_id).unwrap();
        assert!(
            (current_health.current - (current_health.max - 10.0)).abs() < f32::EPSILON,
            "Pop without Rust-Lung should take toxic gas damage"
        );
    }

    // Accumulation from mining tested in mining.rs since the logic is there.
}

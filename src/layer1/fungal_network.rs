use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct SporeNetwork {
    pub active_taps: u32,
}

#[derive(Component)]
pub struct SporeTap;

#[derive(Component)]
pub struct PopCollectivism {
    pub level: f32,
}

pub fn process_spore_taps_system(
    mut network: ResMut<SporeNetwork>,
    tap_query: Query<&SporeTap>,
    mut pop_query: Query<&mut PopCollectivism>,
) {
    network.active_taps = tap_query.iter().count() as u32;

    if network.active_taps > 0 {
        for mut pop in pop_query.iter_mut() {
            // Apply a very small increase that simulates a slow drift, suitable for Update
            pop.level += 0.0001 * network.active_taps as f32; // Increase collectivism
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_spore_taps_system);
        app.insert_resource(SporeNetwork { active_taps: 0 });
        app
    }

    #[test]
    fn test_spore_tap_increases_network_activity() {
        let mut app = setup_app();

        // Spawn a tap
        app.world_mut().spawn(SporeTap);
        app.update();

        let network = app.world().resource::<SporeNetwork>();
        assert_eq!(
            network.active_taps, 1,
            "The network should count active taps."
        );
    }

    #[test]
    fn test_network_alters_pop_ethics() {
        let mut app = setup_app();

        app.world_mut().spawn(SporeTap);
        let pop = app.world_mut().spawn(PopCollectivism { level: 0.0 }).id();

        app.update(); // Tick 1

        let collectivism = app.world().get::<PopCollectivism>(pop).unwrap();
        assert!(
            collectivism.level > 0.0,
            "Collectivism should increase over time when a tap is active."
        );
    }
}

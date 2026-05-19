use crate::layer1::economy::resources::ColonyResources;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Resource, Default)]
pub struct FeralLogisticsNetwork {
    pub connected_colonies: Vec<Entity>,
}

#[derive(Event)]
pub struct FeralDeliveryTriggerEvent {
    pub target: Entity,
}

pub fn process_feral_logistics(
    mut events: EventReader<FeralDeliveryTriggerEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for _event in events.read() {
        let choice = rng.gen_range(0..=2);
        let amount = rng.gen_range(10..=100) as f32;
        match choice {
            0 => {
                resources.add_food(amount);
            }
            1 => {
                resources.add_metal(amount);
            }
            2 => {
                resources.add_waste(amount);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use bevy_app::App;

    #[test]
    fn test_feral_network_delivers_random_cargo() {
        let mut app = App::new();
        app.add_event::<FeralDeliveryTriggerEvent>();
        app.add_systems(bevy_app::Update, process_feral_logistics);

        app.world_mut().init_resource::<ColonyResources>();

        let target_colony = app.world_mut().spawn_empty().id();
        app.world_mut().insert_resource(FeralLogisticsNetwork {
            connected_colonies: vec![target_colony],
        });

        // Advance time to trigger a delivery
        app.world_mut().send_event(FeralDeliveryTriggerEvent {
            target: target_colony,
        });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        // Since it's randomized, just verify it's no longer entirely empty/default.
        assert!(
            resources.food > 0.0 || resources.metal > 0.0 || resources.waste > 0.0,
            "Feral Logistics should deliver random items to connected colonies"
        );
    }
}

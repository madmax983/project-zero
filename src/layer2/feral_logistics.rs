use bevy::prelude::*;
use rand::Rng;
use crate::layer1::economy::resources::ColonyResources;

#[derive(Resource)]
pub struct FeralLogisticsNetwork {
    pub connected_colonies: Vec<Entity>, // Keeping Entity for design consistency, though ColonyResources is a global resource in Layer 1.
}

#[derive(Event)]
pub struct FeralDeliveryTriggerEvent {
    pub target: Entity,
}

#[allow(clippy::needless_pass_by_value)]
pub fn process_feral_logistics(
    mut events: EventReader<FeralDeliveryTriggerEvent>,
    mut resources: ResMut<ColonyResources>,
) {
    let mut rng = rand::thread_rng();

    for _event in events.read() {
        let choice = rng.gen_range(0..=2);
        let amount = rng.gen_range(10..=100) as f32;
        match choice {
            0 => resources.food += amount,
            1 => resources.stone += amount,
            2 => resources.wood += amount,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feral_network_delivers_random_cargo() {
        let mut app = App::new();
        app.add_event::<FeralDeliveryTriggerEvent>();
        app.add_systems(Update, process_feral_logistics);

        let target_colony = Entity::PLACEHOLDER;
        app.world_mut().insert_resource(ColonyResources::default());
        app.world_mut().insert_resource(FeralLogisticsNetwork {
            connected_colonies: vec![target_colony],
        });

        // Advance time to trigger a delivery
        app.world_mut().send_event(FeralDeliveryTriggerEvent { target: target_colony });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        // Since it's randomized, just verify it's no longer entirely empty/default.
        assert!(
            resources.food > 10.0 || resources.stone > 5.0 || resources.wood > 15.0,
            "Feral Logistics should deliver random items to connected colonies"
        );
    }
}

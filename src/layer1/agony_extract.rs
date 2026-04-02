use crate::layer1::economy::inventory::Inventory;
use crate::layer1::social::morale::Morale;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct HarvestAgonyExtractEvent {
    pub worker: Entity,
}

#[derive(Resource)]
pub struct AgonyExtractConfig {
    pub morale_threshold: f32,
    pub stress_threshold: f32,
}

impl Default for AgonyExtractConfig {
    fn default() -> Self {
        Self {
            morale_threshold: 20.0,
            stress_threshold: 80.0,
        }
    }
}

pub fn process_agony_extract_harvest_system(
    mut events: EventReader<HarvestAgonyExtractEvent>,
    mut workers: Query<(&Morale, &StressTracker, &mut Inventory)>,
    config: Res<AgonyExtractConfig>,
) {
    for event in events.read() {
        if let Ok((morale, stress, mut inventory)) = workers.get_mut(event.worker) {
            if morale.value < config.morale_threshold
                || stress.accumulated_stress > config.stress_threshold
            {
                let _ = inventory.try_add(crate::layer1::economy::inventory::InventoryItem {
                    item_type: crate::layer1::economy::items::ItemType::AgonyExtract,
                    entity: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inventory::Inventory;
    use crate::layer1::economy::items::ItemType;
    use crate::layer1::social::morale::Morale;
    use crate::layer1::stress::StressTracker;
    use bevy::prelude::*;

    #[test]
    fn test_harvest_agony_extract_requires_high_stress() {
        let mut app = App::new();
        app.init_resource::<AgonyExtractConfig>();
        app.add_event::<HarvestAgonyExtractEvent>();
        app.add_systems(Update, process_agony_extract_harvest_system);

        let happy_worker = app
            .world_mut()
            .spawn((
                Morale {
                    value: 80.0,
                    ..default()
                },
                StressTracker {
                    accumulated_stress: 10.0,
                    ..default()
                },
                Inventory::default(),
            ))
            .id();

        let stressed_worker = app
            .world_mut()
            .spawn((
                Morale {
                    value: 10.0,
                    ..default()
                },
                StressTracker {
                    accumulated_stress: 90.0,
                    ..default()
                },
                Inventory::default(),
            ))
            .id();

        // Assume an event triggering a harvest action
        app.world_mut().send_event(HarvestAgonyExtractEvent {
            worker: happy_worker,
        });
        app.world_mut().send_event(HarvestAgonyExtractEvent {
            worker: stressed_worker,
        });

        app.update();

        // Happy worker gets nothing
        let happy_inv = app.world().get::<Inventory>(happy_worker).unwrap();
        let happy_count = happy_inv
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::AgonyExtract)
            .count();
        assert_eq!(happy_count, 0);

        // Stressed worker successfully harvests
        let stressed_inv = app.world().get::<Inventory>(stressed_worker).unwrap();
        let stressed_count = stressed_inv
            .items
            .iter()
            .filter(|i| i.item_type == ItemType::AgonyExtract)
            .count();
        assert_eq!(stressed_count, 1);
    }
}

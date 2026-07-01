use bevy_ecs::prelude::*;
use crate::layer1::economy::inventory::{Inventory, InventoryItem};
use crate::layer1::economy::items::ItemType;
use crate::layer1::nature::radioactive::RadiationSickness;
use rand::Rng;

#[derive(Component)]
pub struct RuinTile {
    pub malady_chance: f32,
}

#[derive(Event)]
pub struct ExcavateEvent {
    pub pop: Entity,
    pub target: Entity,
}

pub fn handle_excavation_system(
    mut events: EventReader<ExcavateEvent>,
    mut pops: Query<&mut Inventory>,
    ruins: Query<&RuinTile>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(ruin) = ruins.get(event.target) {
            if let Ok(mut inventory) = pops.get_mut(event.pop) {
                // Yield Artifact
                inventory.try_add(InventoryItem {
                    item_type: ItemType::Artifact,
                    entity: None,
                });

                // Check for malady
                if rng.gen::<f32>() < ruin.malady_chance {
                    commands.entity(event.pop).insert(RadiationSickness { severity: 50.0 });
                }

                // Destroy the ruin tile
                commands.entity(event.target).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    fn setup_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.add_event::<ExcavateEvent>();
        app.add_systems(bevy_app::Update, handle_excavation_system);
        app
    }

    #[test]
    fn test_excavating_ruin_yields_artifact() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Inventory::default(),
        )).id();

        let ruin_id = app.world_mut().spawn(RuinTile {
            malady_chance: 0.0, // Force no malady
        }).id();

        app.world_mut().send_event(ExcavateEvent {
            pop: pop_id,
            target: ruin_id,
        });

        app.update();

        let pop_inventory = app.world().get::<Inventory>(pop_id).unwrap();
        assert!(pop_inventory.has_item(ItemType::Artifact));
        assert!(app.world().get_entity(ruin_id).is_err()); // Ruin destroyed
    }

    #[test]
    fn test_excavating_ruin_can_trigger_malady() {
        let mut app = setup_app();

        let pop_id = app.world_mut().spawn((
            Pop,
            Inventory::default(),
        )).id();

        let ruin_id = app.world_mut().spawn(RuinTile {
            malady_chance: 1.0, // Force malady
        }).id();

        app.world_mut().send_event(ExcavateEvent {
            pop: pop_id,
            target: ruin_id,
        });

        app.update();

        // Pop should be afflicted with a malady (e.g., RadiationSickness component added)
        assert!(app.world().get::<RadiationSickness>(pop_id).is_some());
    }
}

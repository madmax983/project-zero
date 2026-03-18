use bevy_ecs::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::items::ItemType;
use crate::layer1::inventory::{Inventory, InventoryItem};
use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use rand::Rng;

#[derive(Component, Default)]
pub struct Nanoforge {
    pub active_recipe: Option<ItemType>,
    pub breach_risk: f32, // 0.0 to 1.0
}

#[derive(Component)]
pub struct GreyGoo {
    pub replication_progress: f32, // 0.0 to 1.0, 1.0 triggers spread
}

#[derive(Event)]
pub struct ContainmentBreachEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
}

pub fn nanite_fabrication_system(
    mut query: Query<(Entity, &mut Nanoforge, Option<&mut Inventory>, Option<&GridPosition>)>,
    mut resources: ResMut<ColonyResources>,
    mut breach_events: EventWriter<ContainmentBreachEvent>,
) {
    let mut rng = rand::thread_rng();

    for (entity, forge, mut inventory_opt, pos_opt) in query.iter_mut() {
        // Roll for breach
        if rng.gen::<f32>() < forge.breach_risk {
            let pos = pos_opt.copied().unwrap_or(GridPosition { x: 0, y: 0 }); // Fallback for tests
            breach_events.send(ContainmentBreachEvent {
                source_entity: entity,
                position: pos,
            });

            // Optionally, transform forge itself into Grey Goo here or let another system handle it
            continue;
        }

        // Instant Production Logic
        if let Some(recipe) = forge.active_recipe {
            // Simplified recipe check: 10 Energy + 5 RawMass = 1 item
            // Using ColonyResources since we don't have Energy/RawMass in ItemType yet
            // (Note: there is no "energy" in ColonyResources, so we'll just consume metal)
            let mass_needed = 5.0; // Treating metal as mass

            if resources.metal >= mass_needed {
                resources.metal -= mass_needed;

                if let Some(inv) = inventory_opt.as_mut() {
                    inv.add(InventoryItem {
                        item_type: recipe,
                        entity: None,
                    });
                }
            }
        }
    }
}

pub fn grey_goo_replication_system(
    mut commands: Commands,
    mut goo_query: Query<(&mut GreyGoo, &GridPosition)>,
    target_query: Query<(Entity, &GridPosition, Option<&Building>)>, // Simplified target finding
) {
    for (mut goo, goo_pos) in goo_query.iter_mut() {
        if goo.replication_progress >= 1.0 {
            goo.replication_progress = 0.0; // Reset progress after replication

            // Find an adjacent target to consume
            for (target_entity, target_pos, building_opt) in target_query.iter() {
                // Check adjacency (simplistic orthogonal check)
                if (goo_pos.x - target_pos.x).abs() + (goo_pos.y - target_pos.y).abs() == 1 && building_opt.is_some() {
                    // Consume!
                    commands.entity(target_entity).despawn();
                    commands.spawn((
                        GreyGoo { replication_progress: 0.0 },
                        *target_pos,
                    ));
                    break; // Only consume one per tick per goo tile
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use bevy_ecs::event::Events;
    use crate::layer1::building::BuildingType;

    #[test]
    fn test_nanoforge_instant_production() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        app.insert_resource(ColonyResources {
            metal: 50.0,
            ..Default::default()
        });

        let inventory = Inventory::default();

        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            Nanoforge {
                active_recipe: Some(ItemType::Tool),
                breach_risk: 0.0,
            },
            inventory,
        )).id();

        app.update();

        let res = app.world().get_resource::<ColonyResources>().unwrap();
        assert!(res.metal < 50.0);

        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        assert!(inventory_after.items.iter().any(|i| i.item_type == ItemType::Tool));
    }

    #[test]
    fn test_nanoforge_containment_breach_event() {
        let mut app = App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_systems(Update, nanite_fabrication_system);

        app.insert_resource(ColonyResources::default());

        let forge_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            Nanoforge {
                active_recipe: Some(ItemType::Tool),
                breach_risk: 1.0, // Guaranteed breach
            },
        )).id();

        app.update();

        let events = app.world().resource::<Events<ContainmentBreachEvent>>();
        let mut cursor = events.get_cursor();
        let breach_events: Vec<_> = cursor.read(events).collect();

        assert_eq!(breach_events.len(), 1);
        assert_eq!(breach_events[0].source_entity, forge_entity);
    }

    #[test]
    fn test_grey_goo_replication() {
        let mut app = App::new();
        app.add_systems(Update, grey_goo_replication_system);

        let pos_goo = GridPosition { x: 10, y: 10 };
        let pos_target = GridPosition { x: 10, y: 11 };

        let target_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            pos_target,
        )).id();

        app.world_mut().spawn((
            GreyGoo { replication_progress: 1.0 },
            pos_goo,
        ));

        app.update();

        assert!(app.world().get_entity(target_entity).is_err() || app.world().get::<Building>(target_entity).is_none());

        let mut new_goo_found = false;
        let mut query = app.world_mut().query::<(&GreyGoo, &GridPosition)>();
        for (_, pos) in query.iter(app.world()) {
            if pos.x == 10 && pos.y == 11 {
                new_goo_found = true;
                break;
            }
        }
        assert!(new_goo_found, "Grey Goo failed to replicate to adjacent tile.");
    }
}

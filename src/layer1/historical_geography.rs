use bevy_ecs::prelude::*;
use bevy::utils::HashMap;
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct HistoricalName {
    pub name: String,
}

#[derive(Event)]
pub struct HistoricalEvent {
    pub location: GridPosition,
    pub event_type: EventType,
}

#[derive(Clone, Copy)]
pub enum EventType {
    Famine,
    Tragedy,
    Discovery,
}

pub fn process_historical_events(
    mut events: EventReader<HistoricalEvent>,
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, Option<&HistoricalName>)>,
) {
    if events.is_empty() {
        return;
    }

    // Build a spatial map for fast lookups
    let mut grid_map: HashMap<GridPosition, (Entity, Option<String>)> = HashMap::new();
    for (entity, pos, name) in query.iter() {
        grid_map.insert(*pos, (entity, name.map(|n| n.name.clone())));
    }

    for event in events.read() {
        if let Some((entity, current_name)) = grid_map.get(&event.location) {
            let generated_name = match event.event_type {
                EventType::Famine => "Famine Field".to_string(),
                EventType::Tragedy => "Site of Tragedy".to_string(),
                EventType::Discovery => "Discovery Point".to_string(),
            };

            if let Some(existing) = current_name {
                let new_name = format!("{} ({})", existing, generated_name);
                commands.entity(*entity).insert(HistoricalName { name: new_name.clone() });

                // Update our cache in case of multiple events on the same tile in one tick
                grid_map.insert(event.location, (*entity, Some(new_name)));
            } else {
                commands.entity(*entity).insert(HistoricalName { name: generated_name.clone() });

                // Update our cache
                grid_map.insert(event.location, (*entity, Some(generated_name)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_tile_earns_name_from_event() {
        let mut world = World::new();
        world.init_resource::<Events<HistoricalEvent>>();

        let tile_entity = world.spawn((
            GridPosition { x: 5, y: 5 },
        )).id();

        // Emit an event that happened at this location
        world.send_event(HistoricalEvent {
            location: GridPosition { x: 5, y: 5 },
            event_type: EventType::Famine,
        });

        world.run_system_once(process_historical_events).unwrap();

        // The tile should now have a HistoricalName component
        let name_comp = world.get::<HistoricalName>(tile_entity);
        assert!(name_comp.is_some());
        assert_eq!(name_comp.unwrap().name, "Famine Field"); // Or some generated equivalent
    }

    #[test]
    fn test_name_persists_and_updates_gracefully() {
        let mut world = World::new();
        world.init_resource::<Events<HistoricalEvent>>();

        let tile_entity = world.spawn((
            GridPosition { x: 0, y: 0 },
            HistoricalName { name: "Founders' Peak".to_string() }
        )).id();

        // New event happens
        world.send_event(HistoricalEvent {
            location: GridPosition { x: 0, y: 0 },
            event_type: EventType::Tragedy,
        });

        world.run_system_once(process_historical_events).unwrap();

        // Name might append, or keep the old one depending on rules
        let name = &world.get::<HistoricalName>(tile_entity).unwrap().name;
        // Verify expected behavior (e.g., "Founders' Peak (Site of Tragedy)")
        assert!(name.contains("Founders' Peak"));
        assert!(name.contains("Tragedy"));
    }
}

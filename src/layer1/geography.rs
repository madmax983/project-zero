use crate::layer1::map::GridPosition;
use bevy::prelude::*;
use bevy::utils::HashMap;

/// Name given to a tile due to a significant historical event.
#[derive(Component, Debug, Clone)]
pub struct HistoricalName {
    pub name: String,
}

/// Event representing a significant occurrence at a specific location.
#[derive(Event, Debug, Clone)]
pub struct HistoricalEvent {
    pub location: GridPosition,
    pub event_type: EventType,
}

/// Types of historical events that can name a geography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Famine,
    Tragedy,
    Discovery,
}

/// Processes historical events and assigns names to tiles.
pub fn process_historical_events(
    mut events: EventReader<HistoricalEvent>,
    mut commands: Commands,
    query: Query<(Entity, &GridPosition, Option<&HistoricalName>)>,
) {
    if events.is_empty() {
        return;
    }

    // Build a spatial map for fast lookup
    let mut tile_map: HashMap<GridPosition, (Entity, Option<String>)> = HashMap::new();
    for (entity, pos, current_name) in query.iter() {
        tile_map.insert(*pos, (entity, current_name.map(|n| n.name.clone())));
    }

    for event in events.read() {
        if let Some((entity, current_name)) = tile_map.get_mut(&event.location) {
            let generated_name = match event.event_type {
                EventType::Famine => "Famine Field".to_string(),
                EventType::Tragedy => "Site of Tragedy".to_string(),
                EventType::Discovery => "Discovery Point".to_string(),
            };

            let new_name = if let Some(existing) = current_name {
                format!("{} ({})", existing, generated_name)
            } else {
                generated_name
            };

            commands.entity(*entity).insert(HistoricalName {
                name: new_name.clone(),
            });

            // Update the map to reflect the new name in case multiple events hit the same tile
            *current_name = Some(new_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::nature::terrain::TerrainType;

    #[test]
    fn test_tile_earns_name_from_event() {
        let mut app = App::new();
        app.add_event::<HistoricalEvent>();
        app.add_systems(Update, process_historical_events);

        let tile_entity = app
            .world_mut()
            .spawn((GridPosition { x: 5, y: 5 }, TerrainType::Grass))
            .id();

        // Emit an event that happened at this location
        app.world_mut().send_event(HistoricalEvent {
            location: GridPosition { x: 5, y: 5 },
            event_type: EventType::Famine,
        });
        app.update();

        // The tile should now have a HistoricalName component
        let name_comp = app.world().get::<HistoricalName>(tile_entity);
        assert!(name_comp.is_some());
        assert_eq!(name_comp.unwrap().name, "Famine Field");
    }

    #[test]
    fn test_name_persists_and_updates_gracefully() {
        let mut app = App::new();
        app.add_event::<HistoricalEvent>();
        app.add_systems(Update, process_historical_events);

        let tile_entity = app
            .world_mut()
            .spawn((
                GridPosition { x: 0, y: 0 },
                TerrainType::Rock,
                HistoricalName {
                    name: "Founders' Peak".to_string(),
                },
            ))
            .id();

        // New event happens
        app.world_mut().send_event(HistoricalEvent {
            location: GridPosition { x: 0, y: 0 },
            event_type: EventType::Tragedy,
        });
        app.update();

        // Name might append, or keep the old one depending on rules
        let name = &app.world().get::<HistoricalName>(tile_entity).unwrap().name;
        // Verify expected behavior (e.g., "Founders' Peak (Site of Tragedy)")
        assert!(name.contains("Founders' Peak"));
        assert!(name.contains("Site of Tragedy"));
        assert_eq!(name, "Founders' Peak (Site of Tragedy)");
    }
}

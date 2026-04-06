use bevy::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structure::Structure;

#[derive(Component)]
pub struct OrbitalTetherAnchor {
    pub orientation: Vec2, // Direction the cable falls if severed
}

#[derive(Event)]
pub struct TetherSnapEvent {
    pub anchor_entity: Entity,
    pub origin: GridPosition,
    pub fall_direction: Vec2,
}

pub fn detect_tether_destruction_system(
    query: Query<(Entity, &OrbitalTetherAnchor, &GridPosition, &Structure), Changed<Structure>>,
    mut snap_events: EventWriter<TetherSnapEvent>,
) {
    for (entity, anchor, pos, health) in query.iter() {
        if health.current_hp <= 0.0 {
            snap_events.send(TetherSnapEvent {
                anchor_entity: entity,
                origin: *pos,
                fall_direction: anchor.orientation,
            });
        }
    }
}

pub fn process_tether_whip_system(
    mut snap_events: EventReader<TetherSnapEvent>,
    mut struct_query: Query<(&GridPosition, &mut Structure)>,
) {
    for event in snap_events.read() {
        // Very basic line check (horizontal or vertical based on orientation vector)
        for (pos, mut health) in struct_query.iter_mut() {
            let is_in_path = if event.fall_direction.x > 0.0 {
                pos.y == event.origin.y && pos.x > event.origin.x
            } else if event.fall_direction.y > 0.0 {
                pos.x == event.origin.x && pos.y > event.origin.y
            } else {
                false
            };

            if is_in_path {
                health.current_hp = 0.0; // Instant obliteration
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use crate::layer1::chronicle::AddChronicleEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<TetherSnapEvent>();
        app.add_event::<AddChronicleEvent>();
        // TerrainGrid is not used by tests
        app.add_systems(Update, (
            detect_tether_destruction_system,
            process_tether_whip_system,
        ));
        app
    }

    #[test]
    fn test_tether_destruction_triggers_snap_event() {
        let mut app = setup_app();

        let tether_entity = app.world_mut().spawn((
            OrbitalTetherAnchor { orientation: Vec2::new(1.0, 0.0) },
            GridPosition { x: 10, y: 10 },
            Structure { current_hp: 0.0, max_hp: 1000.0 }, // Destroyed
        )).id();

        app.update();

        let snap_events = app.world().resource::<Events<TetherSnapEvent>>();
        let mut reader = snap_events.get_cursor();
        let events: Vec<_> = reader.read(&snap_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].anchor_entity, tether_entity);
        assert_eq!(events[0].origin, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_tether_whip_destroys_structures_in_line() {
        let mut app = setup_app();

        // Spawn structures in the path of the falling cable (horizontal fall)
        let struct_1 = app.world_mut().spawn((
            GridPosition { x: 11, y: 10 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();
        let struct_2 = app.world_mut().spawn((
            GridPosition { x: 20, y: 10 },
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();
        let struct_safe = app.world_mut().spawn((
            GridPosition { x: 10, y: 11 }, // Not in line
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Trigger a snap event
        app.world_mut().send_event(TetherSnapEvent {
            anchor_entity: Entity::from_raw(0), // Dummy
            origin: GridPosition { x: 10, y: 10 },
            fall_direction: Vec2::new(1.0, 0.0),
        });

        app.update();

        // Structures in path should be destroyed (health = 0)
        assert_eq!(app.world().get::<Structure>(struct_1).unwrap().current_hp, 0.0);
        assert_eq!(app.world().get::<Structure>(struct_2).unwrap().current_hp, 0.0);
        // Safe structure remains intact
        assert_eq!(app.world().get::<Structure>(struct_safe).unwrap().current_hp, 100.0);
    }
}

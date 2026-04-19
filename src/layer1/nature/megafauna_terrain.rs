use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::TerrainType;
use crate::layer1::shipbreaking::MineEvent;
use crate::layer1::structure::Structure;

#[derive(Component)]
pub struct MegafaunaTerrain {
    pub is_dormant: bool,
    pub base_health: f32,
}

#[derive(Event)]
pub struct AwakenTitanEvent {
    pub entity: Entity,
}

pub fn process_mining_titan_system(
    mut mine_events: EventReader<MineEvent>,
    mut titans: Query<&mut MegafaunaTerrain>,
    mut awaken_events: EventWriter<AwakenTitanEvent>,
) {
    for ev in mine_events.read() {
        if let Ok(mut titan) = titans.get_mut(ev.target) {
            if titan.is_dormant {
                titan.is_dormant = false;
                awaken_events.send(AwakenTitanEvent { entity: ev.target });
            }
        }
    }
}

pub fn awaken_titan_system(
    mut awaken_events: EventReader<AwakenTitanEvent>,
    mut commands: Commands,
    titans: Query<&GridPosition, With<MegafaunaTerrain>>,
    structures: Query<(Entity, &GridPosition), With<Structure>>,
) {
    for ev in awaken_events.read() {
        if let Ok(titan_pos) = titans.get(ev.entity) {
            // Destroy any structures built on the titan
            for (struct_entity, struct_pos) in structures.iter() {
                if struct_pos == titan_pos {
                    commands.entity(struct_entity).despawn();
                }
            }

            // Alter the terrain type or convert to fauna
            commands.entity(ev.entity).insert(TerrainType::DeepRock);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, Update};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::TerrainType;
    use crate::layer1::structure::Structure;
    use crate::layer1::shipbreaking::MineEvent;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_event::<AwakenTitanEvent>();
        app.add_systems(Update, (process_mining_titan_system, awaken_titan_system));
        app
    }

    #[test]
    fn test_mining_dormant_titan_triggers_awakening() {
        let mut app = setup_app();

        let titan_entity = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        let miner = app.world_mut().spawn_empty().id();

        // Simulate a pop mining the titan terrain
        app.world_mut().send_event(MineEvent {
            target: titan_entity,
            miner,
        });

        app.update();

        // Assert an awaken event was fired or the titan state changed
        let titan = app.world().get::<MegafaunaTerrain>(titan_entity).unwrap();
        assert!(!titan.is_dormant, "Mining a dormant titan should wake it up");
    }

    #[test]
    fn test_titan_awakening_destroys_structures_on_top() {
        let mut app = setup_app();

        let pos = GridPosition { x: 15, y: 15 };

        let titan_entity = app.world_mut().spawn((
            pos,
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        let building_entity = app.world_mut().spawn((
            pos,
            Structure { current_hp: 100.0, max_hp: 100.0 },
        )).id();

        // Wake up the titan
        app.world_mut().send_event(AwakenTitanEvent {
            entity: titan_entity,
        });

        app.update();

        // The structure should be destroyed (despawned or integrity 0)
        assert!(app.world().get_entity(building_entity).is_err(), "Buildings on top of an awakened titan must be destroyed");
    }

    #[test]
    fn test_awakened_titan_changes_terrain_type() {
        let mut app = setup_app();

        let titan_entity = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            TerrainType::Rock,
            MegafaunaTerrain {
                is_dormant: true,
                base_health: 1000.0,
            },
        )).id();

        app.world_mut().send_event(AwakenTitanEvent {
            entity: titan_entity,
        });

        app.update();

        let terrain = app.world().get::<TerrainType>(titan_entity).unwrap();
        assert_eq!(*terrain, TerrainType::DeepRock, "Awakened titan changes its physical terrain representation or becomes a distinct entity");
        // Note: Implementation may choose to convert it to a Fauna entity instead.
    }
}

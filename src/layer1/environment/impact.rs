use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::structure::Structure;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct IncomingImpact {
    pub target_pos: GridPosition,
    pub radius: i32,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactWarningEvent {
    pub target_pos: GridPosition,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct ImpactStrikeEvent {
    pub center: GridPosition,
    pub radius: i32,
}

pub fn process_impact_countdown_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut impacts: Query<(Entity, &mut IncomingImpact)>,
    mut strike_events: EventWriter<ImpactStrikeEvent>,
) {
    if !time.is_changed() {
        return;
    }

    for (entity, mut impact) in impacts.iter_mut() {
        if impact.ticks_remaining > 0 {
            impact.ticks_remaining -= 1;
        }

        if impact.ticks_remaining == 0 {
            strike_events.send(ImpactStrikeEvent {
                center: impact.target_pos,
                radius: impact.radius,
            });
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_impact_strike_system(
    mut strike_events: EventReader<ImpactStrikeEvent>,
    mut commands: Commands,
    structures: Query<(Entity, &GridPosition), With<Structure>>,
    mut terrain_grid_res: Option<ResMut<TerrainGrid>>,
    mut chronicle_events: EventWriter<crate::layer1::chronicle::AddChronicleEvent>,
) {
    for ev in strike_events.read() {
        chronicle_events.send(crate::layer1::chronicle::AddChronicleEvent {
            text: format!(
                "A devastating impact struck the colony at coordinates ({}, {}).",
                ev.center.x, ev.center.y
            ),
            importance: crate::layer1::chronicle::EventImportance::Major,
        });
        // Obliterate structures
        for (entity, pos) in structures.iter() {
            if pos.distance_chebyshev(ev.center) <= ev.radius as u32 {
                commands.entity(entity).despawn();
            }
        }

        // Crater terrain
        if let Some(terrain_grid) = terrain_grid_res.as_deref_mut() {
            let min_x = (ev.center.x - ev.radius).max(0);
            let max_x = (ev.center.x + ev.radius).min(terrain_grid.width as i32 - 1);
            let min_y = (ev.center.y - ev.radius).max(0);
            let max_y = (ev.center.y + ev.radius).min(terrain_grid.height as i32 - 1);
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let p = GridPosition { x, y };
                    if p.distance_chebyshev(ev.center) <= ev.radius as u32 {
                        let idx = (y * terrain_grid.width as i32 + x) as usize;
                        if idx < terrain_grid.tiles.len() {
                            terrain_grid.tiles[idx] = TerrainType::DeepRock;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::structure::Structure;
    use crate::shared::time::SimulationTime;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_event::<ImpactWarningEvent>();
        app.add_event::<ImpactStrikeEvent>();
        app.add_event::<crate::layer1::chronicle::AddChronicleEvent>();
        app.add_systems(
            Update,
            (
                process_impact_countdown_system,
                process_impact_strike_system,
            ),
        );
        app
    }

    #[test]
    fn test_impact_countdown_triggers_strike() {
        let mut app = setup_app();

        let impact = app
            .world_mut()
            .spawn(IncomingImpact {
                target_pos: GridPosition { x: 20, y: 20 },
                radius: 5,
                ticks_remaining: 10,
            })
            .id();

        // Simulate 9 ticks passing
        for _ in 0..9 {
            let mut time = app.world_mut().resource_mut::<SimulationTime>();
            time.tick += 1;
            app.update();
        }

        assert!(app.world().get::<IncomingImpact>(impact).is_some());

        // 10th tick
        {
            let mut time = app.world_mut().resource_mut::<SimulationTime>();
            time.tick += 1;
        }
        app.update();

        assert!(app.world().get::<IncomingImpact>(impact).is_none());

        let strike_events = app
            .world()
            .get_resource::<Events<ImpactStrikeEvent>>()
            .unwrap();
        let mut reader = strike_events.get_cursor();
        let event = reader.read(strike_events).next().unwrap();
        assert_eq!(event.center, GridPosition { x: 20, y: 20 });
        assert_eq!(event.radius, 5);
    }

    #[test]
    fn test_impact_strike_destroys_structures_and_alters_terrain() {
        let mut app = setup_app();

        app.world_mut().insert_resource(TerrainGrid {
            width: 20,
            height: 20,
            tiles: vec![TerrainType::Grass; 400],
        });

        let center = GridPosition { x: 10, y: 10 };

        let structure = app
            .world_mut()
            .spawn((
                center,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .send_event(ImpactStrikeEvent { center, radius: 2 });

        app.update();

        assert!(app.world().get_entity(structure).is_err());

        let terrain_grid = app.world().get_resource::<TerrainGrid>().unwrap();
        // 10 + 10 * 20 = 210
        assert_eq!(terrain_grid.tiles[210], TerrainType::DeepRock);
        // 11 + 10 * 20 = 211
        assert_eq!(terrain_grid.tiles[211], TerrainType::DeepRock);
    }
}

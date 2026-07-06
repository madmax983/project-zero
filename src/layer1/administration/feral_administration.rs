use crate::layer1::admin::Office;
use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AdministrativeBuilding;

#[derive(Component)]
pub struct PowerStatus {
    pub is_powered: bool,
}

#[derive(Component)]
pub struct StaffedStatus {
    pub is_staffed: bool,
}

#[derive(Component)]
pub struct UnprocessedForms {
    pub stack_size: i32,
}

#[derive(Component)]
pub struct FeralColony;

pub fn spawn_unprocessed_forms_system(
    mut commands: Commands,
    query: Query<(&GridPosition, &PowerConsumer, &Office), With<AdministrativeBuilding>>,
    mut forms_query: Query<(&GridPosition, &mut UnprocessedForms)>,
    time: Option<Res<crate::shared::time::SimulationTime>>,
) {
    if let Some(t) = time {
        if t.tick % 10 != 0 {
            return;
        }
    }

    let mut rng = rand::thread_rng();
    use rand::Rng;

    for (pos, power, office) in query.iter() {
        if !power.active || office.workers.is_empty() {
            let offset_x = 1;
            let offset_y = 0;
            // consume unused rng
            let _ = rng.gen_range(-1..=1);

            let spawn_pos = GridPosition {
                x: pos.x + offset_x,
                y: pos.y + offset_y,
            };

            let mut found = false;
            for (form_pos, mut form) in forms_query.iter_mut() {
                if form_pos.x == spawn_pos.x && form_pos.y == spawn_pos.y {
                    form.stack_size += 1;
                    found = true;
                    break;
                }
            }

            if !found {
                commands.spawn((UnprocessedForms { stack_size: 1 }, spawn_pos));
            }
        }
    }
}

pub fn feral_admin_chronicle_bridge(
    query: Query<(Entity, &UnprocessedForms), Changed<UnprocessedForms>>,
    mut recorded: Local<bevy_utils::HashSet<Entity>>,
    mut chronicle_events: EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for (entity, forms) in query.iter() {
        if forms.stack_size >= 10 && !recorded.contains(&entity) {
            recorded.insert(entity);
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                text: "A mountain of Unprocessed Forms has collapsed, rendering a section of the colony impassable!".to_string(),
                importance: crate::layer1::core::chronicle::EventImportance::Minor,
            });
        }
    }
}

pub fn process_impassable_terrain_system(
    query: Query<(&GridPosition, &UnprocessedForms), Changed<UnprocessedForms>>,
    mut occupied: ResMut<crate::layer1::core::spatial::OccupiedTiles>,
) {
    for (pos, forms) in query.iter() {
        if forms.stack_size >= 10 {
            occupied.0.insert((pos.x, pos.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_unprocessed_forms_spawn_when_unstaffed_or_unpowered() {
        let mut app = App::new();
        app.add_systems(Update, spawn_unprocessed_forms_system);

        let admin_pos = GridPosition { x: 5, y: 5 };
        app.insert_resource(crate::shared::time::SimulationTime {
            tick: 10,
            ..Default::default()
        });

        app.world_mut().spawn((
            FeralColony,
            AdministrativeBuilding,
            PowerConsumer {
                demand: 10.0,
                active: false,
            },
            Office::default(),
            admin_pos,
        ));

        app.update();

        let mut found_forms = false;
        for (_, pos) in app
            .world_mut()
            .query::<(&UnprocessedForms, &GridPosition)>()
            .iter(app.world())
        {
            let dist = (pos.x - admin_pos.x).abs() + (pos.y - admin_pos.y).abs();
            if dist == 1 {
                found_forms = true;
                break;
            }
        }
        assert!(found_forms);
    }

    #[test]
    fn test_unprocessed_forms_block_pathfinding() {
        let mut app = App::new();
        app.add_systems(Update, process_impassable_terrain_system);
        app.insert_resource(crate::layer1::core::spatial::OccupiedTiles::default());

        app.world_mut().spawn((
            UnprocessedForms { stack_size: 10 },
            GridPosition { x: 10, y: 10 },
        ));

        app.update();

        let grid = app
            .world()
            .resource::<crate::layer1::core::spatial::OccupiedTiles>();
        assert!(grid.0.contains(&(10, 10)));
    }
}

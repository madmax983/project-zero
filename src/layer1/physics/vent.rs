use bevy_ecs::prelude::*;
use bevy::prelude::{App, Plugin, Update};
use bevy::math::UVec2;

#[derive(Component)]
pub struct Position(pub UVec2);

#[derive(Component)]
pub struct VentConnection {
    pub pos_a: UVec2,
    pub pos_b: UVec2,
    pub grated: bool,
}

#[derive(Component)]
pub struct SmallEntity;

#[derive(Component)]
pub struct PathNavigator;

#[derive(Event)]
pub struct PathRequest {
    pub entity: Entity,
    pub start: UVec2,
    pub end: UVec2,
}

#[derive(Component)]
pub struct PathResult {
    pub success: bool,
}

pub struct VentilationPlugin;

impl Plugin for VentilationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PathRequest>()
           .add_systems(Update, handle_vent_pathfinding);
    }
}

pub fn handle_vent_pathfinding(
    mut events: EventReader<PathRequest>,
    vents: Query<&VentConnection>,
    small_entities: Query<&SmallEntity>,
    mut commands: Commands,
) {
    use bevy::utils::HashMap;
    let mut vent_map: HashMap<(UVec2, UVec2), bool> = HashMap::default();

    for vent in vents.iter() {
        vent_map.insert((vent.pos_a, vent.pos_b), vent.grated);
        vent_map.insert((vent.pos_b, vent.pos_a), vent.grated);
    }

    for event in events.read() {
        let is_small = small_entities.get(event.entity).is_ok();

        let mut success = false;
        if is_small {
            if let Some(&grated) = vent_map.get(&(event.start, event.end)) {
                success = !grated;
            }
        }

        commands.entity(event.entity).insert(PathResult { success });
    }
}

pub fn calculate_vent_airflow(vent: &VentConnection) -> f32 {
    if vent.grated {
        0.5 // 50% airflow
    } else {
        1.0 // 100% airflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_entities_can_traverse_vents() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        // Grid positions
        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        // Vent connecting A and B
        app.world_mut().spawn(VentConnection { pos_a, pos_b, grated: false });

        // Small entity
        let rat = app.world_mut().spawn((SmallEntity, PathNavigator, Position(pos_a))).id();

        // Large entity
        let human = app.world_mut().spawn((PathNavigator, Position(pos_a))).id();

        // System should allow rat to path through, but not human
        app.world_mut().send_event(PathRequest { entity: rat, start: pos_a, end: pos_b });
        app.world_mut().send_event(PathRequest { entity: human, start: pos_a, end: pos_b });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(rat_path.success, "Small entities should path through open vents");

        let human_path = app.world().get::<PathResult>(human).unwrap();
        assert!(!human_path.success, "Large entities should not path through vents");
    }

    #[test]
    fn test_grates_block_movement_but_allow_some_air() {
        let mut app = App::new();
        app.add_plugins(VentilationPlugin);

        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(0, 1);

        let vent = app.world_mut().spawn(VentConnection { pos_a, pos_b, grated: true }).id();

        let rat = app.world_mut().spawn((SmallEntity, PathNavigator, Position(pos_a))).id();

        app.world_mut().send_event(PathRequest { entity: rat, start: pos_a, end: pos_b });
        app.update();

        let rat_path = app.world().get::<PathResult>(rat).unwrap();
        assert!(!rat_path.success, "Small entities should not path through grated vents");

        // Verify airflow is still present but reduced
        let airflow_amount = calculate_vent_airflow(app.world().get::<VentConnection>(vent).unwrap());
        assert!(airflow_amount > 0.0 && airflow_amount < 1.0, "Grated vent should have reduced airflow");
    }
}

use crate::layer1::map::GridPosition;
use crate::layer1::map::ZLevel;
use crate::layer1::shipbreaking::MineEvent;
use bevy::prelude::*;

#[derive(Component)]
pub struct HazardFlora {
    pub lethality: f32,
    pub spread_rate: f32,
}

const DEEP_CRUST_THRESHOLD: i32 = -30;

pub fn deep_crust_breach_system(
    mut commands: Commands,
    mut mine_events: EventReader<MineEvent>,
    query: Query<(&GridPosition, Option<&ZLevel>)>,
) {
    for event in mine_events.read() {
        if let Ok((pos, z_level)) = query.get(event.target) {
            let z = z_level.map(|z| z.0).unwrap_or(0);
            if z <= DEEP_CRUST_THRESHOLD {
                commands.spawn((
                    *pos,
                    HazardFlora {
                        lethality: 10.0,
                        spread_rate: 2.0,
                    },
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_mining_spawns_hostile_biosphere() {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_systems(Update, deep_crust_breach_system);

        let target_entity = app
            .world_mut()
            .spawn((GridPosition { x: 10, y: 10 }, ZLevel(-50)))
            .id();

        app.world_mut()
            .resource_mut::<Events<MineEvent>>()
            .send(MineEvent {
                target: target_entity,
                miner: Entity::from_raw(1),
            });

        app.update();

        let mut hazard_query = app.world_mut().query::<(&GridPosition, &HazardFlora)>();
        let mut found_hazard = false;
        for (pos, _hazard) in hazard_query.iter(app.world()) {
            if pos.x == 10 && pos.y == 10 {
                found_hazard = true;
                break;
            }
        }

        assert!(
            found_hazard,
            "Mining at depth should spawn hostile flora/fauna."
        );
    }

    #[test]
    fn test_shallow_mining_does_not_spawn_hostile_biosphere() {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_systems(Update, deep_crust_breach_system);

        let target_entity = app
            .world_mut()
            .spawn((GridPosition { x: 10, y: 10 }, ZLevel(-2)))
            .id();

        app.world_mut()
            .resource_mut::<Events<MineEvent>>()
            .send(MineEvent {
                target: target_entity,
                miner: Entity::from_raw(1),
            });

        app.update();

        let mut hazard_query = app.world_mut().query::<&HazardFlora>();
        assert_eq!(
            hazard_query.iter(app.world()).count(),
            0,
            "Shallow mining should not trigger deep crust breaches."
        );
    }
}

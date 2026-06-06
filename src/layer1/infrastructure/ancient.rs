use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::shipbreaking::MineEvent;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConduitResource {
    Power,
}

#[derive(Component)]
pub struct BuriedConduit {
    pub resource_type: ConduitResource,
    pub yield_amount: f32,
}

#[derive(Component)]
pub struct ExposedConduit {
    pub resource_type: ConduitResource,
    pub yield_amount: f32,
}

#[derive(Component)]
pub struct ConduitRisk {
    pub surge_chance: f32, // 0.0 to 1.0
}

#[derive(Event)]
pub struct ConduitSurgeEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
    pub magnitude: f32,
}

pub fn process_mine_conduit_system(
    mut commands: Commands,
    mut mine_events: EventReader<MineEvent>,
    conduits: Query<(Entity, &BuriedConduit)>,
) {
    for ev in mine_events.read() {
        if let Ok((entity, buried)) = conduits.get(ev.target) {
            // Expose the conduit
            commands
                .entity(entity)
                .remove::<BuriedConduit>()
                .insert(ExposedConduit {
                    resource_type: buried.resource_type,
                    yield_amount: buried.yield_amount,
                })
                .insert(ConduitRisk { surge_chance: 0.01 }); // Base 1% risk per tick
        }
    }
}

pub fn tap_conduit_power_system(
    mut commands: Commands,
    conduits: Query<(Entity, &ExposedConduit), Without<PowerSource>>,
) {
    for (entity, conduit) in conduits.iter() {
        if conduit.resource_type == ConduitResource::Power {
            commands.entity(entity).insert(PowerSource {
                output: conduit.yield_amount,
                active: true,
            });
        }
    }
}

pub fn process_conduit_surge_system(
    mut commands: Commands,
    mut surge_events: EventWriter<ConduitSurgeEvent>,
    conduits: Query<(Entity, &GridPosition, &ConduitRisk, &ExposedConduit)>,
) {
    let mut rng = rand::thread_rng();
    for (entity, pos, risk, conduit) in conduits.iter() {
        if rng.gen::<f32>() < risk.surge_chance {
            surge_events.send(ConduitSurgeEvent {
                source_entity: entity,
                position: *pos,
                magnitude: conduit.yield_amount * 2.0, // Surge is a massive spike
            });

            // Optionally, the conduit breaks after a surge
            commands.entity(entity).remove::<ExposedConduit>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::map::GridPosition;
    use crate::layer1::shipbreaking::MineEvent;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<MineEvent>();
        app.add_event::<ConduitSurgeEvent>();
        app.add_systems(
            Update,
            (
                process_mine_conduit_system,
                tap_conduit_power_system,
                process_conduit_surge_system,
            ),
        );
        app
    }

    #[test]
    fn test_mining_reveals_ancient_conduit() {
        let mut app = setup_app();
        let miner = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                BuriedConduit {
                    resource_type: ConduitResource::Power,
                    yield_amount: 100.0,
                },
            ))
            .id();

        // Mine the tile containing the buried conduit
        app.world_mut().send_event(MineEvent {
            miner,
            target: entity,
        });
        app.update();

        // Assert the conduit is now exposed
        assert!(app.world().get::<ExposedConduit>(entity).is_some());
    }

    #[test]
    fn test_exposed_conduit_provides_free_power_but_risks_surge() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                ExposedConduit {
                    resource_type: ConduitResource::Power,
                    yield_amount: 100.0,
                },
                ConduitRisk { surge_chance: 1.0 }, // 100% chance to surge for testing
            ))
            .id();

        app.update();

        // The conduit should have provided power
        let power_source = app.world().get::<PowerSource>(entity).unwrap();
        assert_eq!(power_source.output, 100.0);

        // And it should have caused a surge event due to 100% risk
        let surge_events = app
            .world()
            .get_resource::<Events<ConduitSurgeEvent>>()
            .unwrap();
        let mut reader = surge_events.get_cursor();
        let ev = reader.read(surge_events).next().unwrap();
        assert_eq!(ev.source_entity, entity);
    }
}

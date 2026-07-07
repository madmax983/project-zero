use crate::layer1::core::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct EmMachine {
    pub active: bool,
}

#[derive(Component, Clone, Copy)]
pub struct Emissions {
    pub em_level: f32,
}

#[derive(Component, Clone, Copy)]
pub struct DataFauna {
    pub mass: f32,
    pub capacity: f32,
    pub species: DataFaunaSpecies,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DataFaunaSpecies {
    StaticMites,
    LogicLeeches,
}

#[derive(Component, Clone, Copy)]
pub struct EmSensor {
    pub active: bool,
}

#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Event, Clone, Copy)]
pub struct ShortCircuitEvent {
    pub target: Entity,
}

// Systems

pub fn spawn_data_fauna(
    mut commands: Commands,
    query: Query<(&Emissions, &GridPosition)>,
    fauna_query: Query<&DataFauna>,
) {
    let fauna_count = fauna_query.iter().count();
    if fauna_count >= 100 {
        return;
    }

    for (em, pos) in query.iter() {
        if em.em_level > 10.0 {
            commands.spawn((
                DataFauna {
                    mass: 1.0,
                    capacity: 10.0,
                    species: DataFaunaSpecies::StaticMites,
                },
                Visibility::Hidden,
                GridPosition { x: pos.x, y: pos.y },
            ));
        }
    }
}

pub fn data_fauna_feeding(
    mut commands: Commands,
    mut fauna_query: Query<(Entity, &mut DataFauna, &GridPosition)>,
    em_query: Query<(&Emissions, &GridPosition)>,
) {
    let mut em_map = std::collections::HashMap::new();
    for (em, e_pos) in em_query.iter() {
        *em_map.entry(*e_pos).or_insert(0.0) += em.em_level;
    }

    for (entity, mut fauna, f_pos) in fauna_query.iter_mut() {
        if let Some(level) = em_map.get(f_pos) {
            fauna.mass += level * 0.1;
        } else {
            fauna.mass -= 0.5;
            if fauna.mass <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn data_fauna_overfeed(
    mut commands: Commands,
    fauna_query: Query<(Entity, &DataFauna, &GridPosition)>,
    machine_query: Query<(Entity, &EmMachine, &GridPosition)>,
    mut ev_short_circuit: EventWriter<ShortCircuitEvent>,
) {
    let mut fauna_map: std::collections::HashMap<GridPosition, Vec<(Entity, f32, f32)>> =
        std::collections::HashMap::new();
    for (f_entity, fauna, f_pos) in fauna_query.iter() {
        fauna_map
            .entry(*f_pos)
            .or_default()
            .push((f_entity, fauna.mass, fauna.capacity));
    }

    for (m_entity, _machine, m_pos) in machine_query.iter() {
        if let Some(faunas) = fauna_map.get(m_pos) {
            for (f_entity, mass, capacity) in faunas {
                if *mass > *capacity {
                    ev_short_circuit.send(ShortCircuitEvent { target: m_entity });
                    commands.entity(*f_entity).despawn();
                }
            }
        }
    }
}

pub fn reveal_data_fauna(
    mut fauna_query: Query<(&mut Visibility, &GridPosition), With<DataFauna>>,
    sensor_query: Query<(&EmSensor, &GridPosition)>,
) {
    let mut active_sensors = Vec::new();
    for (sensor, s_pos) in sensor_query.iter() {
        if sensor.active {
            active_sensors.push(*s_pos);
        }
    }

    for (mut vis, f_pos) in fauna_query.iter_mut() {
        let mut revealed = false;
        for s_pos in &active_sensors {
            if (f_pos.x - s_pos.x).abs() <= 2 && (f_pos.y - s_pos.y).abs() <= 2 {
                revealed = true;
                break;
            }
        }

        *vis = if revealed {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}


/// INT-1319: Bridges ShortCircuitEvent to EmMachine deactivation and Chronicle
pub fn shadow_ecosystems_short_circuit_bridge(
    mut events: bevy_ecs::event::EventReader<ShortCircuitEvent>,
    mut machine_query: Query<&mut EmMachine>,
    mut chronicle_events: bevy_ecs::event::EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
) {
    for event in events.read() {
        if let Ok(mut machine) = machine_query.get_mut(event.target) {
            machine.active = false;
            chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                text: "A swarm of Data Fauna has overfed and short-circuited a machine!".to_string(),
                importance: crate::layer1::core::chronicle::EventImportance::Major,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_data_fauna_spawns_near_emissions() {
        let mut app = App::new();
        app.add_systems(Update, spawn_data_fauna);

        // Setup a high-tech emitter
        app.world_mut().spawn((
            EmMachine { active: true },
            Emissions { em_level: 50.0 },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        // Assert a DataFauna entity is created nearby
        let mut query = app.world_mut().query::<&DataFauna>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "Data-Fauna should spawn near high emissions"
        );
    }

    #[test]
    fn test_data_fauna_feeds_and_grows() {
        let mut app = App::new();
        app.add_systems(Update, data_fauna_feeding);

        // Setup fauna and emitter
        app.world_mut()
            .spawn((Emissions { em_level: 20.0 }, GridPosition { x: 2, y: 2 }));

        app.world_mut().spawn((
            DataFauna {
                mass: 1.0,
                capacity: 10.0,
                species: DataFaunaSpecies::StaticMites,
            },
            GridPosition { x: 2, y: 2 },
        ));

        app.update();

        // Assert the fauna's mass increased by feeding on emissions
        let mut query = app.world_mut().query::<&DataFauna>();
        let fauna = query.single(app.world());
        assert!(fauna.mass > 1.0, "Fauna mass should increase after feeding");
    }

    #[test]
    fn test_data_fauna_overfeeds_and_short_circuits() {
        let mut app = App::new();
        app.add_event::<ShortCircuitEvent>();
        app.add_systems(Update, data_fauna_overfeed);

        // Setup a fully-fed fauna near a machine
        app.world_mut()
            .spawn((EmMachine { active: true }, GridPosition { x: 3, y: 3 }));

        app.world_mut().spawn((
            DataFauna {
                mass: 15.0,
                capacity: 10.0,
                species: DataFaunaSpecies::StaticMites,
            }, // Over capacity
            GridPosition { x: 3, y: 3 },
        ));

        app.update();

        // A ShortCircuitEvent should be fired
        let events = app.world().resource::<Events<ShortCircuitEvent>>();
        let reader = events.get_cursor();
        assert!(
            reader.len(&events) > 0,
            "Overfed fauna should trigger a ShortCircuitEvent"
        );
    }

    #[test]
    fn test_data_fauna_visibility_requires_sensor() {
        let mut app = App::new();
        app.add_systems(Update, reveal_data_fauna);

        let fauna = app
            .world_mut()
            .spawn((
                DataFauna {
                    mass: 1.0,
                    capacity: 10.0,
                    species: DataFaunaSpecies::StaticMites,
                },
                Visibility::Hidden,
                GridPosition { x: 1, y: 1 },
            ))
            .id();

        // No sensor initially
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fauna).unwrap(),
            &Visibility::Hidden
        );

        // Add EM sensor
        app.world_mut().spawn((
            EmSensor { active: true },
            GridPosition { x: 1, y: 1 }, // Nearby
        ));

        app.update();
        assert_eq!(
            app.world().get::<Visibility>(fauna).unwrap(),
            &Visibility::Visible,
            "Fauna should become visible when sensor is active nearby"
        );
    }
}

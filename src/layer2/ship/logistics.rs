use bevy::prelude::*;

// We use Ship from crate::layer2::ship::Ship instead of redefining it
// #[derive(Component)]
// pub struct Ship;

#[derive(Component)]
pub struct DeltaV {
    pub current: f32,
    pub max: f32,
    pub consumption_rate: f32,
}

#[derive(Component)]
pub struct Moving {
    pub destination: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Stranded;

#[derive(Component)]
pub struct DistressBeacon;

#[derive(Event)]
pub struct StrandedEvent {
    pub ship: Entity,
}

#[derive(Component)]
pub struct Tanker {
    pub fuel_payload: f32,
}

#[derive(Component)]
pub struct RefuelingTarget {
    pub target: Entity,
}

pub fn movement_consumes_delta_v_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeltaV), With<Moving>>,
    mut stranded_events: EventWriter<StrandedEvent>,
) {
    for (entity, mut delta_v) in query.iter_mut() {
        delta_v.current -= delta_v.consumption_rate;
        if delta_v.current <= 0.0 {
            delta_v.current = 0.0;
            commands.entity(entity).remove::<Moving>().insert(Stranded);
            stranded_events.send(StrandedEvent { ship: entity });
        }
    }
}

pub fn stranded_system(mut commands: Commands, query: Query<Entity, Added<Stranded>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(DistressBeacon);
    }
}

pub fn refueling_system(
    mut commands: Commands,
    mut tankers: Query<(Entity, &mut Tanker, &RefuelingTarget, &Position)>,
    mut stranded: Query<(&mut DeltaV, &Position), With<Stranded>>,
) {
    for (tanker_entity, mut tanker, target, tanker_pos) in tankers.iter_mut() {
        if let Ok((mut delta_v, stranded_pos)) = stranded.get_mut(target.target) {
            if tanker_pos.0.distance(stranded_pos.0) < 1.0 && tanker.fuel_payload > 0.0 {
                let fuel_needed = delta_v.max - delta_v.current;
                let fuel_transferred = fuel_needed.min(tanker.fuel_payload);

                delta_v.current += fuel_transferred;
                tanker.fuel_payload -= fuel_transferred;

                if delta_v.current > 0.0 {
                    commands
                        .entity(target.target)
                        .remove::<Stranded>()
                        .remove::<DistressBeacon>();
                }

                if tanker.fuel_payload <= 0.0 {
                    commands.entity(tanker_entity).remove::<RefuelingTarget>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_consumes_delta_v_on_movement() {
        // Arrange
        let mut app = App::new();
        app.add_event::<StrandedEvent>();
        app.add_systems(Update, movement_consumes_delta_v_system);

        let ship = app
            .world_mut()
            .spawn((
                crate::layer2::ship::Ship::new(crate::layer2::ship::ShipType::Scout),
                DeltaV {
                    current: 100.0,
                    max: 100.0,
                    consumption_rate: 10.0,
                },
                Moving {
                    destination: Vec2::new(10.0, 0.0),
                    speed: 1.0,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let delta_v = app.world().get::<DeltaV>(ship).unwrap();
        assert_eq!(delta_v.current, 90.0);
    }

    #[test]
    fn test_ship_becomes_stranded_when_delta_v_depleted() {
        // Arrange
        let mut app = App::new();
        app.add_event::<StrandedEvent>();
        app.add_systems(Update, (movement_consumes_delta_v_system, stranded_system));

        let ship = app
            .world_mut()
            .spawn((
                crate::layer2::ship::Ship::new(crate::layer2::ship::ShipType::Scout),
                DeltaV {
                    current: 5.0,
                    max: 100.0,
                    consumption_rate: 10.0,
                },
                Moving {
                    destination: Vec2::new(10.0, 0.0),
                    speed: 1.0,
                },
            ))
            .id();

        // Act
        app.update(); // Depletes to 0, adds Stranded
        app.update(); // Next tick handles stranded

        // Assert
        let stranded = app.world().get::<Stranded>(ship);
        assert!(stranded.is_some());

        let distress_beacon = app.world().get::<DistressBeacon>(ship);
        assert!(distress_beacon.is_some());
    }

    #[test]
    fn test_tanker_refuels_stranded_ship() {
        // Arrange
        let mut app = App::new();
        app.add_event::<StrandedEvent>();
        app.add_systems(Update, refueling_system);

        let stranded_ship = app
            .world_mut()
            .spawn((
                crate::layer2::ship::Ship::new(crate::layer2::ship::ShipType::Scout),
                Stranded,
                DistressBeacon,
                DeltaV {
                    current: 0.0,
                    max: 100.0,
                    consumption_rate: 10.0,
                },
                Position(Vec2::new(5.0, 5.0)),
            ))
            .id();

        let tanker = app
            .world_mut()
            .spawn((
                crate::layer2::ship::Ship::new(crate::layer2::ship::ShipType::Transport),
                Tanker { fuel_payload: 50.0 },
                Position(Vec2::new(5.0, 5.0)),
                RefuelingTarget {
                    target: stranded_ship,
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let stranded_ship_delta_v = app.world().get::<DeltaV>(stranded_ship).unwrap();
        assert_eq!(stranded_ship_delta_v.current, 50.0);
        assert!(app.world().get::<Stranded>(stranded_ship).is_none());
        assert!(app.world().get::<DistressBeacon>(stranded_ship).is_none());

        let tanker_payload = app.world().get::<Tanker>(tanker).unwrap();
        assert_eq!(tanker_payload.fuel_payload, 0.0);
    }
}

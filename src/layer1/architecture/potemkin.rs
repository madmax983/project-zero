use crate::layer1::architecture::building::Building;
use crate::layer1::architecture::structure::Structure;
use crate::layer3::pirates::PirateThreatLevel;
use bevy::prelude::*;

#[derive(Component)]
pub struct PotemkinArchitecture;

#[derive(Component)]
pub struct IntimidationValue {
    pub value: f32,
}

pub fn calculate_intimidation_system(
    query: Query<&IntimidationValue, With<Building>>,
    mut threat_system: ResMut<PirateThreatLevel>,
    time: Res<Time>,
) {
    let mut total_intimidation = 0.0;
    for intimidation in query.iter() {
        total_intimidation += intimidation.value;
    }

    // Reduce level by intimidation point
    threat_system.level = (threat_system.level - total_intimidation * time.delta_secs()).max(0.0);
}

#[derive(Event)]
pub struct PotemkinDestroyedEvent {
    pub entity: Entity,
}

pub fn destroy_potemkin_building_system(
    mut commands: Commands,
    query: Query<(Entity, &Structure), With<PotemkinArchitecture>>,
    mut events: EventWriter<PotemkinDestroyedEvent>,
) {
    for (entity, structure) in query.iter() {
        if structure.current_hp <= 0.0 {
            events.send(PotemkinDestroyedEvent { entity });
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::architecture::structure::Structure;
    use std::time::Duration;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, calculate_intimidation_system);
        app.add_systems(Update, destroy_potemkin_building_system);
        app.add_event::<PotemkinDestroyedEvent>();
        app.insert_resource(PirateThreatLevel { level: 10.0 });
        let mut time = Time::new_with(());
        time.advance_by(Duration::from_secs(1));
        app.insert_resource(time);
        app
    }

    #[test]
    fn test_potemkin_building_has_1_hp_and_increases_intimidation() {
        let mut app = setup_app();

        // Spawn a real turret
        let _real_turret = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                IntimidationValue { value: 2.0 },
            ))
            .id();

        // Spawn a fake turret
        let _fake_turret = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                PotemkinArchitecture,
                Structure {
                    current_hp: 1.0,
                    max_hp: 1.0,
                },
                IntimidationValue { value: 3.0 }, // Looks like the real thing
            ))
            .id();

        app.update();

        let threat = app.world().resource::<PirateThreatLevel>();

        // level should be reduced by both (5 * 1 sec)
        assert_eq!(threat.level, 5.0);
    }

    #[test]
    fn test_potemkin_building_destroys_instantly() {
        let mut app = setup_app();

        // Spawn a fake turret
        let fake_turret = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                PotemkinArchitecture,
                Structure {
                    current_hp: 1.0,
                    max_hp: 1.0,
                },
            ))
            .id();

        app.update();
        assert!(app.world().get_entity(fake_turret).is_ok());

        // Take 2 damage
        let mut health = app.world_mut().get_mut::<Structure>(fake_turret).unwrap();
        health.current_hp -= 2.0;

        app.update();

        // Ensure entity is destroyed
        assert!(app.world().get_entity(fake_turret).is_err());
    }
}

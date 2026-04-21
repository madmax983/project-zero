use crate::layer1::social::Morale;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct OrbitalNecropolis {
    pub colony_id: Entity,
}

#[derive(Component)]
pub struct DesecratedSkies;

#[derive(Event, Debug, Clone)]
pub struct EntityDestroyedEvent {
    pub entity: Entity,
}

pub fn apply_necropolis_bonus(query: Query<&OrbitalNecropolis>, mut colonies: Query<&mut Morale>) {
    for necropolis in query.iter() {
        if let Ok(mut morale) = colonies.get_mut(necropolis.colony_id) {
            morale.value = (morale.value + 0.1).min(1.0);
        }
    }
}

pub fn handle_necropolis_destruction(
    mut events: EventReader<EntityDestroyedEvent>,
    necropolis_query: Query<&OrbitalNecropolis>,
    mut commands: Commands,
    mut colonies: Query<&mut Morale>,
) {
    for event in events.read() {
        if let Ok(necropolis) = necropolis_query.get(event.entity) {
            commands
                .entity(necropolis.colony_id)
                .insert(DesecratedSkies);
            if let Ok(mut morale) = colonies.get_mut(necropolis.colony_id) {
                morale.value = (morale.value - 0.5).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::Health;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_necropolis_provides_bonus() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_necropolis_bonus);

        let colony_entity = app
            .world_mut()
            .spawn(Morale {
                value: 0.5,
                ..default()
            })
            .id();

        let _necropolis = app
            .world_mut()
            .spawn((
                OrbitalNecropolis {
                    colony_id: colony_entity,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
            ))
            .id();

        // Act
        app.update();

        // Assert
        let morale = app.world().get::<Morale>(colony_entity).unwrap();
        assert!(morale.value > 0.5);
    }

    #[test]
    fn test_destroyed_necropolis_applies_desecrated_skies() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, handle_necropolis_destruction);
        app.add_event::<EntityDestroyedEvent>();

        let colony_entity = app
            .world_mut()
            .spawn(Morale {
                value: 0.8,
                ..default()
            })
            .id();

        let necropolis = app
            .world_mut()
            .spawn((OrbitalNecropolis {
                colony_id: colony_entity,
            },))
            .id();

        // Act
        app.world_mut()
            .send_event(EntityDestroyedEvent { entity: necropolis });
        app.update();

        // Assert
        let desecrated = app.world().get::<DesecratedSkies>(colony_entity);
        assert!(desecrated.is_some());
        let morale = app.world().get::<Morale>(colony_entity).unwrap();
        assert!(morale.value < 0.8);
    }
}

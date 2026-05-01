use bevy::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::biology::health::Health;

#[derive(Component)]
pub struct LivingBuilding {
    pub hunger: f32,
    pub heal_rate: f32,
}

#[derive(Event)]
pub struct PopConsumedEvent {
    pub pop: Entity,
    pub building: Entity,
}

pub fn living_building_healing_system(
    mut query: Query<(&LivingBuilding, &mut Health)>,
) {
    for (living, mut health) in query.iter_mut() {
        if living.hunger < 80.0 { // Arbitrary threshold for not starving
            health.current = (health.current + living.heal_rate).min(health.max);
        }
    }
}

pub fn living_building_consume_pop_system(
    mut commands: Commands,
    mut buildings: Query<(Entity, &Transform, &mut LivingBuilding)>,
    pops: Query<(Entity, &Transform), With<Pop>>,
    mut events: EventWriter<PopConsumedEvent>,
) {
    for (building_entity, building_transform, mut living_building) in buildings.iter_mut() {
        if living_building.hunger >= 80.0 {
            for (pop_entity, pop_transform) in pops.iter() {
                if building_transform.translation.distance(pop_transform.translation) < 2.0 {
                    // Consume pop
                    commands.entity(pop_entity).despawn();
                    living_building.hunger = 0.0;
                    events.send(PopConsumedEvent {
                        pop: pop_entity,
                        building: building_entity,
                    });
                    break; // Only eat one per tick
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::biology::health::Health;

    #[test]
    fn test_living_building_heals_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, living_building_healing_system);

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 0.0,
                heal_rate: 5.0,
            },
            Health {
                current: 50.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(health.current, 55.0);
    }

    #[test]
    fn test_living_building_starves_and_stops_healing() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, living_building_healing_system);

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 100.0, // Starving
                heal_rate: 5.0,
            },
            Health {
                current: 50.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(health.current, 50.0); // No healing
    }

    #[test]
    fn test_living_building_eats_pop_when_starving() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopConsumedEvent>();
        app.add_systems(Update, living_building_consume_pop_system);

        let pop = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 100.0, // Starving
                heal_rate: 5.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get_entity(pop).is_err()); // Pop was consumed
        let living_building = app.world().get::<LivingBuilding>(building).unwrap();
        assert!(living_building.hunger < 100.0); // Hunger decreased

        let events = app.world().resource::<Events<PopConsumedEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit PopConsumedEvent");
    }
}

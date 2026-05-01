use bevy::prelude::*;
use crate::layer1::entities::pop::Pop;
use crate::layer1::biology::health::Health;
use crate::layer1::core::map::GridPosition;
use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::core::events::PopConsumedEvent;
use std::collections::HashSet;

#[derive(Component, Default)]
pub struct LivingBuilding {
    pub hunger: f32,
    pub heal_rate: f32,
}

pub fn living_building_healing_system(
    mut query: Query<(&LivingBuilding, &mut Health)>,
) {
    for (living, mut health) in query.iter_mut() {
        if living.hunger < 80.0 {
            health.current = (health.current + living.heal_rate).min(health.max);
        }
    }
}

pub fn living_building_consume_pop_system(
    mut commands: Commands,
    mut buildings: Query<(Entity, &GridPosition, &mut LivingBuilding)>,
    pops: Query<(Entity, &GridPosition), With<Pop>>,
    mut events: EventWriter<PopConsumedEvent>,
) {
    let mut consumed_pops = HashSet::new();
    for (building_entity, building_pos, mut living_building) in buildings.iter_mut() {
        if living_building.hunger >= 80.0 {
            for (pop_entity, pop_pos) in pops.iter() {
                if !consumed_pops.contains(&pop_entity) && building_pos.distance_chebyshev(*pop_pos) <= 1 {
                    commands.entity(pop_entity).despawn();
                    consumed_pops.insert(pop_entity);
                    living_building.hunger = 0.0;
                    events.send(PopConsumedEvent {
                        pop: pop_entity,
                        building: building_entity,
                    });
                    break;
                }
            }
        }
    }
}

pub fn living_building_consume_resources_system(
    mut buildings: Query<&mut LivingBuilding>,
    mut resources: ResMut<ColonyResources>,
) {
    for mut living_building in buildings.iter_mut() {
        if resources.food >= 1.0 {
            resources.food -= 1.0;
            living_building.hunger = (living_building.hunger - 10.0).max(0.0);
        } else {
            living_building.hunger = (living_building.hunger + 5.0).min(100.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_living_building_heals_over_time() {
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

        app.update();

        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(health.current, 55.0);
    }

    #[test]
    fn test_living_building_starves_and_stops_healing() {
        let mut app = App::new();
        app.add_systems(Update, living_building_healing_system);

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 100.0,
                heal_rate: 5.0,
            },
            Health {
                current: 50.0,
                max: 100.0,
                has_rust_lung: false,
            },
        )).id();

        app.update();

        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(health.current, 50.0);
    }

    #[test]
    fn test_living_building_eats_pop_when_starving() {
        let mut app = App::new();
        app.add_event::<PopConsumedEvent>();
        app.add_systems(Update, living_building_consume_pop_system);

        let pop = app.world_mut().spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
        )).id();

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 100.0,
                heal_rate: 5.0,
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.update();

        assert!(app.world().get_entity(pop).is_err());
        let living_building = app.world().get::<LivingBuilding>(building).unwrap();
        assert!(living_building.hunger < 100.0);

        let events = app.world().resource::<Events<PopConsumedEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);
    }

    #[test]
    fn test_living_building_consumes_resources() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });
        app.add_systems(Update, living_building_consume_resources_system);

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 50.0,
                heal_rate: 5.0,
            },
        )).id();

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.food, 9.0);

        let living_building = app.world().get::<LivingBuilding>(building).unwrap();
        assert_eq!(living_building.hunger, 40.0);
    }

    #[test]
    fn test_living_building_starves_without_resources() {
        let mut app = App::new();
        app.insert_resource(ColonyResources {
            food: 0.0,
            ..Default::default()
        });
        app.add_systems(Update, living_building_consume_resources_system);

        let building = app.world_mut().spawn((
            LivingBuilding {
                hunger: 50.0,
                heal_rate: 5.0,
            },
        )).id();

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.food, 0.0);

        let living_building = app.world().get::<LivingBuilding>(building).unwrap();
        assert_eq!(living_building.hunger, 55.0);
    }
}

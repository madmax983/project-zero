use bevy_ecs::prelude::*;
use crate::layer1::biology::health::Health;
use crate::layer1::entities::pop::Pop;
use crate::layer1::core::events::PopConsumedEvent;
use crate::layer1::core::map::GridPosition;

#[derive(Component)]
pub struct LivingBuilding {
    pub hunger: f32,
    pub heal_rate: f32,
}

pub fn living_building_healing_system(
    mut query: Query<(&LivingBuilding, &mut Health)>,
) {
    for (living, mut health) in query.iter_mut() {
        if living.hunger < 80.0 {
            // Note: Since Health::take_damage clamps at 0, we can heal directly or carefully.
            health.current = (health.current + living.heal_rate).min(health.max);
        }
    }
}

pub fn living_building_consume_pop_system(
    mut commands: Commands,
    mut events: EventWriter<PopConsumedEvent>,
    mut buildings: Query<(Entity, &GridPosition, &mut LivingBuilding)>,
    pops: Query<(Entity, &GridPosition), With<Pop>>,
) {
    let mut consumed_pops = std::collections::HashSet::new();

    for (building_ent, building_pos, mut living_building) in buildings.iter_mut() {
        if living_building.hunger >= 80.0 {
            for (pop_entity, pop_pos) in pops.iter() {
                if consumed_pops.contains(&pop_entity) {
                    continue;
                }

                let dx = (building_pos.x as f32) - (pop_pos.x as f32);
                let dy = (building_pos.y as f32) - (pop_pos.y as f32);
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < 2.0 {
                    // Consume pop
                    commands.entity(pop_entity).despawn();
                    consumed_pops.insert(pop_entity);
                    living_building.hunger = 0.0;
                    events.send(PopConsumedEvent {
                        pop: pop_entity,
                        building: building_ent,
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
    use bevy_app::App;
    use bevy_app::Update;

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
                hunger: 100.0, // Starving
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
        assert_eq!(health.current, 50.0); // No healing
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
                hunger: 100.0, // Starving
                heal_rate: 5.0,
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        app.update();

        assert!(app.world().get_entity(pop).is_err()); // Pop was consumed
        let living_building = app.world().get::<LivingBuilding>(building).unwrap();
        assert!(living_building.hunger < 100.0); // Hunger decreased

        let events = app.world().resource::<Events<PopConsumedEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit PopConsumedEvent");
    }
}

use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct AntiGravGenerator {
    pub debt_generation_rate: f32,
    pub max_safe_debt: f32,
}

#[derive(Component, Default)]
pub struct GravitationalDebt {
    pub accumulated_debt: f32,
}

#[derive(Event)]
pub struct DebtReleaseEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
    pub debt_amount: f32,
}

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;
use bevy_time::Time;

pub fn gravitational_debt_accumulation_system(
    mut query: Query<(&AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt)>,
    time: Res<Time>,
) {
    for (generator, powered, mut debt) in query.iter_mut() {
        if powered.active {
            debt.accumulated_debt += generator.debt_generation_rate * time.delta_secs();
        }
    }
}

pub fn gravitational_debt_release_system(
    mut query: Query<(
        Entity,
        &AntiGravGenerator,
        &PowerConsumer,
        &mut GravitationalDebt,
        &GridPosition,
    )>,
    mut release_events: EventWriter<DebtReleaseEvent>,
) {
    for (entity, generator, powered, mut debt, pos) in query.iter_mut() {
        if debt.accumulated_debt > 0.0
            && (!powered.active || debt.accumulated_debt > generator.max_safe_debt)
        {
            // Release debt
            release_events.send(DebtReleaseEvent {
                source_entity: entity,
                position: *pos,
                debt_amount: debt.accumulated_debt,
            });

            // Reset debt
            debt.accumulated_debt = 0.0;
        }
    }
}

use bevy::prelude::DespawnRecursiveExt;
use crate::layer1::health::Health;

pub fn process_debt_release_system(
    mut commands: Commands,
    mut release_events: EventReader<DebtReleaseEvent>,
    mut chronicle: ResMut<Chronicle>,
    time: Res<SimulationTime>,
    mut health_query: Query<(Entity, &GridPosition, &mut Health)>,
    building_query: Query<(Entity, &GridPosition), With<crate::layer1::building::Building>>,
) {
    for event in release_events.read() {
        chronicle.add_event(
            time.tick,
            format!(
                "A localized gravitational debt of {:.1} collapsed at ({}, {})! The area was crushed.",
                event.debt_amount, event.position.x, event.position.y
            ),
            EventImportance::Standard,
        );

        let radius = (event.debt_amount / 20.0).max(1.0) as i32;

        // Damage pops
        for (_, pos, mut health) in health_query.iter_mut() {
            if pos.distance_chebyshev(event.position) <= radius as u32 {
                health.take_damage(event.debt_amount);
            }
        }

        // Destroy buildings
        for (entity, pos) in building_query.iter() {
            // Exclude the generator itself to prevent concurrent despawn issues,
            // though it usually makes sense to destroy it too. We'll destroy it.
            if pos.distance_chebyshev(event.position) <= radius as u32 {
                if let Some(ec) = commands.get_entity(entity) {
                    ec.despawn_recursive();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::map::GridPosition;
    use bevy_app::{App, Update};

    use crate::layer1::chronicle::Chronicle;
    use crate::shared::time::SimulationTime;
    use bevy_time::TimePlugin;

    #[test]
    fn test_gravitational_debt_accumulates_when_powered() {
        let mut app = App::new();
        app.add_plugins(TimePlugin);
        app.add_systems(Update, gravitational_debt_accumulation_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let generator_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::AntiGravGenerator,
                },
                AntiGravGenerator {
                    debt_generation_rate: 5.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: true,
                    ..Default::default()
                },
                GravitationalDebt {
                    accumulated_debt: 0.0,
                },
                pos,
            ))
            .id();

        // Act
        // Tick time multiple times to allow delta_seconds to be > 0
        app.update();
        app.update();

        // Assert
        let debt = app
            .world()
            .get::<GravitationalDebt>(generator_entity)
            .unwrap();
        assert!(
            debt.accumulated_debt > 0.0,
            "Debt should accumulate when generator is powered"
        );
    }

    #[test]
    fn test_gravitational_debt_releases_when_unpowered() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let generator_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::AntiGravGenerator,
                },
                AntiGravGenerator {
                    debt_generation_rate: 5.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: false,
                    ..Default::default()
                },
                GravitationalDebt {
                    accumulated_debt: 50.0,
                }, // Accumulated some debt
                pos,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(
            release_events.len(),
            1,
            "DebtReleaseEvent should be fired when generator loses power"
        );
        assert_eq!(release_events[0].source_entity, generator_entity);
        assert_eq!(release_events[0].debt_amount, 50.0);

        // Check debt is reset
        let debt = app
            .world()
            .get::<GravitationalDebt>(generator_entity)
            .unwrap();
        assert_eq!(
            debt.accumulated_debt, 0.0,
            "Debt should reset after release"
        );
    }

    #[test]
    fn test_gravitational_debt_releases_when_exceeding_max() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let _generator_entity = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::AntiGravGenerator,
                },
                AntiGravGenerator {
                    debt_generation_rate: 5.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: true,
                    ..Default::default()
                }, // Powered, but over max debt
                GravitationalDebt {
                    accumulated_debt: 105.0,
                },
                pos,
            ))
            .id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(
            release_events.len(),
            1,
            "DebtReleaseEvent should be fired when debt exceeds max_safe_debt"
        );
        assert_eq!(release_events[0].debt_amount, 105.0);
    }

    #[test]
    fn test_process_debt_release_logs_to_chronicle() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.init_resource::<Chronicle>();
        app.init_resource::<SimulationTime>();
        app.add_systems(Update, process_debt_release_system);

        let pos = GridPosition { x: 5, y: 5 };
        app.world_mut()
            .resource_mut::<Events<DebtReleaseEvent>>()
            .send(DebtReleaseEvent {
                source_entity: Entity::PLACEHOLDER,
                position: pos,
                debt_amount: 100.0,
            });

        app.update();

        let chronicle = app.world().resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1, "Should log 1 event");
        assert!(chronicle.events[0].text.contains("100.0 collapsed"));
    }
}

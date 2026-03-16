use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::map::GridPosition;
use crate::layer1::energy::PowerConsumer;

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

pub fn gravitational_debt_accumulation_system(
    _time: Res<SimulationTime>,
    mut query: Query<(&AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt)>,
) {
    let delta = 1.0;

    for (generator, powered, mut debt) in query.iter_mut() {
        if powered.active {
            debt.accumulated_debt += generator.debt_generation_rate * delta;
        }
    }
}

pub fn gravitational_debt_release_system(
    mut query: Query<(Entity, &AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt, &GridPosition)>,
    mut release_events: EventWriter<DebtReleaseEvent>,
) {
    for (entity, generator, powered, mut debt, pos) in query.iter_mut() {
        if debt.accumulated_debt > 0.0 && (!powered.active || debt.accumulated_debt > generator.max_safe_debt) {
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

pub fn process_gravitational_debt_release_system(
    mut events: EventReader<DebtReleaseEvent>,
    mut health_query: Query<(&GridPosition, &mut crate::layer1::health::Health)>,
) {
    for event in events.read() {
        let impact_radius = (event.debt_amount / 10.0).sqrt() as i32; // basic formula for radius

        for (pos, mut health) in health_query.iter_mut() {
            let dx = pos.x - event.position.x;
            let dy = pos.y - event.position.y;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();

            if distance <= impact_radius as f32 {
                health.take_damage(event.debt_amount);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::energy::PowerConsumer;
    use crate::layer1::health::Health;

    #[test]
    fn test_gravitational_debt_accumulates_when_powered() {
        let mut app = App::new();
        // And manually advance time
        let time = crate::shared::time::SimulationTime::default();

        app.insert_resource(time);

        app.add_systems(Update, gravitational_debt_accumulation_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let generator_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::AntiGravGenerator },
            AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
            PowerConsumer { active: true, demand: 5.0 },
            GravitationalDebt { accumulated_debt: 0.0 },
            pos,
        )).id();

        // Act
        app.update();

        // Assert
        let debt = app.world().get::<GravitationalDebt>(generator_entity).unwrap();
        assert!(debt.accumulated_debt > 0.0, "Debt should accumulate when generator is powered");
    }

    #[test]
    fn test_gravitational_debt_releases_when_unpowered() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let generator_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::AntiGravGenerator },
            AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
            PowerConsumer { active: false, demand: 5.0 },
            GravitationalDebt { accumulated_debt: 50.0 }, // Accumulated some debt
            pos,
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(release_events.len(), 1, "DebtReleaseEvent should be fired when generator loses power");
        assert_eq!(release_events[0].source_entity, generator_entity);
        assert_eq!(release_events[0].debt_amount, 50.0);

        // Check debt is reset
        let debt = app.world().get::<GravitationalDebt>(generator_entity).unwrap();
        assert_eq!(debt.accumulated_debt, 0.0, "Debt should reset after release");
    }

    #[test]
    fn test_gravitational_debt_releases_when_exceeding_max() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let _generator_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::AntiGravGenerator },
            AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
            PowerConsumer { active: true, demand: 5.0 }, // Powered, but over max debt
            GravitationalDebt { accumulated_debt: 105.0 },
            pos,
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(release_events.len(), 1, "DebtReleaseEvent should be fired when debt exceeds max_safe_debt");
        assert_eq!(release_events[0].debt_amount, 105.0);
    }

    #[test]
    fn test_process_gravitational_debt_release_system_applies_damage() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, process_gravitational_debt_release_system);

        let entity = app.world_mut().spawn((
            GridPosition { x: 10, y: 10 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        app.world_mut().resource_mut::<Events<DebtReleaseEvent>>().send(DebtReleaseEvent {
            source_entity: Entity::PLACEHOLDER,
            position: GridPosition { x: 10, y: 10 },
            debt_amount: 50.0,
        });

        app.update();

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(health.current < 100.0, "Health should have decreased due to damage from gravitational debt release");
    }
}

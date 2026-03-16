use bevy::prelude::*;
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
    mut query: Query<(&AntiGravGenerator, &PowerConsumer, &mut GravitationalDebt)>,
    time: Option<Res<Time>>,
) {
    let dt = time.map_or(0.1, |t| t.delta_secs());
    for (generator, powered, mut debt) in &mut query {
        if powered.active {
            debt.accumulated_debt += generator.debt_generation_rate * dt;
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
    for (entity, generator, powered, mut debt, pos) in &mut query {
        // We add an epsilon to avoid floating point issues when checking if > 0.0
        if debt.accumulated_debt > f32::EPSILON
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

pub fn gravitational_debt_damage_system(
    mut events: EventReader<DebtReleaseEvent>,
    mut health_query: Query<(&GridPosition, &mut crate::layer1::health::Health)>,
) {
    for event in events.read() {
        let epicenter = event.position;
        // Radius scales with debt, max distance 10
        let radius = (event.debt_amount / 20.0).clamp(1.0, 10.0);

        for (pos, mut health) in &mut health_query {
            let dx = pos.x as f32 - epicenter.x as f32;
            let dy = pos.y as f32 - epicenter.y as f32;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= radius {
                // Damage falls off linearly with distance
                let falloff = 1.0 - (distance / radius);
                let damage = event.debt_amount * falloff;

                if damage > 0.0 {
                    health.take_damage(damage);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_gravitational_debt_accumulates_when_powered() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.add_systems(Update, gravitational_debt_accumulation_system);

        // Arrange
        let pos = GridPosition { x: 10, y: 10 };
        let generator_entity = app.world_mut().spawn((
            Building { building_type: BuildingType::AntiGravGenerator },
            AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
            PowerConsumer { active: true, ..Default::default() },
            GravitationalDebt { accumulated_debt: 0.0 },
            pos,
        )).id();

        // Act
        // Setup initial time frame
        app.update();

        // Tick time slightly so delta is > 0
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_secs(1));
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
            PowerConsumer { active: false, ..Default::default() },
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
            PowerConsumer { active: true, ..Default::default() }, // Powered, but over max debt
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
}

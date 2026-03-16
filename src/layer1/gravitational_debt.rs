use crate::layer1::energy::PowerConsumer;
use crate::layer1::map::GridPosition;
use bevy::prelude::*;

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
) {
    for (generator, consumer, mut debt) in query.iter_mut() {
        if consumer.active {
            debt.accumulated_debt += generator.debt_generation_rate;
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
    for (entity, generator, consumer, mut debt, pos) in query.iter_mut() {
        if debt.accumulated_debt > 0.0
            && (!consumer.active || debt.accumulated_debt > generator.max_safe_debt)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulation_active() {
        let mut app = App::new();
        app.add_systems(Update, gravitational_debt_accumulation_system);

        let pos = GridPosition { x: 0, y: 0 };
        let entity = app
            .world_mut()
            .spawn((
                AntiGravGenerator {
                    debt_generation_rate: 2.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                },
                GravitationalDebt {
                    accumulated_debt: 0.0,
                },
                pos,
            ))
            .id();

        app.update();

        let debt = app.world().get::<GravitationalDebt>(entity).unwrap();
        assert_eq!(debt.accumulated_debt, 2.0);
    }

    #[test]
    fn test_accumulation_inactive() {
        let mut app = App::new();
        app.add_systems(Update, gravitational_debt_accumulation_system);

        let pos = GridPosition { x: 0, y: 0 };
        let entity = app
            .world_mut()
            .spawn((
                AntiGravGenerator {
                    debt_generation_rate: 2.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: false,
                    demand: 10.0,
                },
                GravitationalDebt {
                    accumulated_debt: 0.0,
                },
                pos,
            ))
            .id();

        app.update();

        let debt = app.world().get::<GravitationalDebt>(entity).unwrap();
        assert_eq!(debt.accumulated_debt, 0.0);
    }

    #[test]
    fn test_release_inactive() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        let pos = GridPosition { x: 0, y: 0 };
        let entity = app
            .world_mut()
            .spawn((
                AntiGravGenerator {
                    debt_generation_rate: 2.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: false,
                    demand: 10.0,
                },
                GravitationalDebt {
                    accumulated_debt: 50.0,
                },
                pos,
            ))
            .id();

        app.update();

        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);

        let debt = app.world().get::<GravitationalDebt>(entity).unwrap();
        assert_eq!(debt.accumulated_debt, 0.0);
    }

    #[test]
    fn test_release_over_max() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        let pos = GridPosition { x: 0, y: 0 };
        let entity = app
            .world_mut()
            .spawn((
                AntiGravGenerator {
                    debt_generation_rate: 2.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                },
                GravitationalDebt {
                    accumulated_debt: 150.0,
                },
                pos,
            ))
            .id();

        app.update();

        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);

        let debt = app.world().get::<GravitationalDebt>(entity).unwrap();
        assert_eq!(debt.accumulated_debt, 0.0);
    }

    #[test]
    fn test_no_release_active_under_max() {
        let mut app = App::new();
        app.add_event::<DebtReleaseEvent>();
        app.add_systems(Update, gravitational_debt_release_system);

        let pos = GridPosition { x: 0, y: 0 };
        let entity = app
            .world_mut()
            .spawn((
                AntiGravGenerator {
                    debt_generation_rate: 2.0,
                    max_safe_debt: 100.0,
                },
                PowerConsumer {
                    active: true,
                    demand: 10.0,
                },
                GravitationalDebt {
                    accumulated_debt: 50.0,
                },
                pos,
            ))
            .id();

        app.update();

        let events = app.world().resource::<Events<DebtReleaseEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 0);

        let debt = app.world().get::<GravitationalDebt>(entity).unwrap();
        assert_eq!(debt.accumulated_debt, 50.0);
    }
}

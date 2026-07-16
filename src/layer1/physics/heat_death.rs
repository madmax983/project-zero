use bevy::prelude::*;

#[derive(Component)]
pub struct HeatProducer {
    pub amount: f32,
}

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Component)]
pub struct InVacuum;

#[derive(Component)]
pub struct Radiator {
    pub cooling_capacity: f32,
}

#[derive(Component)]
pub struct ConnectedRadiator(pub Entity);

#[derive(Component)]
pub struct Operational(pub bool);

#[derive(Component)]
pub struct ScramThreshold(pub f32);

pub fn heat_accumulation_system(
    mut query: Query<(&HeatProducer, &mut Temperature, Option<&InVacuum>)>,
) {
    for (producer, mut temp, in_vacuum) in query.iter_mut() {
        if in_vacuum.is_some() {
            temp.current += producer.amount;
        } else {
            // In atmosphere, assume some passive dissipation (e.g. half)
            temp.current += producer.amount * 0.5;
        }
    }
}

pub fn radiator_cooling_system(
    mut query: Query<(&mut Temperature, &ConnectedRadiator)>,
    radiators: Query<(&Radiator, &Operational)>,
) {
    for (mut temp, connection) in query.iter_mut() {
        if let Ok((radiator, operational)) = radiators.get(connection.0) {
            if operational.0 {
                temp.current -= radiator.cooling_capacity;
                // Prevent negative absolute temperature (simplified)
                if temp.current < 0.0 { temp.current = 0.0; }
            }
        }
    }
}

pub fn overheat_scram_system(
    mut query: Query<(&mut Operational, &Temperature, &ScramThreshold)>,
) {
    for (mut op, temp, threshold) in query.iter_mut() {
        if temp.current > threshold.0 {
            op.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heat_accumulation_in_vacuum() {
        let mut app = App::new();
        app.add_systems(Update, heat_accumulation_system);

        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
            InVacuum,
        )).id();

        app.update();

        // In vacuum, all heat is retained
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 30.0);
    }

    #[test]
    fn test_heat_accumulation_in_atmosphere() {
        let mut app = App::new();
        app.add_systems(Update, heat_accumulation_system);

        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
        )).id();

        app.update();

        // In atmosphere, 50% heat retained
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 25.0);
    }


    #[test]
    fn test_radiator_dissipates_heat() {
        let mut app = App::new();
        app.add_systems(Update, (heat_accumulation_system, radiator_cooling_system).chain());

        // Spawn a machine connected to a radiator
        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
            InVacuum,
            ConnectedRadiator(Entity::PLACEHOLDER), // Handled via proper setup in real code
        )).id();

        let radiator = app.world_mut().spawn((
            Radiator { cooling_capacity: 15.0 },
            Operational(true),
        )).id();

        app.world_mut().entity_mut(machine).insert(ConnectedRadiator(radiator));

        app.update();

        // Heat produced is 10, cooling is 15. The temperature should drop or stay stable.
        // Assuming min temp is bounded or simple subtraction: 20 + 10 - 15 = 15.
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 15.0);
    }

    #[test]
    fn test_radiator_does_not_dissipate_when_not_operational() {
        let mut app = App::new();
        app.add_systems(Update, (heat_accumulation_system, radiator_cooling_system).chain());

        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
            InVacuum,
            ConnectedRadiator(Entity::PLACEHOLDER),
        )).id();

        let radiator = app.world_mut().spawn((
            Radiator { cooling_capacity: 15.0 },
            Operational(false),
        )).id();

        app.world_mut().entity_mut(machine).insert(ConnectedRadiator(radiator));

        app.update();

        // Heat produced is 10, cooling is 0 since not operational.
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 30.0);
    }

    #[test]
    fn test_radiator_cooling_below_zero_is_clamped() {
        let mut app = App::new();
        app.add_systems(Update, (heat_accumulation_system, radiator_cooling_system).chain());

        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 1.0 },
            InVacuum,
            ConnectedRadiator(Entity::PLACEHOLDER),
        )).id();

        let radiator = app.world_mut().spawn((
            Radiator { cooling_capacity: 15.0 },
            Operational(true),
        )).id();

        app.world_mut().entity_mut(machine).insert(ConnectedRadiator(radiator));

        app.update();

        // 1.0 + 10.0 - 15.0 < 0 => clamped to 0.0
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 0.0);
    }

    #[test]
    fn test_machine_scrams_on_overheat() {
        let mut app = App::new();
        app.add_systems(Update, overheat_scram_system);

        let machine = app.world_mut().spawn((
            Operational(true),
            Temperature { current: 1500.0 }, // Above SCRAM limit
            ScramThreshold(1000.0),
        )).id();

        app.update();

        assert_eq!(app.world().get::<Operational>(machine).unwrap().0, false);
    }

    #[test]
    fn test_machine_does_not_scram_when_under_limit() {
        let mut app = App::new();
        app.add_systems(Update, overheat_scram_system);

        let machine = app.world_mut().spawn((
            Operational(true),
            Temperature { current: 500.0 }, // Below SCRAM limit
            ScramThreshold(1000.0),
        )).id();

        app.update();

        assert_eq!(app.world().get::<Operational>(machine).unwrap().0, true);
    }
}

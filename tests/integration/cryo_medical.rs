#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::actions::{AssignedTo, AssignmentType};
    use scale::layer1::cryo_shock::{decay_cryo_shock_system, CryoShock};
    use scale::layer1::pop::Pop;

    #[test]
    fn test_cryo_shock_decay_faster_in_medical_bed() {
        let mut app = App::new();

        let hospital = app.world_mut().spawn_empty().id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                CryoShock {
                    duration_ticks: 100,
                    severity: 0.5,
                },
                AssignedTo {
                    entity: hospital,
                    assignment_type: AssignmentType::Patient,
                },
            ))
            .id();

        app.add_systems(Update, decay_cryo_shock_system);

        app.update();

        let shock = app.world().get::<CryoShock>(pop).unwrap();
        // Since it's a patient, it should decay by 5 instead of 1
        assert_eq!(shock.duration_ticks, 95, "Should decay by 5 for patients");
    }
}

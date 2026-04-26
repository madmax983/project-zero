use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct InsideChamber {
    pub chamber_entity: Entity,
    pub fractional_age: f32,
}

#[derive(Component)]
pub struct TemporalChamber {
    pub time_dilation_factor: f32,
    pub active: bool,
    pub energy_cost: f32,
    pub ticks_active: u32,
}

pub fn temporal_chamber_energy_system(
    mut chambers: Query<&mut TemporalChamber>,
) {
    for mut chamber in chambers.iter_mut() {
        if chamber.active {
            chamber.ticks_active += 1;
            // Exponential increase
            chamber.energy_cost = 10.0 * (1.01_f32).powi(chamber.ticks_active as i32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chamber_energy_cost_increases_over_time() {
        // Arrange
        let mut world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(temporal_chamber_energy_system);

        let chamber = world.spawn((
            TemporalChamber {
                time_dilation_factor: 0.1,
                active: true,
                energy_cost: 10.0,
                ticks_active: 0,
            },
        )).id();

        // Act
        schedule.run(&mut world);

        // Assert
        let updated_chamber = world.get::<TemporalChamber>(chamber).unwrap();
        assert!(updated_chamber.energy_cost > 10.0);
        assert_eq!(updated_chamber.ticks_active, 1);
    }
}

use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct Planet {
    pub base_gravity: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct GravityDebt {
    pub imported_mass: f32,
    pub exported_mass: f32,
}

impl GravityDebt {
    pub fn current_gravity_modifier(&self) -> f32 {
        let net_mass = self.imported_mass - self.exported_mass;
        // Simple linear scaling for minimal implementation
        1.0 + (net_mass * 0.0001).max(0.0)
    }
}

#[derive(Event)]
pub struct FreighterArrivalEvent {
    pub destination: Entity,
    pub mass_amount: f32,
}

#[derive(Event)]
pub struct FreighterDepartureEvent {
    pub source: Entity,
    pub mass_amount: f32,
}

pub fn accumulate_gravity_debt_system(
    mut arrival_events: EventReader<FreighterArrivalEvent>,
    mut departure_events: EventReader<FreighterDepartureEvent>,
    mut query: Query<&mut GravityDebt>,
) {
    for event in arrival_events.read() {
        if let Ok(mut debt) = query.get_mut(event.destination) {
            debt.imported_mass += event.mass_amount;
        }
    }

    for event in departure_events.read() {
        if let Ok(mut debt) = query.get_mut(event.source) {
            debt.exported_mass += event.mass_amount;
        }
    }
}

pub fn calculate_launch_cost(base_cost: f32, debt: &GravityDebt) -> f32 {
    base_cost * debt.current_gravity_modifier()
}

pub fn calculate_launch_failure_chance(base_chance: f32, debt: &GravityDebt) -> f32 {
    // Exponential or aggressive scaling for failure to force the tension
    base_chance * debt.current_gravity_modifier().powf(1.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gravity_debt_accumulation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, accumulate_gravity_debt_system);
        app.add_event::<FreighterArrivalEvent>();
        app.add_event::<FreighterDepartureEvent>();

        let planet_entity = app
            .world_mut()
            .spawn((
                Planet { base_gravity: 1.0 },
                GravityDebt {
                    imported_mass: 0.0,
                    exported_mass: 0.0,
                },
            ))
            .id();

        app.world_mut().send_event(FreighterArrivalEvent {
            destination: planet_entity,
            mass_amount: 5000.0,
        });

        app.world_mut().send_event(FreighterDepartureEvent {
            source: planet_entity,
            mass_amount: 1000.0,
        });

        // Act
        app.update();

        // Assert
        let debt = app.world().get::<GravityDebt>(planet_entity).unwrap();
        assert_eq!(debt.imported_mass, 5000.0);
        assert_eq!(debt.exported_mass, 1000.0);
        assert!(
            debt.current_gravity_modifier() > 1.0,
            "Importing more mass than exporting should increase gravity modifier."
        );
    }

    #[test]
    fn test_launch_cost_scales_with_debt() {
        // Arrange
        let base_cost = 100.0;
        let planet_debt = GravityDebt {
            imported_mass: 100000.0,
            exported_mass: 0.0,
        };

        // Act
        let modified_cost = calculate_launch_cost(base_cost, &planet_debt);

        // Assert
        assert!(
            modified_cost > base_cost,
            "Launch cost should be significantly higher due to accumulated mass"
        );
    }

    #[test]
    fn test_launch_failure_chance_increases() {
        let planet_debt = GravityDebt {
            imported_mass: 500000.0,
            exported_mass: 0.0,
        };
        let base_chance = 0.01;

        let failure_chance = calculate_launch_failure_chance(base_chance, &planet_debt);
        assert!(
            failure_chance > base_chance,
            "Failure chance must increase on high gravity debt planets."
        );
    }
}

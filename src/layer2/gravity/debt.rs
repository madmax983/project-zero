use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct PhysicsConstants {
    pub gravity_debt_modifier: f32,
}

impl Default for PhysicsConstants {
    fn default() -> Self {
        Self {
            gravity_debt_modifier: 0.0001,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(0.0)
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct Planet {
    pub base_gravity: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct GravityDebt {
    pub imported_mass: Mass,
    pub exported_mass: Mass,
}

impl GravityDebt {
    pub fn current_gravity_modifier(&self, constants: &PhysicsConstants) -> f32 {
        let net_mass = self.imported_mass.0 - self.exported_mass.0;
        1.0 + (net_mass * constants.gravity_debt_modifier).max(0.0)
    }
}

#[derive(Event)]
pub struct FreighterArrivalEvent {
    pub destination: Entity,
    pub mass_amount: Mass,
}

#[derive(Event)]
pub struct FreighterDepartureEvent {
    pub source: Entity,
    pub mass_amount: Mass,
}

pub fn accumulate_gravity_debt_system(
    mut arrival_events: EventReader<FreighterArrivalEvent>,
    mut departure_events: EventReader<FreighterDepartureEvent>,
    mut query: Query<&mut GravityDebt>,
) {
    for event in arrival_events.read() {
        if let Ok(mut debt) = query.get_mut(event.destination) {
            debt.imported_mass.0 += event.mass_amount.0;
        }
    }

    for event in departure_events.read() {
        if let Ok(mut debt) = query.get_mut(event.source) {
            debt.exported_mass.0 += event.mass_amount.0;
        }
    }
}

pub fn calculate_launch_cost(
    base_cost: f32,
    debt: &GravityDebt,
    constants: &PhysicsConstants,
) -> f32 {
    base_cost * debt.current_gravity_modifier(constants)
}

pub fn calculate_launch_failure_chance(
    base_chance: f32,
    debt: &GravityDebt,
    constants: &PhysicsConstants,
) -> f32 {
    base_chance * debt.current_gravity_modifier(constants).powf(1.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_debt_accumulation() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_gravity_debt_system);
        app.add_event::<FreighterArrivalEvent>();
        app.add_event::<FreighterDepartureEvent>();

        let planet_entity = app
            .world_mut()
            .spawn((
                Planet { base_gravity: 1.0 },
                GravityDebt {
                    imported_mass: Mass(0.0),
                    exported_mass: Mass(0.0),
                },
            ))
            .id();

        app.world_mut().send_event(FreighterArrivalEvent {
            destination: planet_entity,
            mass_amount: Mass(5000.0),
        });

        app.world_mut().send_event(FreighterDepartureEvent {
            source: planet_entity,
            mass_amount: Mass(1000.0),
        });

        app.update();

        let debt = app.world().get::<GravityDebt>(planet_entity).unwrap();
        assert_eq!(debt.imported_mass.0, 5000.0);
        assert_eq!(debt.exported_mass.0, 1000.0);

        let constants = PhysicsConstants::default();
        assert!(
            debt.current_gravity_modifier(&constants) > 1.0,
            "Importing more mass than exporting should increase gravity modifier."
        );
    }

    #[test]
    fn test_launch_cost_scales_with_debt() {
        let base_cost = 100.0;
        let planet_debt = GravityDebt {
            imported_mass: Mass(100000.0),
            exported_mass: Mass(0.0),
        };
        let constants = PhysicsConstants::default();

        let modified_cost = calculate_launch_cost(base_cost, &planet_debt, &constants);

        assert!(
            modified_cost > base_cost,
            "Launch cost should be significantly higher due to accumulated mass"
        );
    }

    #[test]
    fn test_launch_failure_chance_increases() {
        let planet_debt = GravityDebt {
            imported_mass: Mass(500000.0),
            exported_mass: Mass(0.0),
        };
        let base_chance = 0.01;
        let constants = PhysicsConstants::default();

        let failure_chance = calculate_launch_failure_chance(base_chance, &planet_debt, &constants);
        assert!(
            failure_chance > base_chance,
            "Failure chance must increase on high gravity debt planets."
        );
    }
}

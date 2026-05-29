use bevy::prelude::*;

#[derive(Component)]
pub struct DayLength {
    pub hours: f32, // f32::INFINITY means tidally locked
}

#[derive(Component)]
pub struct PlanetaryEngine {
    pub torque: f32,
    pub active: bool,
}

#[derive(Component)]
pub struct AttachedToPlanet(pub Entity);

pub fn apply_planetary_torque_system(
    engine_query: Query<(&PlanetaryEngine, &AttachedToPlanet)>,
    mut planet_query: Query<&mut DayLength>,
    mut torque_events: EventWriter<PlanetaryTorqueEvent>,
) {
    for (engine, attachment) in engine_query.iter() {
        if engine.active {
            if let Ok(mut day_length) = planet_query.get_mut(attachment.0) {
                // If tidally locked, start it spinning with an arbitrary large day length
                if day_length.hours.is_infinite() {
                    day_length.hours = 10000.0;
                }

                // Convert hours to a pseudo angular velocity (w = 1 / hours)
                let mut w = 1.0 / day_length.hours;

                // Apply torque (dw)
                w += engine.torque * 0.1;
                w = w.max(0.0001); // Avoid div by zero

                // Convert back to day length
                day_length.hours = 1.0 / w;

                // Fire continuous event to potentially cause weather chaos
                torque_events.send(PlanetaryTorqueEvent {
                    planet: attachment.0,
                    torque_amount: engine.torque,
                });
            }
        }
    }
}

#[derive(Component)]
pub struct BaseGravity {
    pub g: f32,
}

#[derive(Component)]
pub struct EffectiveGravity {
    pub g: f32,
}

pub fn calculate_effective_gravity_system(
    mut planet_query: Query<(&BaseGravity, &DayLength, &mut EffectiveGravity)>,
) {
    for (base_g, day_length, mut eff_g) in planet_query.iter_mut() {
        // If tidally locked, no centrifugal reduction
        let reduction = if day_length.hours.is_infinite() {
            0.0
        } else {
            // Simplified centrifugal reduction: faster spin (lower hours) = more reduction
            1.0 / day_length.hours
        };
        eff_g.g = (base_g.g - reduction).max(0.1);
    }
}

#[derive(Component, PartialEq, Debug)]
pub enum WeatherState {
    Calm,
    Chaotic,
}

#[derive(Event)]
pub struct PlanetaryTorqueEvent {
    pub planet: Entity,
    pub torque_amount: f32,
}

pub fn trigger_coriolis_weather_system(
    mut events: EventReader<PlanetaryTorqueEvent>,
    mut planet_query: Query<&mut WeatherState>,
) {
    for event in events.read() {
        if event.torque_amount.abs() > 2.0 {
            if let Ok(mut weather) = planet_query.get_mut(event.planet) {
                *weather = WeatherState::Chaotic;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::generation::Planet;

    #[test]
    fn test_applying_torque_changes_day_length() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<PlanetaryTorqueEvent>()
            .add_systems(Update, apply_planetary_torque_system);

        let planet = app
            .world_mut()
            .spawn((Planet, DayLength { hours: 24.0 }))
            .id();

        app.world_mut().spawn((
            PlanetaryEngine {
                torque: 1.0,
                active: true,
            },
            AttachedToPlanet(planet),
        ));

        app.update();

        let day_length = app.world().get::<DayLength>(planet).unwrap().hours;
        assert!(
            day_length < 24.0,
            "Day length should decrease when positive torque is applied"
        );
    }

    #[test]
    fn test_rapid_spin_reduces_effective_gravity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, calculate_effective_gravity_system);

        let planet = app
            .world_mut()
            .spawn((
                Planet,
                BaseGravity { g: 1.0 },
                DayLength { hours: 10.0 },
                EffectiveGravity { g: 1.0 },
            ))
            .id();

        app.update();

        let eff_g = app.world().get::<EffectiveGravity>(planet).unwrap().g;
        assert!(
            eff_g < 1.0,
            "Effective gravity should be reduced by rapid spin"
        );
    }

    #[test]
    fn test_spin_up_causes_chaotic_weather() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<PlanetaryTorqueEvent>()
            .add_systems(Update, trigger_coriolis_weather_system);

        let planet = app
            .world_mut()
            .spawn((Planet, DayLength { hours: 24.0 }, WeatherState::Calm))
            .id();

        app.world_mut()
            .resource_mut::<Events<PlanetaryTorqueEvent>>()
            .send(PlanetaryTorqueEvent {
                planet,
                torque_amount: 5.0,
            });

        app.update();

        assert_eq!(
            *app.world().get::<WeatherState>(planet).unwrap(),
            WeatherState::Chaotic
        );
    }

    #[test]
    fn test_tidally_locked_spin_up() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<PlanetaryTorqueEvent>()
            .add_systems(Update, apply_planetary_torque_system);

        let planet = app
            .world_mut()
            .spawn((
                Planet,
                DayLength {
                    hours: f32::INFINITY,
                },
            ))
            .id();

        app.world_mut().spawn((
            PlanetaryEngine {
                torque: 1.0,
                active: true,
            },
            AttachedToPlanet(planet),
        ));

        app.update();

        let day_length = app.world().get::<DayLength>(planet).unwrap().hours;
        assert!(
            !day_length.is_infinite(),
            "Day length should no longer be infinite"
        );
        assert!(day_length > 0.0, "Day length should be positive");
    }
}

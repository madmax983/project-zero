use crate::layer1::economy::ColonyResources;
use crate::layer1::pop::Pop;
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalHabitat {
    pub population: u32,
    pub wealth: f32,
}

#[derive(Component)]
pub struct Unrest {
    pub level: f32,
}

#[derive(Component, Debug, PartialEq)]
pub enum SecessionState {
    Loyal,
    Seceding,
    Seceded,
}

pub const SECESSION_UNREST_THRESHOLD: f32 = 80.0;

pub fn evaluate_orbital_secession_system(
    mut commands: Commands,
    mut habitat_query: Query<(Entity, &OrbitalHabitat, &Unrest)>,
    planet_resources: Option<Res<ColonyResources>>,
    pop_query: Query<&Pop>,
) {
    // If the resource doesn't exist yet, we can't secede based on its values.
    if let Some(resources) = planet_resources {
        let planet_wealth = resources.credits;
        let planet_population = pop_query.iter().count() as u32;

        for (entity, habitat, unrest) in habitat_query.iter_mut() {
            if habitat.wealth > planet_wealth
                && habitat.population > planet_population
                && unrest.level > SECESSION_UNREST_THRESHOLD
            {
                commands.entity(entity).insert(SecessionState::Seceded);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secession_triggers_on_high_wealth_and_unrest() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_orbital_secession_system);

        let habitat_entity = app
            .world_mut()
            .spawn((
                OrbitalHabitat {
                    population: 5000,
                    wealth: 100000.0,
                },
                Unrest { level: 90.0 },
            ))
            .id();

        // Spawn 4000 Pops for the planet
        for _ in 0..4000 {
            app.world_mut().spawn(Pop);
        }

        // Set planet wealth to 50000.0 via ColonyResources
        app.world_mut().insert_resource(ColonyResources {
            credits: 50000.0,
            ..Default::default()
        });

        app.update();

        let state = app.world().get::<SecessionState>(habitat_entity).unwrap();
        assert!(matches!(state, SecessionState::Seceded));
    }

    #[test]
    fn test_no_secession_if_homeworld_wealthier() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_orbital_secession_system);

        let habitat_entity = app
            .world_mut()
            .spawn((
                OrbitalHabitat {
                    population: 5000,
                    wealth: 10000.0,
                },
                Unrest { level: 90.0 },
            ))
            .id();

        // Spawn 10000 Pops for the planet
        for _ in 0..10000 {
            app.world_mut().spawn(Pop);
        }

        // Set planet wealth to 500000.0 via ColonyResources
        app.world_mut().insert_resource(ColonyResources {
            credits: 500000.0,
            ..Default::default()
        });

        app.update();

        let state = app.world().get::<SecessionState>(habitat_entity);
        assert!(state.is_none() || !matches!(state.unwrap(), SecessionState::Seceded));
    }
}

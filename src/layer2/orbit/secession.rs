use crate::layer1::economy::inflation::EmpireResources;
use crate::layer1::entities::pop::PopulationCount;
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalHabitat {
    pub population: usize,
    pub wealth: f32,
}

#[derive(Component)]
pub struct Unrest {
    pub level: f32,
}

pub const SECESSION_UNREST_THRESHOLD: f32 = 80.0;

#[derive(Component, Debug, PartialEq)]
pub enum SecessionState {
    Loyal,
    Seceding,
    Seceded,
}

pub fn evaluate_orbital_secession_system(
    mut commands: Commands,
    mut habitat_query: Query<(Entity, &OrbitalHabitat, &Unrest)>,
    planet_population_query: Option<Res<PopulationCount>>,
    planet_wealth_query: Query<&EmpireResources>,
) {
    let planet_population = if let Some(pop) = planet_population_query {
        pop.total
    } else {
        0
    };

    let planet_wealth = if let Some(wealth) = planet_wealth_query.iter().next() {
        wealth.credits
    } else {
        0.0
    };

    for (entity, habitat, unrest) in habitat_query.iter_mut() {
        if habitat.wealth > planet_wealth
            && habitat.population > planet_population
            && unrest.level > SECESSION_UNREST_THRESHOLD
        {
            commands.entity(entity).insert(SecessionState::Seceded);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::inflation::EmpireResources;
    use crate::layer1::entities::pop::PopulationCount;

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

        app.insert_resource(PopulationCount { total: 4000 });
        app.world_mut().spawn(EmpireResources {
            credits: 50000.0,
            alloys: 0,
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

        app.insert_resource(PopulationCount { total: 10000 });
        app.world_mut().spawn(EmpireResources {
            credits: 500000.0,
            alloys: 0,
        });

        app.update();

        let state = app.world().get::<SecessionState>(habitat_entity);
        assert!(state.is_none() || !matches!(state.unwrap(), SecessionState::Seceded));
    }
}

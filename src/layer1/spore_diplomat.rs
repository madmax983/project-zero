use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Stats {
    pub base_social: i32,
    pub base_intellect: i32,
    pub social: i32,
    pub intellect: i32,
}

impl Stats {
    pub fn new(social: i32, intellect: i32) -> Self {
        Self {
            base_social: social,
            base_intellect: intellect,
            social,
            intellect,
        }
    }
}

#[derive(Component)]
pub struct SporeInfection {
    pub severity: f32,
}

pub fn spore_stat_boost_system(
    mut query: Query<(&mut Stats, &SporeInfection), Added<SporeInfection>>,
) {
    for (mut stats, infection) in query.iter_mut() {
        if infection.severity > 0.0 {
            let boost = (infection.severity * 5.0).round() as i32;
            stats.social += boost;
            stats.intellect += boost;
        }
    }
}

pub fn spore_stat_remove_system(
    mut removed: RemovedComponents<SporeInfection>,
    mut query: Query<&mut Stats>,
) {
    for entity in removed.read() {
        if let Ok(mut stats) = query.get_mut(entity) {
            stats.social = stats.base_social;
            stats.intellect = stats.base_intellect;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_spore_infection_boosts_stats() {
        let mut app = App::new();
        let pop = app.world_mut().spawn((Pop, Stats::new(5, 5))).id();

        app.add_systems(Update, (spore_stat_boost_system, spore_stat_remove_system));

        app.world_mut().entity_mut(pop).insert(SporeInfection { severity: 1.0 });
        app.update();

        let initial_social = app.world().get::<Stats>(pop).unwrap().social;
        let initial_intellect = app.world().get::<Stats>(pop).unwrap().intellect;
        assert!(initial_social > 5);
        assert!(initial_intellect > 5);

        // Remove infection and ensure stats reset
        app.world_mut().entity_mut(pop).remove::<SporeInfection>();
        app.update();
        let stats2 = app.world().get::<Stats>(pop).unwrap();
        assert_eq!(stats2.social, 5);
        assert_eq!(stats2.intellect, 5);
    }
}

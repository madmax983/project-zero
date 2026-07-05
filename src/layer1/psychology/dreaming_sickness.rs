use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::entities::pop::Pop;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use rand::Rng;

#[derive(Component)]
pub struct DreamingSickness {
    pub severity: f32, // 0.0 to 1.0
}

type HealthyPopQueryFilter = (With<Pop>, Without<DreamingSickness>);
#[derive(Resource)]
pub struct DreamingSicknessConfig {
    pub spread_chance: f64,
}

impl Default for DreamingSicknessConfig {
    fn default() -> Self {
        Self { spread_chance: 0.1 }
    }
}

pub fn spread_dreaming_sickness(
    mut commands: Commands,
    infected_query: Query<&GridPosition, With<DreamingSickness>>,
    healthy_query: Query<(Entity, &GridPosition), HealthyPopQueryFilter>,
    config: Option<Res<DreamingSicknessConfig>>,
) {
    let mut rng = rand::thread_rng();
    let chance = config.map_or(0.1, |c| c.spread_chance);

    let mut infected_positions = Vec::new();
    for pos in infected_query.iter() {
        infected_positions.push(*pos);
    }

    if infected_positions.is_empty() {
        return;
    }

    for (healthy_entity, healthy_pos) in healthy_query.iter() {
        for infected_pos in &infected_positions {
            if infected_pos.distance_manhattan(*healthy_pos) <= 2 {
                if rng.gen_bool(chance) {
                    commands.entity(healthy_entity).insert(DreamingSickness { severity: 0.1 });
                }
                break; // Only test infection chance once per healthy pop per tick
            }
        }
    }
}

pub fn dreaming_sickness_effects(
    mut query: Query<(&mut Needs, &DreamingSickness)>,
) {
    for (mut needs, sickness) in query.iter_mut() {
        needs.rest -= sickness.severity * 0.05;
    }
}

pub fn spontaneous_sleep(
    mut query: Query<(&mut PopAction, &DreamingSickness)>,
) {
    for (mut action, sickness) in query.iter_mut() {
        if sickness.severity > 0.9 {
            match action.current {
                ActionType::SatisfyRest => {}
                _ => { action.current = ActionType::SatisfyRest; }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_dreaming_sickness_infection_spreads() {
        let mut app = App::new();
        app.insert_resource(DreamingSicknessConfig { spread_chance: 1.0 });
        app.add_systems(Update, spread_dreaming_sickness);

        let _infected = app.world_mut().spawn((Pop, GridPosition { x: 5, y: 5 }, DreamingSickness { severity: 0.1 })).id();
        let healthy = app.world_mut().spawn((Pop, GridPosition { x: 5, y: 6 })).id();

        app.update();

        assert!(app.world().get::<DreamingSickness>(healthy).is_some());
    }

    #[test]
    fn test_dreaming_sickness_increases_sleep_need() {
        let mut app = App::new();
        app.add_systems(Update, dreaming_sickness_effects);

        let pop = app.world_mut().spawn((Pop, Needs { rest: 1.0, ..Default::default() }, DreamingSickness { severity: 0.8 })).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.rest < 1.0);
        assert!((needs.rest - (1.0 - (0.8 * 0.05))).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spontaneous_sleep_at_high_severity() {
        let mut app = App::new();
        app.add_systems(Update, spontaneous_sleep);

        let pop = app.world_mut().spawn((Pop, PopAction { current: ActionType::Idle, current_utility: 0.0, ticks_committed: 0 }, DreamingSickness { severity: 0.95 })).id();

        app.update();

        let action = app.world().get::<PopAction>(pop).unwrap();
        match action.current {
            ActionType::SatisfyRest => assert!(true),
            _ => panic!("Expected ActionType::SatisfyRest"),
        }
    }
}

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Rearguard;

#[derive(Component)]
pub struct CombatStatsOld {
    pub attack: f32,
    pub defense: f32,
}

#[derive(Component)]
pub struct Morale {
    pub level: f32,
}

#[derive(Component)]
pub struct Pathfinding {
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct EscapePod;

#[derive(Component)]
pub struct Enemy {
    pub action_speed: f32,
}

#[derive(Component)]
pub struct EngagedWith {
    pub target: Entity,
}

pub fn rearguard_buff_system(mut query: Query<(&mut CombatStatsOld, &mut Morale), Added<Rearguard>>) {
    for (mut stats, mut morale) in query.iter_mut() {
        stats.attack *= 2.0; // Massive combat buff
        stats.defense *= 2.0;
        morale.level = 100.0; // Unbreakable morale
    }
}

pub fn rearguard_pathfinding_restriction_system(
    mut query: Query<(&Rearguard, &mut Pathfinding)>,
    escape_pods: Query<Entity, With<EscapePod>>,
) {
    for (_, mut pathfinding) in query.iter_mut() {
        if let Some(target) = pathfinding.target {
            if escape_pods.get(target).is_ok() {
                pathfinding.target = None; // Block pathfinding to escape pods
            }
        }
    }
}

pub fn rearguard_delays_enemy_system(
    mut enemies: Query<(&mut Enemy, &EngagedWith), Added<EngagedWith>>,
    rearguards: Query<Entity, With<Rearguard>>,
) {
    for (mut enemy, engaged) in enemies.iter_mut() {
        if rearguards.get(engaged.target).is_ok() {
            enemy.action_speed *= 0.5; // Significantly reduce enemy action speed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_rearguard_receives_massive_combat_buffs() {
        let mut app = App::new();
        app.add_systems(Update, rearguard_buff_system);

        let entity = app
            .world_mut()
            .spawn((
                Rearguard,
                CombatStatsOld {
                    attack: 10.0,
                    defense: 10.0,
                },
                Morale { level: 50.0 },
            ))
            .id();

        app.update();

        let stats = app.world().get::<CombatStatsOld>(entity).unwrap();
        let morale = app.world().get::<Morale>(entity).unwrap();

        assert_eq!(stats.attack, 20.0);
        assert_eq!(stats.defense, 20.0);
        assert_eq!(morale.level, 100.0);
    }

    #[test]
    fn test_rearguard_pathfinding_to_escape_pods_disabled() {
        let mut app = App::new();
        app.add_systems(Update, rearguard_pathfinding_restriction_system);

        let pod_entity = app.world_mut().spawn(EscapePod).id();
        let pop_entity = app
            .world_mut()
            .spawn((
                Rearguard,
                Pathfinding {
                    target: Some(pod_entity),
                },
            ))
            .id();

        app.update();

        let pathfinding = app.world().get::<Pathfinding>(pop_entity).unwrap();
        assert_eq!(pathfinding.target, None);
    }

    #[test]
    fn test_rearguard_delays_enemy_advance() {
        let mut app = App::new();
        app.add_systems(Update, rearguard_delays_enemy_system);

        let rearguard_entity = app.world_mut().spawn(Rearguard).id();

        let enemy_entity = app
            .world_mut()
            .spawn((
                Enemy { action_speed: 10.0 },
                EngagedWith {
                    target: rearguard_entity,
                },
            ))
            .id();

        app.update();

        let enemy = app.world().get::<Enemy>(enemy_entity).unwrap();
        assert_eq!(enemy.action_speed, 5.0); // Halved speed
    }
}

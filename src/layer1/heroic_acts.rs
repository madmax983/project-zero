use crate::layer1::combat::CombatStats;
use crate::layer1::health::{Dead, Health};
use crate::prelude::{Chronicle, EventImportance};
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct HeroicActStatus {
    pub active: bool,
}

#[derive(Component, Default)]
pub struct PermanentInjury;

/// Custom event to trigger a heroic act.
#[derive(Event)]
pub struct TriggerHeroicAct {
    pub entity: Entity,
}

pub fn activate_heroic_act_system(
    mut events: EventReader<TriggerHeroicAct>,
    mut query: Query<(&mut CombatStats, &mut HeroicActStatus)>,
    time: Res<crate::shared::time::SimulationTime>,
    mut chronicle: Option<ResMut<Chronicle>>,
) {
    for event in events.read() {
        if let Ok((mut stats, mut status)) = query.get_mut(event.entity) {
            status.active = true;
            stats.melee_damage *= 2.0;

            if let Some(ref mut c) = chronicle {
                c.add_event(
                    time.tick,
                    "Heroic Sacrifice Initiated".to_string(),
                    EventImportance::Major,
                );
            }
        }
    }
}

pub fn resolve_heroic_acts_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&Health>), With<HeroicActStatus>>,
) {
    for (entity, health_opt) in query.iter_mut() {
        commands.entity(entity).remove::<HeroicActStatus>();

        let is_injured = if let Some(health) = health_opt {
            health.current > health.max * 0.5
        } else {
            false
        };

        if is_injured {
            commands.entity(entity).insert(PermanentInjury);
        } else {
            commands.entity(entity).insert(Dead);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heroic_act_activation_grants_buffs() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });
        world.insert_resource(Chronicle::default());
        world.init_resource::<Events<TriggerHeroicAct>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(activate_heroic_act_system);

        let entity = world
            .spawn((
                CombatStats {
                    melee_damage: 10.0,
                    damage_types: vec![],
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                HeroicActStatus::default(),
            ))
            .id();

        world
            .resource_mut::<Events<TriggerHeroicAct>>()
            .send(TriggerHeroicAct { entity });

        schedule.run(&mut world);

        let status = world.get::<HeroicActStatus>(entity).unwrap();
        assert!(status.active);

        let stats = world.get::<CombatStats>(entity).unwrap();
        assert!(
            stats.melee_damage > 10.0,
            "Melee damage should be increased"
        );
    }

    #[test]
    fn test_heroic_act_results_in_death_or_injury_after_combat() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_heroic_acts_system);

        let entity = world
            .spawn((
                CombatStats {
                    melee_damage: 20.0,
                    damage_types: vec![],
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                HeroicActStatus { active: true },
            ))
            .id();

        schedule.run(&mut world);

        let is_dead = world.get::<Dead>(entity).is_some();
        let is_injured = world.get::<PermanentInjury>(entity).is_some();

        assert!(
            is_dead || is_injured,
            "Unit must die or be permanently injured after a heroic act"
        );
    }
}

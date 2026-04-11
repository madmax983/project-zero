use crate::layer1::pop::Pop;
use crate::layer1::skills::{SkillType, Skills};
use bevy_ecs::prelude::*;
use bevy::prelude::Time;

// The Deserter's Haven (Spec 954)

#[derive(Component)]
pub struct Deserter {
    pub faction_id: Entity,
}

#[derive(Component)]
pub struct SmuggledWeapon;

#[derive(Resource)]
pub struct EscalatingTension {
    pub level: f32,
    pub threshold: f32,
}

impl Default for EscalatingTension {
    fn default() -> Self {
        Self {
            level: 0.0,
            threshold: 100.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct ProxyWarEvent;

#[derive(Event, Debug, Clone)]
pub struct HavenExposedEvent {
    pub exposed_by: Entity, // Faction/Empire entity
}

#[derive(Resource)]
pub struct ActiveWar {
    pub faction_a: Entity,
    pub faction_b: Entity,
}

pub fn deserter_arrival_system(mut commands: Commands, active_war: Option<Res<ActiveWar>>) {
    if let Some(war) = active_war {
        // Spawn deserter A
        let mut skills_a = Skills::default();
        skills_a.xp.insert(SkillType::Crafting, 90.0);
        skills_a.xp.insert(SkillType::Construction, 90.0);

        commands.spawn((
            Pop,
            skills_a,
            Deserter {
                faction_id: war.faction_a,
            },
            SmuggledWeapon,
        ));

        // Spawn deserter B
        let mut skills_b = Skills::default();
        skills_b.xp.insert(SkillType::Crafting, 90.0);
        skills_b.xp.insert(SkillType::Construction, 90.0);

        commands.spawn((
            Pop,
            skills_b,
            Deserter {
                faction_id: war.faction_b,
            },
            SmuggledWeapon,
        ));

        commands.remove_resource::<ActiveWar>();
    }
}

pub fn deserter_social_interaction_system(
    mut tension: ResMut<EscalatingTension>,
    deserters: Query<&Deserter>,
    time: Res<Time>,
) {
    let deserter_factions: Vec<Entity> = deserters.iter().map(|d| d.faction_id).collect();

    // Check if there are deserters from different factions
    let has_opposing = deserter_factions
        .iter()
        .any(|&f| deserter_factions.iter().any(|&other_f| other_f != f));

    if has_opposing {
        tension.level += time.delta_secs() * 1.0; // Assume 1 unit of tension per second
    }
}

pub fn proxy_war_escalation_system(
    mut commands: Commands,
    mut tension: ResMut<EscalatingTension>,
    deserters: Query<(Entity, &Deserter, Has<SmuggledWeapon>)>,
    mut proxy_events: EventWriter<ProxyWarEvent>,
    mut exposed_events: EventWriter<HavenExposedEvent>,
) {
    if tension.level >= tension.threshold {
        proxy_events.send(ProxyWarEvent);

        for (entity, deserter, has_weapon) in deserters.iter() {
            if has_weapon {
                exposed_events.send(HavenExposedEvent {
                    exposed_by: deserter.faction_id,
                });
            }

            // Remove Deserter to ensure proxy war only escalates once
            commands.entity(entity).remove::<Deserter>();
        }

        tension.level = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserter_arrival_during_war() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, deserter_arrival_system);

        let faction_a = app.world_mut().spawn_empty().id();
        let faction_b = app.world_mut().spawn_empty().id();

        app.insert_resource(ActiveWar {
            faction_a,
            faction_b,
        });

        app.update();

        let deserters = app
            .world_mut()
            .query::<&Deserter>()
            .iter(app.world())
            .count();
        assert_eq!(deserters, 2);
    }

    #[test]
    fn test_deserters_have_advanced_skills() {
        let mut app = bevy::app::App::new();
        app.add_systems(bevy::app::Update, deserter_arrival_system);

        let faction_a = app.world_mut().spawn_empty().id();
        let faction_b = app.world_mut().spawn_empty().id();

        app.insert_resource(ActiveWar {
            faction_a,
            faction_b,
        });

        app.update();

        let mut query = app.world_mut().query::<&Skills>();
        for skills in query.iter(app.world()) {
            assert!(*skills.xp.get(&SkillType::Crafting).unwrap() >= 90.0);
            assert!(*skills.xp.get(&SkillType::Construction).unwrap() >= 90.0);
        }
    }

    #[test]
    fn test_deserter_conflict_escalation() {
        let mut app = bevy::app::App::new();
        app.init_resource::<EscalatingTension>();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(time);

        app.add_systems(bevy::app::Update, deserter_social_interaction_system);

        let faction_a = app.world_mut().spawn_empty().id();
        let faction_b = app.world_mut().spawn_empty().id();

        app.world_mut().spawn(Deserter {
            faction_id: faction_a,
        });
        app.world_mut().spawn(Deserter {
            faction_id: faction_b,
        });

        app.update();

        let tension = app.world().get_resource::<EscalatingTension>().unwrap();
        assert!(tension.level > 0.0);
    }

    #[test]
    fn test_haven_exposure_risk() {
        let mut app = bevy::app::App::new();
        app.insert_resource(EscalatingTension {
            level: 100.0,
            threshold: 100.0,
        });
        app.add_event::<ProxyWarEvent>();
        app.add_event::<HavenExposedEvent>();
        app.add_systems(bevy::app::Update, proxy_war_escalation_system);

        let faction_a = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            Deserter {
                faction_id: faction_a,
            },
            SmuggledWeapon,
        ));

        app.update();

        let proxy_events = app
            .world()
            .get_resource::<bevy::ecs::event::Events<ProxyWarEvent>>()
            .unwrap();
        assert_eq!(proxy_events.len(), 1);

        let exposed_events = app
            .world()
            .get_resource::<bevy::ecs::event::Events<HavenExposedEvent>>()
            .unwrap();
        assert_eq!(exposed_events.len(), 1);
    }
}

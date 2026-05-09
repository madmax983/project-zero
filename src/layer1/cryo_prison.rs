use crate::layer1::entities::pop::Pop;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::social::unrest::Unrest;
use bevy::time::Time;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct CryoPod {
    pub thaw_progress: f32, // 0.0 to 1.0
    pub is_criminal: bool,
    pub specialty: CriminalSpecialty,
}

#[derive(Clone, Copy, Debug)]
pub enum CriminalSpecialty {
    Engineer,
    Scientist,
    Thug,
}

#[derive(Component)]
pub struct CriminalRecord;

#[derive(Event)]
pub struct SabotageEvent {
    pub saboteur: Entity,
    pub target: Option<Entity>,
}

pub fn thaw_cryo_pod_system(
    mut commands: Commands,
    mut unrest: ResMut<Unrest>,
    time: Option<Res<Time>>,
    mut query: Query<(Entity, &mut CryoPod)>,
) {
    let delta = time.map(|t| t.delta_secs()).unwrap_or(1.0); // Default 1.0 for tests without time
    for (entity, mut pod) in query.iter_mut() {
        pod.thaw_progress += delta * 0.1; // Thaws in 10 seconds

        if pod.thaw_progress >= 1.0 {
            commands.entity(entity).despawn();
            if pod.is_criminal {
                let mut skills = Skills::default();
                match pod.specialty {
                    CriminalSpecialty::Engineer => {
                        skills.xp.insert(SkillType::Engineering, 85.0);
                    }
                    CriminalSpecialty::Scientist => {
                        skills.xp.insert(SkillType::Crafting, 85.0);
                    } // Assuming crafting for scientist
                    CriminalSpecialty::Thug => {
                        skills.xp.insert(SkillType::Mining, 85.0);
                    } // Thug might use mining or construction
                }

                commands.spawn((Pop, CriminalRecord, skills));
                unrest.level = (unrest.level + 0.9).min(1.0);
            }
        }
    }
}

/// System to process SabotageEvent and deal damage to targeted structures.
pub fn infrastructure_damage_system(
    mut events: EventReader<SabotageEvent>,
    mut structures: Query<&mut crate::layer1::architecture::structure::Structure>,
) {
    for event in events.read() {
        if let Some(target) = event.target {
            if let Ok(mut structure) = structures.get_mut(target) {
                structure.current_hp -= 50.0;
            }
        }
    }
}

pub fn criminal_sabotage_system(
    mut events: EventWriter<SabotageEvent>,
    unrest: Res<Unrest>,
    query: Query<(Entity, &CriminalRecord)>,
    targets: Query<Entity, With<crate::layer1::architecture::structure::Structure>>,
) {
    if unrest.level > 0.9 {
        let target = targets.iter().next();
        for (entity, _) in query.iter() {
            events.send(SabotageEvent {
                saboteur: entity,
                target,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::skills::Skills;
    use crate::layer1::social::unrest::Unrest;
    use bevy::app::App;
    use bevy::prelude::Update;

    #[test]
    fn test_thawed_cryo_criminal_has_high_skills_and_unrest() {
        let mut app = App::new();
        app.insert_resource(Unrest::default());
        app.add_systems(Update, thaw_cryo_pod_system);

        let pod_entity = app
            .world_mut()
            .spawn((CryoPod {
                thaw_progress: 1.0,
                is_criminal: true,
                specialty: CriminalSpecialty::Engineer,
            },))
            .id();

        app.update();

        assert!(
            app.world().get::<CryoPod>(pod_entity).is_none(),
            "Pod should thaw and disappear"
        );

        let mut found = false;
        let mut query = app
            .world_mut()
            .query::<(Entity, &Skills, &CriminalRecord)>();
        for (_entity, skills, _criminal) in query.iter(app.world()) {
            assert!(
                skills.xp.values().any(|&v| v > 80.0),
                "Criminal should have high skills"
            );
            found = true;
        }
        assert!(found, "A new criminal pop should have been spawned");
        let unrest = app.world().get_resource::<Unrest>().unwrap();
        assert!(unrest.level > 0.8, "Criminal should have massive unrest");
    }

    #[test]
    fn test_criminal_sabotage_triggers_when_unrest_high() {
        let mut app = App::new();
        app.insert_resource(Unrest {
            level: 0.95,
            ..Default::default()
        });
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, criminal_sabotage_system);

        app.world_mut().spawn((Pop, CriminalRecord));

        app.update();

        let events = app.world().get_resource::<Events<SabotageEvent>>().unwrap();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_some(),
            "SabotageEvent should be fired by high-unrest criminal"
        );
    }

    #[test]
    fn test_infrastructure_damage_system() {
        use crate::layer1::architecture::structure::Structure;

        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, infrastructure_damage_system);

        let saboteur = app.world_mut().spawn((Pop, CriminalRecord)).id();
        let target = app
            .world_mut()
            .spawn(Structure {
                current_hp: 100.0,
                max_hp: 100.0,
            })
            .id();

        app.world_mut()
            .resource_mut::<Events<SabotageEvent>>()
            .send(SabotageEvent {
                saboteur,
                target: Some(target),
            });

        app.update();

        let structure = app.world().get::<Structure>(target).unwrap();
        assert!(
            structure.current_hp < 100.0,
            "Target structure should have taken damage from sabotage"
        );
    }
}

use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug, Clone)]
pub struct Paranoia {
    pub level: f32,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Dissent {
    pub level: f32,
}

#[derive(Event)]
pub struct AccusationEvent {
    pub accuser: Entity,
    pub target: Entity,
}

pub fn process_informant_reports(
    edict: Option<Res<crate::layer1::administration::edicts::ColonyPolicies>>,
    mut unrest: ResMut<crate::layer1::social::unrest::Unrest>,
    mut pops: Query<(Entity, &mut Paranoia, Option<&mut Dissent>)>,
) {
    if let Some(policies) = edict {
        if policies.is_active(crate::layer1::administration::edicts::Policy::CitizenInformant) {
            let mut reports_made = 0;

            // First pass: count dissenters and clear their dissent (they are "arrested" or "suppressed")
            for (_, _, dissent_opt) in pops.iter_mut() {
                if let Some(mut dissent) = dissent_opt {
                    if dissent.level > 50.0 {
                        dissent.level = 0.0;
                        reports_made += 1;
                    }
                }
            }

            // Second pass: apply effects. Unrest drops, but paranoia increases globally
            if reports_made > 0 {
                unrest.level = (unrest.level - (reports_made as f32 * 10.0)).max(0.0);

                for (_, mut paranoia, _) in pops.iter_mut() {
                    paranoia.level += reports_made as f32 * 5.0; // Society becomes paranoid
                }
            }
        }
    }
}

pub fn generate_false_accusations(
    mut events: EventWriter<AccusationEvent>,
    paranoid_query: Query<(Entity, &Paranoia), With<Pop>>,
    target_query: Query<Entity, With<Pop>>,
) {
    for (accuser, paranoia) in paranoid_query.iter() {
        if paranoia.level > 90.0 {
            // Simplified for GREEN phase: just pick the first available target that isn't self
            if let Some(target) = target_query.iter().find(|e| *e != accuser) {
                events.send(AccusationEvent { accuser, target });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
    use crate::layer1::social::unrest::Unrest;
    use bevy_app::App;

    #[test]
    fn test_edict_activates_informant_system() {
        // Arrange
        let mut app = App::new();
        app.add_systems(bevy_app::Update, process_informant_reports);

        let pop_a = app.world_mut().spawn(Paranoia { level: 0.0 }).id();
        let _pop_b = app
            .world_mut()
            .spawn((Dissent { level: 80.0 }, Paranoia { level: 0.0 }))
            .id();

        // Edict active
        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::CitizenInformant);
        app.world_mut().insert_resource(policies);
        app.world_mut().insert_resource(Unrest {
            level: 1.0,
            ..Default::default()
        });

        // Act
        app.update();

        // Assert
        let unrest = app.world().get_resource::<Unrest>().unwrap();
        let paranoia_a = app.world().get::<Paranoia>(pop_a).unwrap();

        // Unrest drops, but Paranoia rises for the informant
        assert!(
            unrest.level < 1.0,
            "Global unrest should drop when dissent is reported"
        );
        assert!(
            paranoia_a.level > 0.0,
            "Informant should gain paranoia after reporting"
        );
    }

    #[test]
    fn test_high_paranoia_causes_false_accusations() {
        // Arrange
        let mut app = App::new();
        app.add_event::<AccusationEvent>();
        app.add_systems(bevy_app::Update, generate_false_accusations);

        // Pop A is paranoid, Pop B is innocent (no Dissent)
        let _pop_a = app.world_mut().spawn((Pop, Paranoia { level: 95.0 })).id();
        let pop_b = app
            .world_mut()
            .spawn((Pop, Paranoia { level: 0.0 }, Dissent { level: 0.0 }))
            .id();

        // Act
        app.update();

        // Assert
        let events = app
            .world()
            .resource::<bevy_ecs::event::Events<AccusationEvent>>();
        let mut reader = events.get_cursor();
        let mut accusation_found = false;

        for ev in reader.read(events) {
            if ev.target == pop_b {
                accusation_found = true;
            }
        }

        assert!(
            accusation_found,
            "Highly paranoid pops should generate false accusations against innocent targets"
        );
    }
}

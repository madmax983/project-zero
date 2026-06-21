use crate::layer1::entities::pop::Pop;
use crate::layer1::social::factions::{FactionId, FactionMember, FactionState, Factions};
use bevy::prelude::*;

#[derive(Component)]
pub struct IntegratedCollective;

#[derive(Component)]
pub struct SynapseProximity {
    pub value: f32,
    pub decay_rate: f32,
}

#[derive(Event)]
pub struct SurgeryEvent {
    pub patient: Entity,
    pub procedure: String,
}

#[derive(Component)]
pub struct Sleep {
    pub value: f32,
}

#[derive(Component)]
pub struct Leisure {
    pub value: f32,
}

#[derive(Component)]
pub struct Hunger {
    pub value: f32,
    pub decay_rate: f32,
}

#[derive(Component)]
pub struct TraitList {
    pub traits: Vec<String>,
}

pub fn process_integration_surgery_system(
    mut commands: Commands,
    mut events: EventReader<SurgeryEvent>,
    mut query: Query<&mut TraitList, With<Pop>>,
) {
    for event in events.read() {
        if event.procedure == "XenoIntegration" {
            // Add collective flag and new need
            commands.entity(event.patient).insert((
                IntegratedCollective,
                SynapseProximity {
                    value: 100.0,
                    decay_rate: 0.05,
                },
            ));

            // Remove needs
            commands.entity(event.patient).remove::<Sleep>();
            commands.entity(event.patient).remove::<Leisure>();

            // Wipe personality
            if let Ok(mut traits) = query.get_mut(event.patient) {
                traits.traits.clear();
            }
        }
    }
}

pub fn decay_synapse_proximity_system(mut query: Query<&mut SynapseProximity>) {
    for mut proximity in query.iter_mut() {
        proximity.value = (proximity.value - proximity.decay_rate).max(0.0);
    }
}

pub fn trigger_hive_mind_faction_split_system(
    factions: Option<ResMut<Factions>>,
    mut pop_query: Query<(Option<&IntegratedCollective>, &mut FactionMember), With<Pop>>,
) {
    let mut total_pops = 0;
    let mut integrated_pops = 0;

    for (integrated, _) in pop_query.iter() {
        total_pops += 1;
        if integrated.is_some() {
            integrated_pops += 1;
        }
    }

    if total_pops == 0 {
        return;
    }

    let integrated_ratio = integrated_pops as f32 / total_pops as f32;

    // Split if more than 30% of the colony is integrated
    if integrated_ratio > 0.30 {
        if let Some(mut factions) = factions {
            factions.map.entry(FactionId::HiveMind).or_insert_with(|| {
                crate::layer1::social::factions::FactionData {
                    name: "The Collective".to_string(),
                    satisfaction: 1.0,
                    members_count: integrated_pops,
                    state: FactionState::Striking, // Hostile to non-integrated
                    active_demand: None,
                }
            });
        }

        // Reassign all integrated pops to the HiveMind faction
        for (integrated, mut member) in pop_query.iter_mut() {
            if integrated.is_some() && member.faction_id != Some(FactionId::HiveMind) {
                member.faction_id = Some(FactionId::HiveMind);
            }
        }
    }
}

pub fn evaluate_feed_score(actor: Entity, target: Entity, world: &World) -> f32 {
    let mut base_score = 100.0;

    let actor_is_collective = world.get::<IntegratedCollective>(actor).is_some();
    let target_is_collective = world.get::<IntegratedCollective>(target).is_some();

    if actor_is_collective && !target_is_collective {
        // Massive penalty to helping the "inefficient"
        base_score -= 90.0;
    }

    base_score
}

pub fn score_feeding_action_system() {
    // Stub for the actual Utility AI system integration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xeno_integration_removes_needs_and_traits() {
        let mut app = App::new();
        app.add_event::<SurgeryEvent>();
        app.add_systems(Update, process_integration_surgery_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                TraitList {
                    traits: vec!["Lazy".to_string(), "Brave".to_string()],
                },
                Sleep { value: 50.0 },
                Leisure { value: 50.0 },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<SurgeryEvent>>()
            .send(SurgeryEvent {
                patient: pop,
                procedure: "XenoIntegration".to_string(),
            });

        app.update();

        assert!(
            app.world().get::<IntegratedCollective>(pop).is_some(),
            "Pop should gain the IntegratedCollective component."
        );
        assert!(
            app.world().get::<Sleep>(pop).is_none(),
            "Integrated pops should not need sleep."
        );
        assert!(
            app.world().get::<Leisure>(pop).is_none(),
            "Integrated pops should not need leisure."
        );
        assert!(
            app.world().get::<SynapseProximity>(pop).is_some(),
            "Integrated pops should gain SynapseProximity."
        );
        let traits = app.world().get::<TraitList>(pop).unwrap();
        assert!(
            traits.traits.is_empty(),
            "Integrated pops should lose individual traits."
        );
    }

    #[test]
    fn test_decay_synapse_proximity() {
        let mut app = App::new();
        app.add_systems(Update, decay_synapse_proximity_system);

        let entity = app
            .world_mut()
            .spawn(SynapseProximity {
                value: 100.0,
                decay_rate: 10.0,
            })
            .id();

        app.update();

        let proximity = app.world().get::<SynapseProximity>(entity).unwrap();
        assert_eq!(proximity.value, 90.0);
    }

    #[test]
    fn test_trigger_hive_mind_faction_split() {
        let mut app = App::new();
        app.add_systems(Update, trigger_hive_mind_faction_split_system);
        app.world_mut().insert_resource(Factions::default());

        // Spawn 10 pops. 4 are integrated (40% > 30% threshold).
        for i in 0..10 {
            let mut entity = app.world_mut().spawn((
                Pop,
                FactionMember {
                    faction_id: Some(FactionId::Unaligned),
                },
            ));
            if i < 4 {
                entity.insert(IntegratedCollective);
            }
        }

        app.update();

        let factions = app.world().resource::<Factions>();
        assert!(factions.map.contains_key(&FactionId::HiveMind));

        let mut hive_mind_count = 0;
        for member in app.world_mut().query::<&FactionMember>().iter(app.world()) {
            if member.faction_id == Some(FactionId::HiveMind) {
                hive_mind_count += 1;
            }
        }

        assert_eq!(hive_mind_count, 4, "Integrated pops should be reassigned to HiveMind faction.");
    }

    #[test]
    fn test_collective_evaluates_unintegrated_pops_as_inefficient() {
        let mut app = App::new();
        app.add_systems(Update, score_feeding_action_system);

        let integrated = app.world_mut().spawn((Pop, IntegratedCollective)).id();
        let unintegrated = app
            .world_mut()
            .spawn((
                Pop,
                Hunger {
                    value: 0.0,
                    decay_rate: 1.0,
                },
            ))
            .id();
        let another_integrated = app
            .world_mut()
            .spawn((
                Pop,
                IntegratedCollective,
                Hunger {
                    value: 0.0,
                    decay_rate: 1.0,
                },
            ))
            .id();

        // Simulate Utility AI evaluating a "Feed Others" action
        // For unintegrated target
        let score_unintegrated = evaluate_feed_score(integrated, unintegrated, app.world());
        // For integrated target
        let score_integrated = evaluate_feed_score(integrated, another_integrated, app.world());

        assert!(
            score_unintegrated < score_integrated,
            "Integrated pops should heavily penalize helping unintegrated pops."
        );
    }
}

use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::core::integration::PirateAmnestyEvent;
use crate::layer2::fleet::{Fleet, FleetFaction, InOrbit};
use bevy::prelude::*;

/// Disbands Pirate Fleets in orbit if the Amnesty Visa policy is active.
#[allow(clippy::type_complexity)]
pub fn evaluate_pirate_amnesty_system(
    mut commands: Commands,
    policies: Res<ColonyPolicies>,
    fleets: Query<(Entity, &FleetFaction), (With<Fleet>, With<InOrbit>)>,
    mut amnesty_events: EventWriter<PirateAmnestyEvent>,
) {
    if !policies.is_active(Policy::AmnestyVisa) {
        return;
    }

    for (entity, faction) in fleets.iter() {
        if *faction == FleetFaction::Pirate {
            commands.entity(entity).despawn_recursive();
            amnesty_events.send(PirateAmnestyEvent { fleet: entity });
        }
    }
}

/// Tracks the pirate threat level globally.
#[derive(Resource, Default)]
pub struct PirateThreatLevel {
    pub level: f32,
}

/// Settings for the resource curse.
#[derive(Resource)]
pub struct ResourceCurseSettings {
    pub diplomacy_penalty: f32,
    pub threat_increase: f32,
    pub threat_cooldown_rate: f32,
}

impl Default for ResourceCurseSettings {
    fn default() -> Self {
        Self {
            diplomacy_penalty: 5.0,
            threat_increase: 10.0,
            threat_cooldown_rate: 1.0,
        }
    }
}

/// Process hyper valuable resources mined and apply the curse.
pub fn threat_cooldown_system(
    mut pirates: ResMut<PirateThreatLevel>,
    settings: Res<ResourceCurseSettings>,
) {
    if pirates.level > 0.0 {
        pirates.level = (pirates.level - settings.threat_cooldown_rate).max(0.0);
    }
}

pub fn process_hyper_resources(
    mut events: EventReader<crate::layer1::economy::resources::ResourceMinedEvent>,
    mut diplomacy: Query<&mut crate::layer3::diplomacy_reflection::DiplomaticRelations>,
    mut pirates: ResMut<PirateThreatLevel>,
    settings: Res<ResourceCurseSettings>,
) {
    for event in events.read() {
        if event.resource_type == crate::layer1::economy::resources::ResourceType::HyperValuable {
            for mut relation_group in diplomacy.iter_mut() {
                for relation in relation_group.relations.iter_mut() {
                    relation.standing -= settings.diplomacy_penalty * (event.amount as f32);
                }
            }
            pirates.level += settings.threat_increase * (event.amount as f32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::{ResourceMinedEvent, ResourceType};
    use crate::layer3::diplomacy_reflection::{DiplomaticRelations, DiplomaticStanding};

    #[test]
    fn test_evaluate_pirate_amnesty() {
        let mut app = App::new();
        app.add_event::<PirateAmnestyEvent>();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::AmnestyVisa);
        app.insert_resource(policies);

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Pirate,
                InOrbit {
                    parent: Entity::PLACEHOLDER,
                },
            ))
            .id();

        let other_fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                InOrbit {
                    parent: Entity::PLACEHOLDER,
                },
            ))
            .id();

        app.add_systems(Update, evaluate_pirate_amnesty_system);
        app.update();

        // Verify the pirate fleet is despawned
        assert!(
            app.world().get_entity(fleet_entity).is_err()
                || app.world().get::<Fleet>(fleet_entity).is_none()
        );

        // Verify the other fleet is unaffected
        assert!(app.world().get_entity(other_fleet).is_ok());
        assert!(app.world().get::<Fleet>(other_fleet).is_some());

        // Verify the event was sent
        let events = app.world().resource::<Events<PirateAmnestyEvent>>();
        let mut cursor = events.get_cursor();
        let mut event_count = 0;
        for ev in cursor.read(events) {
            assert_eq!(ev.fleet, fleet_entity);
            event_count += 1;
        }
        assert_eq!(event_count, 1);
    }

    #[test]
    fn test_threat_cooldown_system() {
        let mut app = App::new();
        app.add_systems(Update, threat_cooldown_system);
        app.init_resource::<PirateThreatLevel>();
        app.init_resource::<ResourceCurseSettings>();

        app.world_mut().resource_mut::<PirateThreatLevel>().level = 5.0;
        app.world_mut()
            .resource_mut::<ResourceCurseSettings>()
            .threat_cooldown_rate = 2.0;

        app.update();
        assert_eq!(app.world().resource::<PirateThreatLevel>().level, 3.0);

        app.update();
        app.update();
        // Should not go below 0.0
        assert_eq!(app.world().resource::<PirateThreatLevel>().level, 0.0);
    }

    #[test]
    fn test_hyper_valuable_resource_discovery_triggers_attention() {
        let mut app = App::new();
        app.add_event::<ResourceMinedEvent>();
        app.add_systems(Update, process_hyper_resources);

        app.init_resource::<PirateThreatLevel>();
        app.init_resource::<ResourceCurseSettings>();

        let initial_standing = 100.0;
        let _diplomacy_entity = app
            .world_mut()
            .spawn(DiplomaticRelations {
                relations: vec![DiplomaticStanding {
                    target_id: "player".to_string(),
                    standing: initial_standing,
                    sanctioned: false,
                }],
            })
            .id();
        app.world_mut().resource_mut::<PirateThreatLevel>().level = 0.0;

        app.world_mut()
            .resource_mut::<Events<ResourceMinedEvent>>()
            .send(ResourceMinedEvent {
                resource_type: ResourceType::HyperValuable,
                amount: 10,
            });

        app.update();

        let mut query = app.world_mut().query::<&DiplomaticRelations>();
        let final_standing = query.iter(app.world()).next().unwrap().relations[0].standing;
        let final_threat = app.world().resource::<PirateThreatLevel>().level;

        assert!(
            final_standing < initial_standing,
            "Diplomatic standing should decrease"
        );
        assert!(final_threat > 0.0, "Pirate threat level should increase");
    }
}

//! Intellectual Property Wars
//!
//! Simulates the corporate battles over technological patents at a galactic scale.
//! Factions engage in corporate espionage, legal sabotage, and patent theft, weaponizing
//! knowledge rather than fleets.

use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Component)]
pub struct Civilization;

#[derive(Component)]
pub struct Treasury {
    pub credits: i32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TechId(String);

impl TechId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

#[derive(Resource, Default)]
pub struct PatentRegistry {
    // tech -> (owner, fee)
    pub patents: HashMap<TechId, (Entity, i32)>,
}

impl PatentRegistry {
    pub fn register(&mut self, tech: TechId, owner: Entity, fee: i32) {
        self.patents.insert(tech, (owner, fee));
    }
    pub fn get_owner(&self, tech: &TechId) -> Option<Entity> {
        self.patents.get(tech).map(|(owner, _)| *owner)
    }
    pub fn get_fee(&self, tech: &TechId) -> Option<i32> {
        self.patents.get(tech).map(|(_, fee)| *fee)
    }
    pub fn invalidate(&mut self, tech: &TechId) {
        self.patents.remove(tech);
    }
}

#[derive(Event)]
pub struct TechDiscoveredEvent {
    pub civilization: Entity,
    pub tech: TechId,
}

pub fn register_patents_system(
    mut events: EventReader<TechDiscoveredEvent>,
    mut registry: ResMut<PatentRegistry>,
) {
    for event in events.read() {
        if registry.get_owner(&event.tech).is_none() {
            // Default fee of 100 for newly discovered tech
            registry.register(event.tech.clone(), event.civilization, 100);
        }
    }
}

#[derive(Component)]
pub struct TechUsage {
    pub civilization: Entity,
    pub tech: TechId,
    pub is_legal: bool,
}

pub fn process_licensing_fees_system(
    registry: Res<PatentRegistry>,
    usage_query: Query<&TechUsage>,
    mut treasury_query: Query<&mut Treasury>,
) {
    for usage in usage_query.iter() {
        if usage.is_legal {
            if let Some((owner, fee)) = registry.patents.get(&usage.tech) {
                if let Ok(mut licensee_treasury) = treasury_query.get_mut(usage.civilization) {
                    // Only deduct if they have enough, for simplicity of minimal implementation
                    if licensee_treasury.credits >= *fee {
                        licensee_treasury.credits -= *fee;
                        if let Ok(mut owner_treasury) = treasury_query.get_mut(*owner) {
                            owner_treasury.credits += *fee;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CassusBelli {
    pub aggressor: Entity,
    pub target: Entity,
    pub reason: CassusBelliReason,
}

#[derive(PartialEq, Debug)]
pub struct CassusBelliReason;

pub fn detect_ip_piracy_system(
    mut commands: Commands,
    registry: Res<PatentRegistry>,
    usage_query: Query<&TechUsage>,
    existing_cb_query: Query<&CassusBelli>,
) {
    for usage in usage_query.iter() {
        if !usage.is_legal {
            if let Some(owner) = registry.get_owner(&usage.tech) {
                if owner != usage.civilization {
                    // Avoid spawning duplicate CassusBelli for the same (aggressor, target, reason)
                    let already_exists = existing_cb_query.iter().any(|cb| {
                        cb.aggressor == owner
                            && cb.target == usage.civilization
                            && cb.reason == CassusBelliReason
                    });

                    if !already_exists {
                        commands.spawn(CassusBelli {
                            aggressor: owner,
                            target: usage.civilization,
                            reason: CassusBelliReason,
                        });
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct EspionageSuccessEvent {
    pub target: Entity,
    pub operation_type: EspionageOperation,
}

pub struct EspionageOperation {
    pub invalidated_tech: TechId,
}

pub fn process_espionage_system(
    mut events: EventReader<EspionageSuccessEvent>,
    mut registry: ResMut<PatentRegistry>,
) {
    for event in events.read() {
        registry.invalidate(&event.operation_type.invalidated_tech);
    }
}

pub struct IntellectualPropertyWarsPlugin;

impl Plugin for IntellectualPropertyWarsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatentRegistry>()
            .add_event::<TechDiscoveredEvent>()
            .add_event::<EspionageSuccessEvent>()
            .add_systems(
                Update,
                (
                    register_patents_system,
                    process_licensing_fees_system,
                    detect_ip_piracy_system,
                    process_espionage_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patent_registration_on_discovery() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, IntellectualPropertyWarsPlugin));

        // Arrange: Civilization discovers a new tech
        let civ_entity = app.world_mut().spawn(Civilization).id();
        let tech_id = TechId::new("warp_drive_v2");

        app.world_mut()
            .resource_mut::<Events<TechDiscoveredEvent>>()
            .send(TechDiscoveredEvent {
                civilization: civ_entity,
                tech: tech_id.clone(),
            });

        app.update();

        // Assert: Patent is created and owned by the civilization
        let patent_registry = app.world().resource::<PatentRegistry>();
        assert_eq!(patent_registry.get_owner(&tech_id), Some(civ_entity));
    }

    #[test]
    fn test_paying_licensing_fees() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, IntellectualPropertyWarsPlugin));

        let owner = app
            .world_mut()
            .spawn((Civilization, Treasury { credits: 100 }))
            .id();
        let licensee = app
            .world_mut()
            .spawn((Civilization, Treasury { credits: 500 }))
            .id();
        let tech_id = TechId::new("hyper_shields");

        app.world_mut()
            .resource_mut::<PatentRegistry>()
            .register(tech_id.clone(), owner, 50); // Fee is 50

        // Licensee uses the tech legally
        app.world_mut().spawn(TechUsage {
            civilization: licensee,
            tech: tech_id,
            is_legal: true,
        });

        app.update();

        // Assert: Credits transferred
        assert_eq!(app.world().get::<Treasury>(owner).unwrap().credits, 150);
        assert_eq!(app.world().get::<Treasury>(licensee).unwrap().credits, 450);
    }

    #[test]
    fn test_pirate_status_and_cassus_belli() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, IntellectualPropertyWarsPlugin));

        let owner = app.world_mut().spawn(Civilization).id();
        let pirate = app.world_mut().spawn(Civilization).id();
        let tech_id = TechId::new("quantum_torpedo");

        app.world_mut()
            .resource_mut::<PatentRegistry>()
            .register(tech_id.clone(), owner, 100);

        // Pirate uses the tech illegally
        app.world_mut().spawn(TechUsage {
            civilization: pirate,
            tech: tech_id,
            is_legal: false, // Not paying fees
        });

        app.update();

        // Assert: Cassus Belli generated for the owner against the pirate
        let cb_query = app
            .world_mut()
            .query::<&CassusBelli>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert_eq!(cb_query.len(), 1);
        assert_eq!(cb_query[0].aggressor, owner);
        assert_eq!(cb_query[0].target, pirate);
        assert_eq!(cb_query[0].reason, CassusBelliReason);
    }

    #[test]
    fn test_patent_invalidation_via_espionage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, IntellectualPropertyWarsPlugin));

        let owner = app.world_mut().spawn(Civilization).id();
        let tech_id = TechId::new("stealth_plating");

        app.world_mut()
            .resource_mut::<PatentRegistry>()
            .register(tech_id.clone(), owner, 200);

        // Arrange: Successful espionage event to invalidate
        app.world_mut()
            .resource_mut::<Events<EspionageSuccessEvent>>()
            .send(EspionageSuccessEvent {
                target: owner,
                operation_type: EspionageOperation {
                    invalidated_tech: tech_id.clone(),
                },
            });

        app.update();

        // Assert: Patent is no longer owned, moved to public domain
        let patent_registry = app.world().resource::<PatentRegistry>();
        assert_eq!(patent_registry.get_owner(&tech_id), None);
    }
}

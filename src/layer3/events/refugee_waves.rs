// src/layer3/events/refugee_waves.rs

use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::health::Health;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::Traits;
use crate::layer3::diplomacy_reflection::DiplomaticRelations;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Accept,
    Reject,
}

#[derive(Event)]
pub struct RefugeeWaveEvent {
    pub decision: Decision,
    pub population_count: usize,
    pub origin_faction: String,
    pub health_penalty: f32,
}

pub fn process_refugee_decision(
    mut events: EventReader<RefugeeWaveEvent>,
    mut commands: Commands,
    mut diplomacy_query: Query<&mut DiplomaticRelations>,
) {
    for event in events.read() {
        match event.decision {
            Decision::Accept => {
                // Spawn massive influx of low-health, desperate pops
                for _ in 0..event.population_count {
                    let mut traits = Traits::default();
                    traits.add(crate::layer1::psychology::traits::Trait::Refugee);
                    traits.add(crate::layer1::psychology::traits::Trait::Traumatized);

                    commands.spawn((
                        Pop,
                        Health { current: 100.0 - event.health_penalty, max: 100.0, has_rust_lung: false },
                        Needs { hunger: 10.0, rest: 10.0, ..default() },
                        traits,
                    ));
                }
            }
            Decision::Reject => {
                // Apply a severe diplomatic penalty with the origin faction (or their enemies)
                for mut dip in diplomacy_query.iter_mut() {
                    for relation in dip.relations.iter_mut() {
                        if relation.target_id == event.origin_faction {
                            relation.standing -= 50.0;
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer3::diplomacy_reflection::DiplomaticRelations;
    use crate::layer3::diplomacy_reflection::DiplomaticStanding;
    use crate::layer1::pop::Pop;
    use crate::layer1::health::Health;

    #[test]
    fn test_accepting_refugees_spawns_pops_with_low_health() {
        let mut app = App::new();
        app.add_event::<RefugeeWaveEvent>();
        app.add_systems(Update, process_refugee_decision);

        // Fire decision event to ACCEPT
        app.world_mut().send_event(RefugeeWaveEvent {
            decision: Decision::Accept,
            population_count: 50,
            origin_faction: "Rebel Alliance".to_string(),
            health_penalty: 50.0,
        });

        app.update();

        // Verify 50 new pops spawned with poor health
        let mut pop_query = app.world_mut().query::<(&Pop, &Health)>();
        let new_pops: Vec<_> = pop_query.iter(app.world()).collect();

        assert_eq!(new_pops.len(), 50);
        // Assuming max health is 100
        assert!(new_pops[0].1.current <= 50.0);
    }

    #[test]
    fn test_rejecting_refugees_causes_diplomatic_penalty() {
        let mut app = App::new();
        app.add_event::<RefugeeWaveEvent>();
        app.add_systems(Update, process_refugee_decision);

        let entity = app.world_mut().spawn(DiplomaticRelations {
            relations: vec![DiplomaticStanding {
                target_id: "Galactic Senate".to_string(),
                standing: 0.0,
                sanctioned: false,
            }],
        }).id();

        // Fire decision event to REJECT
        app.world_mut().send_event(RefugeeWaveEvent {
            decision: Decision::Reject,
            population_count: 50,
            origin_faction: "Galactic Senate".to_string(),
            health_penalty: 0.0,
        });

        app.update();

        let diplomacy = app.world().get::<DiplomaticRelations>(entity).unwrap();
        // The player's standing with "Galactic Senate" should decrease significantly
        assert!(diplomacy.relations[0].standing < 0.0);
    }
}

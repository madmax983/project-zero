use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::architecture::Building;
use crate::layer1::architecture::BuildingType;
use crate::layer1::health::Health;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct MindSporeInfection {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct SymbiontFaction {
    pub members: usize,
    pub critical_mass: usize,
}

#[derive(Event, Debug, Clone)]
pub struct MindSporeSabotageEvent {
    pub target: SabotageTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SabotageTarget {
    AirFiltration,
    Airlocks,
}

// Applies morale and work speed buff to infected Pops
pub fn process_mind_spore_infection_system(
    mut query: Query<(&mut Morale, &mut Traits, &MindSporeInfection)>,
) {
    for (mut morale, mut traits, infection) in query.iter_mut() {
        if infection.active {
            let has_modifier = morale
                .modifiers
                .iter()
                .any(|m| m.label == "Mind Spore Euphoria");
            if !has_modifier {
                morale.add_modifier(MoodModifier {
                    label: "Mind Spore Euphoria".to_string(),
                    value: 1.0,
                    duration: 100, // Renew occasionally rather than every tick
                });
            }

            if !traits.has(Trait::HardWorker) {
                traits.add(Trait::HardWorker);
            }
        }
    }
}

// Triggers sabotage events based on SymbiontFaction size
pub fn trigger_symbiont_sabotage_system(
    faction: Option<Res<SymbiontFaction>>,
    mut events: EventWriter<MindSporeSabotageEvent>,
) {
    let Some(faction) = faction else {
        return;
    };
    if faction.members == 0 {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.1) {
        // 10% chance per tick to trigger sabotage
        if faction.members >= faction.critical_mass {
            events.send(MindSporeSabotageEvent {
                target: SabotageTarget::Airlocks,
            });
        } else {
            // Less than critical mass might just sabotage air filters
            events.send(MindSporeSabotageEvent {
                target: SabotageTarget::AirFiltration,
            });
        }
    }
}

// Applies sabotage to buildings
pub fn process_sabotage_events_system(
    mut events: EventReader<MindSporeSabotageEvent>,
    mut airlocks: Query<(Entity, &mut AccessControl, &Building)>,
    mut life_supports: Query<(Entity, &mut Health, &Building)>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        match event.target {
            SabotageTarget::Airlocks => {
                let valid_targets: Vec<Entity> = airlocks
                    .iter()
                    .filter(|(_, _, b)| b.building_type == BuildingType::Airlock)
                    .map(|(e, _, _)| e)
                    .collect();

                if !valid_targets.is_empty() {
                    let target_idx = rng.gen_range(0..valid_targets.len());
                    let target_entity = valid_targets[target_idx];

                    if let Ok((_, mut access, _)) = airlocks.get_mut(target_entity) {
                        access.mode = AccessMode::Public; // Open to all
                    }
                }
            }
            SabotageTarget::AirFiltration => {
                let valid_targets: Vec<Entity> = life_supports
                    .iter()
                    .filter(|(_, _, b)| b.building_type == BuildingType::LifeSupport)
                    .map(|(e, _, _)| e)
                    .collect();

                if !valid_targets.is_empty() {
                    let target_idx = rng.gen_range(0..valid_targets.len());
                    let target_entity = valid_targets[target_idx];

                    if let Ok((_, mut health, _)) = life_supports.get_mut(target_entity) {
                        health.current -= 10.0; // Damage the life support
                    }
                }
            }
        }
    }
}

pub fn is_pop_infected(world: &World, entity: Entity) -> bool {
    world
        .get::<MindSporeInfection>(entity)
        .is_some_and(|i| i.active)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_mind_spore_infection_boosts_morale_and_work_speed() {
        let mut app = bevy_app::App::new();
        app.add_systems(Update, process_mind_spore_infection_system);

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                Traits::default(),
                MindSporeInfection { active: true },
            ))
            .id();

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        let traits = app.world().get::<Traits>(pop).unwrap();

        assert_eq!(
            morale.modifiers.len(),
            1,
            "Infected pops must have a mood modifier"
        );
        assert!(
            morale.modifiers[0].value > 0.5,
            "Infected pops must have boosted morale"
        );
        assert!(
            traits.has(Trait::HardWorker),
            "Infected pops must work faster"
        );
    }

    #[test]
    fn test_symbiont_faction_growth_triggers_sabotage() {
        let mut app = bevy_app::App::new();
        app.add_event::<MindSporeSabotageEvent>();
        app.add_systems(Update, trigger_symbiont_sabotage_system);

        app.world_mut().insert_resource(SymbiontFaction {
            members: 50,
            critical_mass: 40,
        });

        // Trigger lots of updates so the chance hits. Using a high number of iterations to avoid flaky test
        let mut sabotage_triggered = false;
        for _ in 0..1000 {
            app.update();
            let sabotage_events = app.world().resource::<Events<MindSporeSabotageEvent>>();
            let mut reader = sabotage_events.get_cursor();
            if reader.read(sabotage_events).len() > 0 {
                sabotage_triggered = true;
                break;
            }
        }

        assert!(
            sabotage_triggered,
            "Critical mass symbiont faction must trigger sabotage"
        );
    }

    #[test]
    fn test_process_sabotage_events_system() {
        let mut app = bevy_app::App::new();
        app.add_event::<MindSporeSabotageEvent>();
        app.add_systems(bevy_app::Update, process_sabotage_events_system);

        let airlock = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Airlock,
                },
                AccessControl {
                    mode: AccessMode::Restricted,
                    ..Default::default()
                },
            ))
            .id();

        let filter = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::LifeSupport,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<MindSporeSabotageEvent>>()
            .send(MindSporeSabotageEvent {
                target: SabotageTarget::Airlocks,
            });
        app.world_mut()
            .resource_mut::<Events<MindSporeSabotageEvent>>()
            .send(MindSporeSabotageEvent {
                target: SabotageTarget::AirFiltration,
            });

        app.update();

        let access = app.world().get::<AccessControl>(airlock).unwrap();
        assert_eq!(access.mode, AccessMode::Public, "Airlock should be opened");

        let health = app.world().get::<Health>(filter).unwrap();
        assert_eq!(health.current, 90.0, "LifeSupport should be damaged");
    }
}

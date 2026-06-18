use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Traits, Trait};

#[derive(Component)]
pub struct MindSporeInfection {
    pub active: bool,
}

#[derive(Resource, Default)]
pub struct SymbiontFaction {
    pub members: usize,
    pub critical_mass: usize,
}

#[derive(Event)]
pub struct SabotageEvent {
    pub target: SabotageTarget,
}

pub enum SabotageTarget {
    AirFiltration,
    Airlocks,
}

#[derive(Resource)]
pub struct InfectionConfig {
    pub base_transmission_rate: f32,
    pub flora_density_modifier: f32,
}

impl Default for InfectionConfig {
    fn default() -> Self {
        Self {
            base_transmission_rate: 0.05,
            flora_density_modifier: 0.02,
        }
    }
}

pub fn transmit_mind_spore_infection_system(
    config: Res<InfectionConfig>,
    mut commands: Commands,
    uninfected_query: Query<(Entity, &crate::layer1::entities::pop::Pop), Without<MindSporeInfection>>,
    mut faction: ResMut<SymbiontFaction>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let active_carriers = faction.members;
    let probability = config.base_transmission_rate + (active_carriers as f32 * 0.01) + config.flora_density_modifier;

    for (entity, _) in uninfected_query.iter() {
        if rng.gen::<f32>() < probability {
            commands.entity(entity).insert(MindSporeInfection { active: true });
            faction.members += 1;
        }
    }
}

pub fn process_mind_spore_infection_system(
    mut query: Query<(&mut Needs, &mut Traits, &MindSporeInfection)>,
) {
    for (mut needs, mut traits, infection) in query.iter_mut() {
        if infection.active {
            needs.hunger = 1.0;
            needs.rest = 1.0;
            needs.leisure = 1.0;
            needs.hygiene = 1.0;
            if !traits.has(Trait::MindSporeInfected) {
                traits.add(Trait::MindSporeInfected);
            }
        }
    }
}

pub fn trigger_symbiont_sabotage_system(
    faction: Res<SymbiontFaction>,
    mut events: EventWriter<SabotageEvent>,
    mut timer: Local<u32>,
) {
    *timer += 1;
    if *timer < 10 {
        return;
    }
    *timer = 0;

    if faction.members >= faction.critical_mass {
        events.send(SabotageEvent { target: SabotageTarget::Airlocks });
    } else if faction.members > 0 {
        events.send(SabotageEvent { target: SabotageTarget::AirFiltration });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::traits::{Traits, Trait, get_trait_work_speed_modifier};

    #[test]
    fn test_mind_spore_infection_boosts_morale_and_work_speed() {
        let mut app = App::new();
        app.add_systems(Update, process_mind_spore_infection_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0, hygiene: 0.0 }, // Morale will be 0.0
            Traits::default(),
            MindSporeInfection { active: true },
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let traits = app.world().get::<Traits>(pop).unwrap();

        assert!((needs.morale() - 1.0).abs() < f32::EPSILON, "Infected pops must have boosted morale (Needs set to 1.0)");
        assert!(traits.has(Trait::MindSporeInfected), "Infected pops must have MindSporeInfected trait");

        let work_speed = get_trait_work_speed_modifier(traits);
        assert!((work_speed - 1.5).abs() < f32::EPSILON, "Infected pops must work faster (1.5x)");
    }

    #[test]
    fn test_symbiont_faction_growth_triggers_sabotage() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, trigger_symbiont_sabotage_system);

        app.world_mut().insert_resource(SymbiontFaction { members: 50, critical_mass: 40 });

        for _ in 0..10 {
            app.update();
        }

        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        let mut reader = sabotage_events.get_cursor();
        let events: Vec<_> = reader.read(sabotage_events).collect();
        assert!(events.len() > 0, "Critical mass symbiont faction must trigger sabotage");

        // At critical mass, target should be Airlocks
        let target_is_airlocks = events.iter().any(|e| matches!(e.target, SabotageTarget::Airlocks));
        assert!(target_is_airlocks, "Sabotage target should be Airlocks");
    }

    #[test]
    fn test_transmit_mind_spore_infection_system() {
        let mut app = App::new();
        app.insert_resource(InfectionConfig {
            base_transmission_rate: 1.0, // 100% transmission for test
            flora_density_modifier: 0.0,
        });
        app.insert_resource(SymbiontFaction { members: 0, critical_mass: 40 });
        app.add_systems(Update, transmit_mind_spore_infection_system);

        let pop = app.world_mut().spawn(Pop).id();

        app.update();

        assert!(app.world().get::<MindSporeInfection>(pop).is_some(), "Pop should be infected");
        assert_eq!(app.world().resource::<SymbiontFaction>().members, 1, "Faction members should increase");
    }

    #[test]
    fn test_trigger_symbiont_sabotage_system_below_critical() {
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, trigger_symbiont_sabotage_system);

        app.world_mut().insert_resource(SymbiontFaction { members: 10, critical_mass: 40 });

        for _ in 0..10 {
            app.update();
        }

        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        let mut reader = sabotage_events.get_cursor();
        let events: Vec<_> = reader.read(sabotage_events).collect();
        assert!(events.len() > 0, "Non-zero symbiont faction must trigger sabotage");

        let target_is_air_filtration = events.iter().any(|e| matches!(e.target, SabotageTarget::AirFiltration));
        assert!(target_is_air_filtration, "Sabotage target should be AirFiltration");
    }
}

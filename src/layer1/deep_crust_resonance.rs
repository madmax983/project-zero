use bevy_ecs::prelude::*;
use crate::layer1::social::Relationships;
use crate::layer1::morale::{MoodModifier, Morale};

#[derive(Component)]
pub struct ResonantOre;

#[derive(Component)]
pub struct ResonantInfection {
    pub severity: f32,
}

#[derive(Event)]
pub struct ExcavationEvent {
    pub colony: Entity,
    pub miner: Entity,
    pub discovery_type: String,
    pub target: Entity,
}

#[derive(Component)]
pub struct MineableOre;

pub fn resonant_ore_exposure_system(
    mut commands: Commands,
    mut events: EventReader<ExcavationEvent>,
    mut q_morale: Query<&mut Morale>,
) {
    for event in events.read() {
        if event.discovery_type == "ResonantOre" {
            if let Ok(mut morale) = q_morale.get_mut(event.miner) {
                if !morale.modifiers.iter().any(|m| m.label == "Deep Resonance") {
                    morale.modifiers.push(MoodModifier {
                        label: "Deep Resonance".to_string(),
                        value: -0.10, // -10.0 scaled to -0.10 since value is typically 0.0-1.0 and summed
                        duration: 100, // Giving it a duration
                    });
                }
                commands.entity(event.miner).insert(ResonantInfection { severity: 1.0 });
            }
        }
    }
}

pub fn resonance_social_spread_system(
    q_relationships: Query<&Relationships>,
    q_infected: Query<&ResonantInfection>,
    mut q_morale: Query<(Entity, &mut Morale)>,
) {
    let mut to_infect = Vec::new();

    // Find all pops that have relationships
    for (entity, _) in q_morale.iter() {
        if let Ok(rel) = q_relationships.get(entity) {
             // For every pop this pop has a relationship with
             for (&target, &affinity) in &rel.affinities {
                  // If the target is infected and they are friends (or just related)
                  if q_infected.get(target).is_ok() && affinity > 0.0 {
                      to_infect.push(entity);
                      break; // Just need one infected friend to get paranoid
                  }
             }
        }
    }

    for entity in to_infect {
        if let Ok((_, mut morale)) = q_morale.get_mut(entity) {
            if !morale.modifiers.iter().any(|m| m.label == "Paranoia") {
                morale.modifiers.push(MoodModifier {
                    label: "Paranoia".to_string(),
                    value: -0.05,
                    duration: 100,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::social::Relationships;
    use crate::layer1::pop::Pop;

    use crate::layer1::morale::Morale;

    #[test]
    fn test_mining_resonant_ore_inflicts_resonance_debuff() {
        let mut app = App::new();
        app.add_event::<ExcavationEvent>();
        app.add_systems(Update, resonant_ore_exposure_system);

        let miner = app.world_mut().spawn((Pop, Morale::default())).id();
        let resonant_ore = app.world_mut().spawn((MineableOre, ResonantOre)).id();

        app.world_mut().resource_mut::<Events<ExcavationEvent>>().send(ExcavationEvent {
            colony: Entity::PLACEHOLDER,
            miner,
            discovery_type: "ResonantOre".to_string(),
            target: resonant_ore,
        });

        app.update();

        let morale = app.world().get::<Morale>(miner).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Deep Resonance"), "Miner exposed to resonant ore should gain negative resonance mood modifier.");
        assert!(app.world().get::<ResonantInfection>(miner).is_some(), "Miner should be marked as carrying the resonance.");
    }

    #[test]
    fn test_resonance_spreads_paranoia_through_relationships() {
        let mut app = App::new();
        app.add_systems(Update, resonance_social_spread_system);

        let pop_a = app.world_mut().spawn((Pop, ResonantInfection { severity: 1.0 })).id();
        let mut rels = Relationships::default();
        rels.affinities.insert(pop_a, 0.5); // pop_b likes pop_a
        let pop_b = app.world_mut().spawn((Pop, Morale::default(), rels)).id();


        app.update();

        let morale_b = app.world().get::<Morale>(pop_b).unwrap();
        assert!(morale_b.modifiers.iter().any(|m| m.label == "Paranoia"), "Paranoia should spread to related pops.");
    }
}

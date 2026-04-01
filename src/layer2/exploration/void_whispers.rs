use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer2::fleet::Fleet;
use rand::seq::IteratorRandom;

#[derive(Component)]
pub struct DeepSpaceExposure {
    pub ticks: u32,
}

#[derive(Component)]
pub struct VoidWhispers {
    pub intensity: f32,
}

#[derive(Component)]
pub struct MemeticInfection;

#[derive(Event)]
pub struct FleetReturnedEvent {
    pub fleet: Entity,
    pub colony: Entity,
}

pub fn accumulate_void_whispers_in_deep_space(
    mut commands: Commands,
    fleets: Query<(Entity, &DeepSpaceExposure), (With<Fleet>, Without<VoidWhispers>)>,
) {
    for (entity, exposure) in fleets.iter() {
        if exposure.ticks >= 100 {
            commands.entity(entity).insert(VoidWhispers { intensity: 10.0 });
        }
    }
}

pub fn spread_whispers_to_colony(
    mut commands: Commands,
    mut events: EventReader<FleetReturnedEvent>,
    fleets_with_whispers: Query<&VoidWhispers>,
    pops: Query<(Entity, Option<&Parent>), With<Pop>>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if fleets_with_whispers.get(event.fleet).is_ok() {
            // Infect a single "Patient Zero" pop instead of instantly infecting all.
            // Safe assumption: If pops are children of the colony entity, filter by parent.
            // If the hierarchy is flat, fall back to infecting any random pop.
            let colony_pops: Vec<Entity> = pops
                .iter()
                .filter(|(_, parent)| {
                    parent.map_or(false, |p| p.get() == event.colony)
                })
                .map(|(e, _)| e)
                .collect();

            let target_pop = if !colony_pops.is_empty() {
                colony_pops.into_iter().choose(&mut rng)
            } else {
                pops.iter().map(|(e, _)| e).choose(&mut rng)
            };

            if let Some(pop_entity) = target_pop {
                commands.entity(pop_entity).insert(MemeticInfection);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explorers_accumulate_void_whispers() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_void_whispers_in_deep_space);

        let fleet_entity = app.world_mut().spawn((
            Fleet,
            DeepSpaceExposure { ticks: 100 },
        )).id();

        app.update();

        let whispers = app.world().get::<VoidWhispers>(fleet_entity);
        assert!(whispers.is_some(), "Fleet exposed to deep space should accumulate Void Whispers");
        assert_eq!(whispers.unwrap().intensity, 10.0);
    }

    #[test]
    fn test_returning_fleet_infects_colony() {
        let mut app = App::new();
        app.add_event::<FleetReturnedEvent>();
        app.add_systems(Update, spread_whispers_to_colony);

        let fleet_entity = app.world_mut().spawn((
            VoidWhispers { intensity: 50.0 },
        )).id();

        let colony_pop_entity = app.world_mut().spawn((
            Pop,
        )).id();

        app.world_mut().send_event(FleetReturnedEvent {
            fleet: fleet_entity,
            colony: Entity::PLACEHOLDER, // Generic target for test
        });

        app.update();

        // Colony pop should now have an infection or meme component
        let meme = app.world().get::<MemeticInfection>(colony_pop_entity);
        assert!(meme.is_some(), "Colony pop should receive MemeticInfection from returning fleet");
    }
}
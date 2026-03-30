use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::manhattan_distance;
use crate::layer1::unrest::Unrest;

/// Represents an Ethic in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ethic {
    Pacifism,
    Militarism,
    Collectivism,
    Individualism,
    Materialism,
    Spiritualism,
}

/// The state ideology.
#[derive(Resource, Default, Debug, Clone)]
pub struct StateIdeology {
    pub dominant_ethic: Option<Ethic>,
}

/// A Pop's individual ethics.
#[derive(Component, Default, Debug, Clone)]
pub struct PopEthics {
    pub alignments: HashMap<Ethic, f32>, // -1.0 to 1.0 (opposed to aligned)
}

/// A building that broadcasts an Ethic.
#[derive(Component, Debug, Clone)]
pub struct IndoctrinationAura {
    pub target_ethic: Ethic,
    pub strength: f32, // Rate of shift per tick
    pub radius: i32,
}

/// Event triggered when indoctrination fails, causing dissent.
#[derive(Event, Debug, Clone)]
pub struct IndoctrinationFailureEvent {
    pub pop: Entity,
    pub building: Entity,
    pub ethic: Ethic,
}

/// System that processes the indoctrination aura effect on nearby pops.
pub fn process_indoctrination_system(
    buildings: Query<(Entity, &GridPosition, &IndoctrinationAura)>,
    mut pops: Query<(Entity, &GridPosition, &mut PopEthics)>,
    mut dissent_events: EventWriter<IndoctrinationFailureEvent>,
) {
    for (building_entity, b_pos, aura) in buildings.iter() {
        for (pop_entity, p_pos, mut ethics) in pops.iter_mut() {
            let dist = manhattan_distance(b_pos, p_pos);
            if dist <= aura.radius {
                // Get current alignment
                let current_alignment = *ethics.alignments.get(&aura.target_ethic).unwrap_or(&0.0);

                // If pop is highly opposed, there's a chance they reject it and cause dissent
                if current_alignment < -0.8 {
                    dissent_events.send(IndoctrinationFailureEvent {
                        pop: pop_entity,
                        building: building_entity,
                        ethic: aura.target_ethic,
                    });

                    // Slightly increase alignment to avoid infinite unrest looping, pushing them out of the highly opposed range over time or with just one step depending on strength.
                    let new_alignment = (current_alignment + (aura.strength * 2.0)).clamp(-1.0, 1.0);
                    ethics.alignments.insert(aura.target_ethic, new_alignment);
                } else {
                    // Shift ethics towards the target
                    let new_alignment = (current_alignment + aura.strength).clamp(-1.0, 1.0);
                    ethics.alignments.insert(aura.target_ethic, new_alignment);
                }
            }
        }
    }
}

/// System that handles indoctrination failure events by increasing Unrest.
pub fn handle_dissent_system(
    mut events: EventReader<IndoctrinationFailureEvent>,
    unrest: Option<ResMut<Unrest>>,
) {
    let mut total_dissent = 0.0;
    for _event in events.read() {
        total_dissent += 0.05; // 5% unrest per failure
    }

    if total_dissent > 0.0 {
        if let Some(mut unrest_res) = unrest {
            unrest_res.level = (unrest_res.level + total_dissent).clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> World {
        let mut world = World::new();
        world.init_resource::<Events<IndoctrinationFailureEvent>>();
        world
    }

    #[test]
    fn test_indoctrination_shifts_pop_ethics_over_time() {
        let mut world = setup();

        // Arrange: Pop with divergent ethics near an active Media Station broadcasting State Ethics
        let pop = world.spawn((
            GridPosition { x: 0, y: 0 },
            PopEthics {
                alignments: [(Ethic::Militarism, 0.0)].into_iter().collect(),
            },
        )).id();

        world.spawn((
            GridPosition { x: 1, y: 0 },
            IndoctrinationAura {
                target_ethic: Ethic::Militarism,
                strength: 0.1,
                radius: 5,
            },
        ));

        // Act: Step the simulation forward over time
        let mut schedule = Schedule::default();
        schedule.add_systems(process_indoctrination_system);
        schedule.run(&mut world);

        // Assert: Pop's ethics drift closer to the State Ethics
        let ethics = world.get::<PopEthics>(pop).unwrap();
        assert_eq!(*ethics.alignments.get(&Ethic::Militarism).unwrap(), 0.1);
    }

    #[test]
    fn test_indoctrination_failure_causes_dissent() {
        let mut world = setup();
        world.insert_resource(Unrest { level: 0.0, modifiers: vec![] });

        // Arrange: Pop with highly stubborn, opposed ethics exposed to Indoctrination
        let pop = world.spawn((
            GridPosition { x: 0, y: 0 },
            PopEthics {
                alignments: [(Ethic::Militarism, -0.9)].into_iter().collect(),
            },
        )).id();

        let _building = world.spawn((
            GridPosition { x: 1, y: 0 },
            IndoctrinationAura {
                target_ethic: Ethic::Militarism,
                strength: 0.1,
                radius: 5,
            },
        )).id();

        // Act: Step simulation
        let mut schedule = Schedule::default();
        schedule.add_systems((process_indoctrination_system, handle_dissent_system.after(process_indoctrination_system)));
        schedule.run(&mut world);

        // Assert: Pop gains Dissent/Unrest and shifting ethics
        let ethics = world.get::<PopEthics>(pop).unwrap();
        assert!(*ethics.alignments.get(&Ethic::Militarism).unwrap() > -0.9); // shifted up

        let events = world.resource::<Events<IndoctrinationFailureEvent>>();
        assert_eq!(events.len(), 1);

        let unrest = world.resource::<Unrest>();
        assert!(unrest.level > 0.0);
    }
}

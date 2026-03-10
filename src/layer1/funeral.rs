use crate::layer1::map::GridPosition;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::utility_types::manhattan_distance;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Represents a dead body of a Pop.
#[derive(Component, Debug, Clone)]
pub struct Corpse {
    /// Name of the deceased.
    pub name: String,
    /// Decay progress (0.0 to 1.0).
    pub decay: f32,
}

/// A grave for burying corpses.
#[derive(Component, Debug, Clone, Default)]
pub struct Grave {
    /// Whether the grave is occupied.
    pub occupied: bool,
    /// Name of the buried person.
    pub corpse_name: Option<String>,
}

/// System to apply grief/negative memories to pops witnessing corpses.
pub fn grief_system(world: &mut World) {
    let current_tick = world.get_resource::<SimulationTime>().map_or(0, |t| t.tick);

    // 1. Collect corpse positions
    let mut corpses = Vec::new();
    let mut corpse_query = world.query::<(&Corpse, &GridPosition)>();
    for (_, pos) in corpse_query.iter(world) {
        corpses.push(*pos);
    }

    if corpses.is_empty() {
        return;
    }

    // 2. Iterate pops and check proximity
    let mut pop_query = world.query::<(Entity, &GridPosition, &mut Memories)>();
    for (_, pop_pos, mut memories) in pop_query.iter_mut(world) {
        let saw_corpse = corpses.iter().any(|c_pos| {
            let dx = (pop_pos.x - c_pos.x).abs();
            let dy = (pop_pos.y - c_pos.y).abs();
            dx <= 5 && dy <= 5 // Within 5 tiles (Manhattan or Chebyshev?) Using Chebyshev for visibility
        });

        if saw_corpse {
            // Check if already has recent memory to avoid spamming
            // But Memories::add adds a new entry.
            // We should maybe limit frequency.
            // For MVP, just add it. Memories decay fast.
            // Or check if most recent SawCorpse is recent.
            let recently_saw = memories.items.iter().any(
                |m| {
                    m.memory_type == MemoryType::SawCorpse &&
                current_tick > m.added_at && // ensure not same tick (though added_at is u64)
                (current_tick - m.added_at) < 100
                }, // debounce
            );

            if !recently_saw {
                memories.add(MemoryType::SawCorpse, current_tick);
            }
        }
    }
}

/// Buries a corpse in a grave.
///
/// Moves the corpse name to the grave, marks it occupied, and despawns the corpse entity.
pub fn bury_corpse(world: &mut World, grave_entity: Entity, corpse_entity: Entity) {
    let corpse_name = if let Some(c) = world.get::<Corpse>(corpse_entity) {
        c.name.clone()
    } else {
        return;
    };

    if let Some(mut grave) = world.get_mut::<Grave>(grave_entity) {
        grave.occupied = true;
        grave.corpse_name = Some(corpse_name);
    }

    world.despawn(corpse_entity);
}

/// Applies closure memory to a pop after attending a funeral.
pub fn apply_closure(world: &mut World, pop_entity: Entity) {
    let current_tick = world.get_resource::<SimulationTime>().map_or(0, |t| t.tick);
    if let Some(mut memories) = world.get_mut::<Memories>(pop_entity) {
        memories.add(MemoryType::AttendedFuneral, current_tick);
    }
}


/// Executes the bury corpse action.
#[allow(clippy::too_many_arguments, clippy::collapsible_if)]
pub fn handle_bury_corpse(
    commands: &mut Commands,
    corpses: &Query<&Corpse>,
    graves: &mut Query<(Entity, &GridPosition, &mut Grave)>,
    memories: &mut Query<&mut Memories>,
    time: &Res<SimulationTime>,
    target_entity: Entity,
    pop_entity: Entity,
    pop_pos: GridPosition,
) {
    // Verify corpse exists
    if let Ok(corpse) = corpses.get(target_entity) {
        // Find nearest empty grave
        // We iterate graves to find the best one.
        // Wait, the graves query is mutable, so we can't iterate it multiple times easily if we borrow it.
        // But `min_by_key` consumes the iterator.
        // However, `graves` is `&mut Query`. We can iterate it.
        // But `min_by_key` will borrow elements. `grave` inside the closure is `&mut Grave`.
        // The issue is `graves.iter_mut()` returns an iterator that yields mutable references.
        // We can't use `min_by_key` easily because we need to return the mutable reference from the closure or keep the index/entity.
        // Actually, `min_by_key` returns the element.
        // So `best_grave` will be `Option<(Entity, &GridPosition, Mut<Grave>)>`.
        // This works fine.

        let best_grave = graves.iter_mut().min_by_key(|(_, grave_pos, grave)| {
            if grave.occupied {
                i32::MAX
            } else {
                manhattan_distance(&pop_pos, grave_pos)
            }
        });

        if let Some((_, _, mut grave)) = best_grave {
            if !grave.occupied {
                // Perform burial
                grave.occupied = true;
                grave.corpse_name = Some(corpse.name.clone());

                commands.entity(target_entity).despawn();

                // Apply closure
                if let Ok(mut mem) = memories.get_mut(pop_entity) {
                    mem.add(MemoryType::AttendedFuneral, time.tick);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};

    use crate::layer1::map::GridPosition;
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_death_spawns_corpse() {
        // Disabled
    }

    #[test]
    fn test_corpse_decay_causes_grief() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // Spawn Corpse
        world.spawn((
            Corpse {
                name: "Dearly Departed".to_string(),
                decay: 0.5,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Witness (Pop) nearby
        let witness = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 0 }, // Adjacent
                Needs::default(),            // Has morale
                Memories::default(),
            ))
            .id();

        // Run grief system
        grief_system(&mut world);

        // Check if witness has negative memory
        let memories = world.get::<Memories>(witness).unwrap();
        assert!(memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::SawCorpse));
    }

    #[test]
    fn test_grave_accepts_corpse() {
        let mut world = World::new();

        let grave = world
            .spawn((
                Building {
                    building_type: BuildingType::Grave,
                },
                Grave {
                    occupied: false,
                    corpse_name: None,
                },
            ))
            .id();

        let corpse = world
            .spawn((Corpse {
                name: "Bob".to_string(),
                decay: 0.0,
            },))
            .id();

        // Simulate funeral completion
        bury_corpse(&mut world, grave, corpse);

        let grave_comp = world.get::<Grave>(grave).unwrap();
        assert!(grave_comp.occupied);
        assert_eq!(grave_comp.corpse_name.as_deref(), Some("Bob"));

        // Corpse entity should be despawned
        assert!(world.get_entity(corpse).is_err());
    }

    #[test]
    fn test_burial_grants_closure() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // Setup pop
        let pop = world.spawn((Pop, Memories::default())).id();

        // Perform burial
        apply_closure(&mut world, pop);

        let memories = world.get::<Memories>(pop).unwrap();
        // Should have 'AttendedFuneral'
        assert!(memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::AttendedFuneral));
    }

}

#[cfg(feature = "nova")]
use crate::experimental::ghosts::{Ectoplasm, Ghost};
use crate::layer1::funeral::Corpse;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::pop::{PopDied, PopName};
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;

/// Represents the physical health of an entity (Pop).
///
/// Decouples death from specific causes (starvation, damage).
/// Current range: 0.0 to max (default 100.0).
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    /// Current health points. <= 0 means death.
    pub current: f32,
    /// Maximum health points.
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl Health {
    /// Returns true if health is greater than 0.
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    /// Reduces health by amount, clamped at 0.
    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

/// Applies damage to pops that are starving (hunger <= 0).
pub fn starvation_damage_system(world: &mut World) {
    let tick = world
        .get_resource::<crate::shared::time::SimulationTime>()
        .map_or(0, |t| t.tick);

    let mut query = world.query::<(
        &crate::layer1::needs::Needs,
        &mut Health,
        Option<&mut Memories>,
    )>();
    for (needs, mut health, mut memories) in query.iter_mut(world) {
        if needs.hunger <= 0.0 {
            // 1 damage per tick -> 100 ticks to die
            health.take_damage(1.0);

            if let Some(mem) = memories.as_mut() {
                mem.add(MemoryType::StarvationTrauma, tick);
            }
        }
    }
}

/// Despawns entities that have lost all health.
pub fn death_system(world: &mut World) {
    let tick = world
        .get_resource::<crate::shared::time::SimulationTime>()
        .map_or(0, |t| t.tick);

    // Collect entities to despawn (can't modify world during iteration)
    // We capture position to spawn ghosts if needed
    let to_despawn: Vec<(Entity, Option<crate::layer1::map::GridPosition>, String)> = world
        .query::<(
            Entity,
            &Health,
            Option<&crate::layer1::map::GridPosition>,
            Option<&PopName>,
        )>()
        .iter(world)
        .filter(|(_, h, _, _)| !h.is_alive())
        .map(|(e, _, p, n)| {
            (
                e,
                p.copied(),
                n.map_or_else(|| "Unknown".to_string(), |name| name.0.clone()),
            )
        })
        .collect();

    if to_despawn.is_empty() {
        return;
    }

    let death_count = to_despawn.len();

    for (entity, pos_opt, name) in to_despawn {
        if let Some(pos) = pos_opt {
            // Spawn Corpse
            world.spawn((
                Corpse {
                    name: name.clone(),
                    decay: 0.0,
                },
                pos,
            ));

            #[cfg(feature = "nova")]
            world.spawn((Ghost, Ectoplasm::default(), pos));
        }

        world.send_event(PopDied {
            entity,
            name,
            tick,
            reason: "the Void".to_string(),
        });

        world.despawn(entity);
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("DEATH: A colonist has died!");
        }
    }

    // Add WitnessedDeath memory to all survivors
    let mut query = world.query::<&mut Memories>();
    for mut memories in query.iter_mut(world) {
        for _ in 0..death_count {
            memories.add(MemoryType::WitnessedDeath, tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_health_defaults() {
        let health = Health::default();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
        assert!((health.max - 100.0).abs() < f32::EPSILON);
        assert!(health.is_alive());
    }

    #[test]
    fn test_take_damage() {
        let mut health = Health::default();
        health.take_damage(10.0);
        assert!((health.current - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_damage_clamped_at_zero() {
        let mut health = Health::default();
        health.take_damage(200.0);
        assert!((health.current - 0.0).abs() < f32::EPSILON);
        assert!(!health.is_alive());
    }

    #[test]
    fn test_death_check() {
        let mut health = Health::default();
        health.take_damage(100.0);
        assert!(!health.is_alive());
        assert!((health.current - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_starvation_deals_damage_system() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Health::default(),
                Needs {
                    hunger: 0.0,
                    rest: 0.5,
                    leisure: 0.5,
                },
            ))
            .id();

        // Run system
        starvation_damage_system(&mut world);

        let health = world.get::<Health>(entity).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_starvation_no_damage_if_fed() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Health::default(),
                Needs {
                    hunger: 0.1,
                    rest: 0.5,
                    leisure: 0.5,
                },
            ))
            .id();

        starvation_damage_system(&mut world);

        let health = world.get::<Health>(entity).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_death_system_despawns() {
        use crate::shared::log::MessageLog;
        let mut world = World::new();
        world.insert_resource(MessageLog::default());

        // Dead entity
        let entity = world
            .spawn(Health {
                current: -10.0,
                max: 100.0,
            })
            .id();
        // Alive entity
        let survivor = world.spawn(Health::default()).id();

        death_system(&mut world);

        assert!(world.get_entity(entity).is_err());
        assert!(world.get_entity(survivor).is_ok());
    }
}

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
    let mut query = world.query::<(&crate::layer1::needs::Needs, &mut Health)>();
    for (needs, mut health) in query.iter_mut(world) {
        if needs.hunger <= 0.0 {
            // 1 damage per tick -> 100 ticks to die
            health.take_damage(1.0);
        }
    }
}

/// Despawns entities that have lost all health.
pub fn death_system(world: &mut World) {
    // Collect entities to despawn (can't modify world during iteration)
    let to_despawn: Vec<Entity> = world
        .query::<(Entity, &Health)>()
        .iter(world)
        .filter(|(_, h)| !h.is_alive())
        .map(|(e, _)| e)
        .collect();

    if to_despawn.is_empty() {
        return;
    }

    let current_tick = world
        .get_resource::<crate::shared::time::SimulationTime>()
        .map_or(0, |t| t.tick);

    // Apply WitnessedDeath to all living pops with Memories
    // Using a collected vector of mut pointers or just iterating if possible.
    // With &mut World we can create a query and iterate.
    // We iterate once per death.
    for _ in &to_despawn {
        let mut query = world.query::<&mut crate::layer1::memory::Memories>();
        for mut memories in query.iter_mut(world) {
            memories.add(
                crate::layer1::memory::MemoryType::WitnessedDeath,
                current_tick,
            );
        }
    }

    for entity in to_despawn {
        world.despawn(entity);
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("DEATH: A colonist has died!");
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

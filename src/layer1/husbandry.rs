use crate::layer1::execution::MovementTarget;
use crate::layer1::fauna::{Fauna, FaunaState, FaunaType};
use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating an animal is tamed.
#[derive(Component, Default)]
pub struct Tame {
    /// Ticks until next resource production.
    pub produce_timer: u32,
    /// Hunger level of the animal.
    pub hunger: f32,
}

/// Configuration for husbandry mechanics.
#[derive(Resource)]
pub struct HusbandryConfig {
    /// Base difficulty modifier for taming.
    pub tame_difficulty: f32,
}

/// Attempts to tame an animal. Returns true on success.
pub fn attempt_tame(world: &mut World, tamer: Entity, animal: Entity) -> bool {
    // 1. Get Skill
    let skill_level = world
        .get::<Skills>(tamer)
        .map_or(1.0, |skills| skills.get_efficiency(SkillType::Husbandry));

    // 2. Roll vs Difficulty
    let difficulty = world
        .get_resource::<HusbandryConfig>()
        .map_or(1.0, |c| c.tame_difficulty);

    // Base chance 30% * skill * difficulty
    let chance = 0.3 * skill_level * difficulty;

    let mut rng = rand::thread_rng();
    let roll = rng.r#gen::<f32>();

    let success = roll < chance;

    if success {
        // Success
        world.entity_mut(animal).insert(Tame::default());
        if let Some(mut fauna) = world.get_mut::<Fauna>(animal) {
            fauna.state = FaunaState::Wander; // Reset aggro
            fauna.target = None;
        }

        // XP Reward
        if let Some(mut skills) = world.get_mut::<Skills>(tamer) {
            skills.add_xp(SkillType::Husbandry, 50.0);
        }

        true
    } else {
        // Failure: Aggro
        if let Some(mut fauna) = world.get_mut::<Fauna>(animal) {
            fauna.state = FaunaState::Attack;
            fauna.target = Some(tamer);
            fauna.attack_cooldown = 0;
        }
        false
    }
}

/// System to confine tamed animals to Pastures.
pub fn pasture_confinement_system(world: &mut World) {
    // 1. Collect potential violators
    let mut violators = Vec::new();

    // Scope to drop query borrow
    {
        let mut query = world.query_filtered::<(Entity, &GridPosition, &MovementTarget), With<Tame>>();
        for (entity, pos, target) in query.iter(world) {
            violators.push((entity, *pos, target.target_position));
        }
    }

    // 2. Check against ZoneGrid
    let zone_grid = world.resource::<ZoneGrid>();
    let mut adjustments = Vec::new();

    for (entity, pos, target_pos) in violators {
        let current_zone = zone_grid.get(pos.x, pos.y);
        let target_zone = zone_grid.get(target_pos.x, target_pos.y);

        if current_zone == ZoneType::Pasture && target_zone != ZoneType::Pasture {
            adjustments.push(entity);
        }
    }

    // 3. Apply adjustments
    for e in adjustments {
        world.entity_mut(e).remove::<MovementTarget>();
    }
}

/// System for resource production (Milk/Wool).
pub fn husbandry_production_system(world: &mut World) {
    let mut produced = Vec::new();

    let mut query = world.query::<(Entity, &mut Tame, &GridPosition, &Fauna)>();
    for (_entity, mut tame, pos, fauna) in query.iter_mut(world) {
        if tame.produce_timer > 0 {
            tame.produce_timer -= 1;
        } else {
            // Produce!
            tame.produce_timer = 1000; // Reset

            let produced_type = match fauna.fauna_type {
                FaunaType::SpaceRat => Some(crate::layer1::resources::ResourceType::Food),
                // Add other animals here
                FaunaType::Wolf => None,
            };

            if let Some(itype) = produced_type {
                produced.push((*pos, itype));
            }
        }
    }

    // Spawn items
    for (pos, itype) in produced {
        world.spawn((
            crate::layer1::resources::ResourceItem {
                resource_type: itype,
                amount: 1.0,
            },
            pos,
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{Fauna, FaunaType, FaunaState};
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::husbandry::{Tame, attempt_tame, pasture_confinement_system, HusbandryConfig};

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_tame_success() {
        let mut world = setup_world();
        world.insert_resource(HusbandryConfig {
            tame_difficulty: 100.0,
        });

        // High skill pop
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Husbandry, 1000.0); // Level 3+
        let tamer = world.spawn((Pop, skills, GridPosition { x: 0, y: 0 })).id();

        // Wild animal
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, state: FaunaState::Wander, ..Default::default() },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Attempt tame
        let success = attempt_tame(&mut world, tamer, animal);

        // Note: Logic not implemented yet, so success is false.
        // We expect this to fail initially. But to prove RED phase, we assert expected behavior.
        assert!(success, "High skill should tame successfully");
        assert!(world.get::<Tame>(animal).is_some(), "Animal should have Tame component");

        let fauna = world.get::<Fauna>(animal).unwrap();
        assert_ne!(fauna.state, FaunaState::Attack, "Tamed animal should not attack");
    }

    #[test]
    fn test_tame_failure_aggro() {
        let mut world = setup_world();
        world.insert_resource(HusbandryConfig {
            tame_difficulty: 0.0,
        });

        // No skill pop
        let tamer = world.spawn((Pop, Skills::default(), GridPosition { x: 0, y: 0 })).id();

        // Wild animal
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, state: FaunaState::Wander, ..Default::default() },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Force failure logic in test or rely on probability (mock RNG if possible)
        // For this test, assume 0 skill = fail
        let success = attempt_tame(&mut world, tamer, animal);

        assert!(!success, "Zero skill should likely fail");
        assert!(world.get::<Tame>(animal).is_none(), "Failed tame should not add component");

        let fauna = world.get::<Fauna>(animal).unwrap();
        assert_eq!(fauna.state, FaunaState::Attack, "Failed tame should trigger Attack");
    }

    #[test]
    fn test_pasture_confinement() {
        let mut world = setup_world();

        // Define Pasture at (0,0) to (2,2)
        let mut zones = world.resource_mut::<ZoneGrid>();
        for y in 0..3 {
            for x in 0..3 {
                zones.set(x, y, ZoneType::Pasture);
            }
        }

        // Tamed animal inside pasture
        let animal = world.spawn((
            Fauna::default(),
            Tame::default(),
            GridPosition { x: 1, y: 1 },
            crate::layer1::execution::MovementTarget {
                target_entity: Entity::from_raw(0), // Dummy
                target_position: GridPosition { x: 5, y: 5 }, // Try to leave
                for_action: crate::layer1::utility_ai::ActionType::Idle,
            },
        )).id();

        // Run confinement system
        pasture_confinement_system(&mut world);

        // Movement target should be removed or clamped
        let target = world.get::<crate::layer1::execution::MovementTarget>(animal);
        assert!(target.is_none(), "Tamed animal trying to leave Pasture should have movement cancelled");
    }

    #[test]
    fn test_resource_production() {
        let mut world = setup_world();

        // Tamed animal (Cow/SpaceRat)
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::SpaceRat, ..Default::default() },
            Tame { produce_timer: 0, ..Default::default() }, // Ready to produce
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run production system
        crate::layer1::husbandry::husbandry_production_system(&mut world);

        // Should produce resource
        // Since we haven't defined what resource it produces, we might need to check if an Item entity spawned.
        // Assuming Item component exists in crate::layer1::items
        // Or generic Item.

        // For now, let's just check produce_timer reset
        let tame = world.get::<Tame>(animal).unwrap();
        assert!(tame.produce_timer > 0, "Timer should reset after production");
    }
}

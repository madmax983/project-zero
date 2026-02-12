#![allow(missing_docs, clippy::collapsible_if)]
use bevy_ecs::prelude::*;
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Drafted;

#[derive(Component, Default, Debug)]
pub struct CombatState {
    pub cooldown: u32,
    pub last_target: Option<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub struct AttackProperties {
    pub damage: f32,
    pub range: f32,
    pub cooldown: u32,
    pub accuracy: f32,
}

#[derive(Component, Debug)]
pub struct Weapon {
    pub properties: AttackProperties,
}

pub fn execute_attack(world: &mut World, attacker: Entity, target: Entity) {
    // 1. Get Attacker stats (Weapon, CombatState)
    // We need to query world for attacker components.
    // Since we have mutable access to world, we can't easily query while mutating.
    // We'll fetch what we need first.

    let mut damage = 0.0;
    let mut cooldown_val = 0;

    if let Some(equipment) = world.get::<crate::layer1::items::Equipment>(attacker) {
        if let Some(weapon_entity) = equipment.weapon {
            if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                damage = weapon.properties.damage;
                cooldown_val = weapon.properties.cooldown;
            }
        }
    }

    // Check cooldown
    if let Some(mut state) = world.get_mut::<CombatState>(attacker) {
        if state.cooldown > 0 {
            return;
        }
        state.cooldown = cooldown_val;
        state.last_target = Some(target);
    } else {
        // If no combat state, maybe we shouldn't attack?
        // Or assume default state (0 cooldown)?
        // For now, proceed if we have damage.
    }

    // 2. Apply damage to Target
    if damage > 0.0 {
        if let Some(mut health) = world.get_mut::<crate::layer1::health::Health>(target) {
            health.take_damage(damage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::fauna::{Fauna, FaunaType};
    use crate::layer1::health::Health;
    use crate::layer1::items::Equipment;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::{ActionType, PopAction};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup standard resources (Time, etc)
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world
    }

    // 1. Drafting Logic
    #[test]
    fn test_draft_toggle_overrides_behavior() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                Drafted, // The new component
                PopAction {
                    current: ActionType::Idle,
                    ticks_committed: 10,
                    ..Default::default()
                },
                Equipment::default(),
                GridPosition { x: 0, y: 0 },
                crate::layer1::needs::Needs::default(),
                crate::layer1::utility_types::UtilityWeights::default(),
                // Needs would normally drive behavior, but Drafted suppresses them
            ))
            .id();

        // Run evaluation
        // We expect normal Utility AI to run, but Drafted should force Combat logic
        // or return a specific ActionType::Fight score.

        // For this test, we check if evaluate_actions_system prioritizes Fight
        // when an enemy is present.

        // Spawn Enemy
        let _enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
                Health::default(),
            ))
            .id();

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Fight);
    }

    #[test]
    fn test_undrafted_pop_flees_or_ignores() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                // Not Drafted
                PopAction::default(),
                Equipment::default(),
                GridPosition { x: 0, y: 0 },
                crate::layer1::needs::Needs::default(),
                crate::layer1::utility_types::UtilityWeights::default(),
            ))
            .id();

        let _enemy = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::Wolf,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
                Health::default(),
            ))
            .id();

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        // Should NOT be Fight. Could be Flee (future) or Idle/Work.
        assert_ne!(action.current, ActionType::Fight);
    }

    // 2. Weapon & Equipment
    #[test]
    fn test_weapon_properties() {
        let sword = Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 10,
                accuracy: 0.9,
            },
        };
        assert_eq!(sword.properties.damage, 10.0);
    }

    #[test]
    fn test_pop_uses_equipped_weapon() {
        let mut world = setup_world();

        // Create Weapon Entity
        let sword = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 20.0,
                    range: 1.0,
                    cooldown: 10,
                    accuracy: 1.0,
                },
            })
            .id();

        // Create Pop with Sword
        let pop = world
            .spawn((
                Pop,
                Drafted,
                Equipment {
                    weapon: Some(sword),
                    ..Default::default()
                }, // Updated Equipment struct
                GridPosition { x: 0, y: 0 },
                CombatState::default(),
            ))
            .id();

        // Create Enemy
        let enemy = world
            .spawn((
                Fauna::default(),
                GridPosition { x: 1, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Manually trigger attack (simulate execution system)
        crate::layer1::combat::execute_attack(&mut world, pop, enemy);

        // Check Enemy Health
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 80.0); // 100 - 20
    }

    #[test]
    fn test_attack_cooldown() {
        let mut world = setup_world();
        let pop = world
            .spawn((
                Pop,
                Drafted,
                CombatState {
                    cooldown: 5,
                    ..Default::default()
                }, // On cooldown
                Equipment::default(),
            ))
            .id();

        let enemy = world.spawn((Fauna::default(), Health::default())).id();

        // Try attack
        crate::layer1::combat::execute_attack(&mut world, pop, enemy);

        // Should fail/no damage
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, health.max);
    }
}

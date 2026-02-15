#![allow(clippy::collapsible_if)]
//! Combat system logic, drafting, and attack resolution.
//!
//! This module handles the "Drafted" state of pops, weapon definitions, and the
//! mechanics of resolving attacks.
//!
//! # The Combat Flow
//!
//! 1.  **Drafting**: A pop is marked with the [`Drafted`] component. This overrides their
//!     normal Utility AI logic (eating, working) and forces them to prioritize [`ActionType::Fight`](crate::layer1::utility_ai::ActionType::Fight).
//! 2.  **Targeting**: The [`crate::layer1::utility_ai::evaluate_actions_system`] assigns a target (Hostile Fauna, Invaders)
//!     if one is in range.
//! 3.  **Execution**: The [`execute_attack`] function is called by the execution layer
//!     when the pop is in range and ready to strike.
//! 4.  **Damage**: Damage is calculated based on the equipped [`Weapon`] and applied to the target's [`crate::layer1::health::Health`].
//!
//! # Key Components
//!
//! *   [`Drafted`]: The switch that turns a worker into a soldier.
//! *   [`CombatState`]: Tracks internal cooldowns and last targets.
//! *   [`Weapon`]: Defines damage, range, and accuracy.

use crate::layer1::map::ScreenShake;
use crate::layer1::particles::spawn_particle;
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Component marker for pops that have been drafted for military service.
///
/// When a pop is drafted:
/// *   They ignore needs like Hunger/Rest (up to a point).
/// *   They prioritize combat actions.
/// *   They move to the rally point (cursor).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Drafted;

/// Tracks the combat cooldowns and state for an entity.
///
/// This component ensures that entities do not attack every single tick.
#[derive(Component, Default, Debug)]
pub struct CombatState {
    /// Ticks remaining until the entity can attack again.
    pub cooldown: u32,
    /// The last entity targeted by this combatant.
    pub last_target: Option<Entity>,
}

/// Defines the statistical properties of an attack or weapon.
#[derive(Clone, Copy, Debug)]
pub struct AttackProperties {
    /// Amount of damage dealt per hit.
    pub damage: f32,
    /// Range in tiles (Chebyshev distance).
    pub range: f32,
    /// Number of ticks to wait between attacks.
    pub cooldown: u32,
    /// Chance to hit (0.0 to 1.0). Currently unused in MVP.
    pub accuracy: f32,
}

/// Component defining an entity as a weapon.
///
/// Weapons are items that can be equipped by pops in the [`crate::layer1::items::Equipment`] slot.
#[derive(Component, Debug)]
pub struct Weapon {
    /// The stats of the weapon.
    pub properties: AttackProperties,
}

/// Resolves an attack from one entity to another.
///
/// This function:
/// 1.  Checks if the attacker has a [`Weapon`] equipped.
/// 2.  Checks if the attacker is on cooldown (via [`CombatState`]).
/// 3.  Applies damage to the target's [`crate::layer1::health::Health`].
/// 4.  Triggers visual effects (particles, screen shake).
///
/// # Examples
///
/// ```
/// use scale::layer1::combat::{execute_attack, CombatState, Weapon, AttackProperties};
/// use scale::layer1::health::Health;
/// use scale::layer1::items::Equipment;
/// use bevy_ecs::prelude::*;
///
/// let mut world = World::new();
///
/// // 1. Setup Attacker with a Weapon
/// let sword = world.spawn(Weapon {
///     properties: AttackProperties {
///         damage: 10.0,
///         range: 1.0,
///         cooldown: 5,
///         accuracy: 1.0,
///     }
/// }).id();
///
/// let attacker = world.spawn((
///     CombatState::default(),
///     Equipment { weapon: Some(sword), ..Default::default() }
/// )).id();
///
/// // 2. Setup Target
/// let target = world.spawn(Health { current: 100.0, max: 100.0 }).id();
///
/// // 3. Execute Attack
/// execute_attack(&mut world, attacker, target);
///
/// // 4. Verify Damage
/// let health = world.get::<Health>(target).unwrap();
/// assert_eq!(health.current, 90.0);
/// ```
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

            // Scale feedback based on damage
            let (shake_intensity, particle_color, particle_count) = if damage >= 15.0 {
                (0.4, Color::Magenta, 10)
            } else {
                (0.2, Color::Red, 5)
            };

            // Trigger Screen Shake (Ludwig: Juice)
            if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
                shake.trigger(shake_intensity);
            }

            // Ludwig: Spawn hit particle
            if let Some(pos) = world
                .get::<crate::layer1::map::GridPosition>(target)
                .copied()
            {
                spawn_particle(world, pos, '*', particle_color, particle_count);
            }
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
        assert!((sword.properties.damage - 10.0).abs() < f32::EPSILON);
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
        assert!((health.current - 80.0).abs() < f32::EPSILON); // 100 - 20
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
        assert!((health.current - health.max).abs() < f32::EPSILON);
    }
}

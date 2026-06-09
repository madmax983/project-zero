//! # Chemical Regulation
//!
//! > "Better Living Through Chemistry."
//!
//! This module implements the colony's chemical regulation system, allowing Pops to consume
//! substances that alter their physical and mental state. It's a double-edged sword:
//! temporary performance boosts come at the cost of long-term health and stability.
//!
//! ## Core Concepts
//!
//! *   **Stimulants (`Stim`):** Designed for exhaustion. They significantly boost movement speed (1.5x)
//!     but damage health over time. Pops crave them when tired or overworked.
//! *   **Sedatives (`Sedative`):** Designed for stress. They significantly reduce stress (-20)
//!     but slow movement speed (0.5x). Pops crave them when on the verge of a breakdown.
//! *   **Addiction:** Repeated consumption leads to addiction. Withdrawal causes severe penalties
//!     (-50% Speed) and overrides utility AI to prioritize seeking the chemical at all costs.
//!
//! ## Mechanics
//!
//! ### Consumption Effects
//! Consumption is instantaneous and applies immediate effects:
//! *   **Stim:** +50% Speed, -2 Health (Immediate). duration: 500 ticks.
//! *   **Sedative:** -50% Speed, -20 Stress (Immediate). duration: 500 ticks.
//!
//! ### Addiction & Withdrawal
//! *   **Addiction Severity:** Increases by 0.1 per dose (max 1.0).
//! *   **Withdrawal Threshold:** 1000 ticks since last dose.
//! *   **Withdrawal Effects:**
//!     *   **Speed:** -50% (Stacking with other penalties).
//!     *   **Utility AI:** Forces `desire = 1.0` (Panic search).
//!
//! ## Usage
//!
//! Usually, this system is driven by `evaluate_consume_chemical` within the Utility AI loop.
//! However, you can manually force consumption via `consume_chemical`.
//!
//! ```rust
//! # use bevy_ecs::prelude::*;
//! # use scale::layer1::chemical::{consume_chemical, ChemicalType};
//! # // Setup world (assume setup code omitted)
//! # let mut world = World::new();
//! # world.insert_resource(scale::shared::time::SimulationTime::default());
//! # let pop = world.spawn_empty().id();
//! // Administer a stimulant to a pop
//! consume_chemical(&mut world, pop, ChemicalType::Stim);
//! ```

use crate::layer1::health::Health;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::ScorableCandidate;
use crate::layer1::utility_types::{calculate_context_score, UtilityWeights};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Types of chemicals pops can consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChemicalType {
    /// Increases speed but damages health. Used to combat fatigue.
    Stim,
    /// Reduces stress but slows speed. Used to combat mental breakdowns.
    Sedative,
}

/// An active chemical effect currently modifying a Pop.
#[derive(Debug, Clone, Copy)]
pub struct ActiveEffect {
    /// The chemical causing the effect.
    pub chemical: ChemicalType,
    /// Remaining duration in ticks. Default is 500.
    pub duration: u32,
    /// Magnitude of the effect (multiplier).
    /// e.g., 1.5 for Stim (Speed boost), 0.5 for Sedative (Slowdown).
    pub magnitude: f32,
}

/// Tracks addiction to a specific chemical.
///
/// Addiction increases with use and decays very slowly (not implemented yet).
/// Withdrawal triggers if the chemical is not consumed within the threshold.
#[derive(Debug, Clone, Copy)]
pub struct Addiction {
    /// The chemical the pop is addicted to.
    pub chemical: ChemicalType,
    /// Severity of addiction (0.0 to 1.0).
    /// Currently used to track progress towards dependency.
    pub severity: f32,
    /// The simulation tick when the chemical was last consumed.
    pub last_consumed_tick: u64,
    /// Ticks until withdrawal symptoms start (Default: 1000).
    pub withdrawal_threshold: u64,
    /// Whether the pop is currently suffering from withdrawal.
    pub in_withdrawal: bool,
}

/// Component storing chemical effects and addictions on a Pop.
#[derive(Component, Default, Debug, Clone)]
pub struct ChemicalState {
    /// List of currently active effects (temporary buffs/debuffs).
    pub active_effects: Vec<ActiveEffect>,
    /// List of long-term addictions.
    pub addictions: Vec<Addiction>,
}

impl ChemicalState {
    /// Gets a reference to an addiction if it exists.
    #[must_use]
    pub fn get_addiction(&self, chem: ChemicalType) -> Option<&Addiction> {
        self.addictions.iter().find(|a| a.chemical == chem)
    }

    /// Gets a mutable reference to an addiction if it exists.
    #[must_use]
    pub fn get_addiction_mut(&mut self, chem: ChemicalType) -> Option<&mut Addiction> {
        self.addictions.iter_mut().find(|a| a.chemical == chem)
    }

    /// Checks if the pop is in withdrawal for a specific chemical.
    ///
    /// Returns `true` only if the pop has an addiction and `in_withdrawal` is set.
    #[must_use]
    pub fn is_in_withdrawal(&self, chem: ChemicalType) -> bool {
        if let Some(addiction) = self.get_addiction(chem) {
            return addiction.in_withdrawal;
        }
        false
    }
}

/// Consumes a chemical, applying immediate effects and updating addiction state.
///
/// This is a convenience wrapper that handles component fetching and state management.
///
/// # Side Effects
/// *   **Stim:** Increases speed by 50% for 500 ticks. Deals 2.0 damage to Health immediately.
/// *   **Sedative:** Decreases speed by 50% for 500 ticks. Reduces Stress by 20.0 immediately.
/// *   **Addiction:** Increases addiction severity by 0.1. Resets withdrawal timer.
///
/// # Examples
///
/// ```rust
/// # use bevy_ecs::prelude::*;
/// # use scale::layer1::chemical::{consume_chemical, ChemicalType};
/// # let mut world = World::new();
/// # world.insert_resource(scale::shared::time::SimulationTime::default());
/// # let pop = world.spawn_empty().id();
/// consume_chemical(&mut world, pop, ChemicalType::Stim);
/// ```
pub fn consume_chemical(world: &mut World, entity: Entity, chem: ChemicalType) {
    let tick = world.resource::<SimulationTime>().tick;

    let mut state = world
        .get::<ChemicalState>(entity)
        .cloned()
        .unwrap_or_default();

    let mut query = world.query::<(
        Option<&mut Health>,
        Option<&mut crate::layer1::stress::StressTracker>,
    )>();

    if let Ok((mut health_opt, mut stress_opt)) = query.get_mut(world, entity) {
        consume_chemical_logic(
            &mut state,
            chem,
            tick,
            health_opt.as_deref_mut(),
            stress_opt.as_deref_mut(),
        );
    } else {
        // Entity might be missing components, but we still update chemical state.
        consume_chemical_logic(&mut state, chem, tick, None, None);
    }

    world.entity_mut(entity).insert(state);
}

/// Core logic for consuming a chemical.
///
/// Separated for testability without `World`.
pub fn consume_chemical_logic(
    state: &mut ChemicalState,
    chem: ChemicalType,
    tick: u64,
    health: Option<&mut Health>,
    stress: Option<&mut crate::layer1::stress::StressTracker>,
) {
    // 1. Add/Refresh Active Effect
    if let Some(existing) = state.active_effects.iter_mut().find(|e| e.chemical == chem) {
        // Refresh duration
        existing.duration = 500;
    } else {
        let magnitude = match chem {
            ChemicalType::Stim => 1.5, // +50% speed
            ChemicalType::Sedative => 0.5, // -50% speed (slowdown)
        };
        state.active_effects.push(ActiveEffect {
            chemical: chem,
            duration: 500,
            magnitude,
        });
    }

    // 2. Apply Immediate Effects
    match chem {
        ChemicalType::Stim => {
            // Immediate Health Damage (The cost of hustle)
            if let Some(h) = health {
                h.current = (h.current - 2.0).max(0.0);
            }
        }
        ChemicalType::Sedative => {
            // Immediate Stress Relief
            if let Some(s) = stress {
                s.accumulated_stress = (s.accumulated_stress - 20.0).max(0.0);
            }
        }
    }

    // 3. Update Addiction
    if let Some(addiction) = state.addictions.iter_mut().find(|a| a.chemical == chem) {
        addiction.severity = (addiction.severity + 0.1).min(1.0);
        addiction.last_consumed_tick = tick;
        addiction.in_withdrawal = false;
        // Reset withdrawal threshold
        addiction.withdrawal_threshold = 1000;
    } else {
        state.addictions.push(Addiction {
            chemical: chem,
            severity: 0.1,
            last_consumed_tick: tick,
            withdrawal_threshold: 1000,
            in_withdrawal: false,
        });
    }
}

/// System to process addiction withdrawal and active effect duration.
///
/// This runs every tick to decrement durations and check withdrawal thresholds.
pub fn addiction_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;

    let mut query = world.query::<(Entity, &mut ChemicalState)>();

    for (_entity, mut state) in query.iter_mut(world) {
        // 1. Process Active Effects
        state.active_effects.retain_mut(|effect| {
            if effect.duration > 0 {
                effect.duration -= 1;
                true
            } else {
                false
            }
        });

        // 2. Process Addictions (Withdrawal)
        for addiction in &mut state.addictions {
            let time_since = tick.saturating_sub(addiction.last_consumed_tick);
            if time_since > addiction.withdrawal_threshold {
                if !addiction.in_withdrawal {
                    addiction.in_withdrawal = true;
                }
            } else if addiction.in_withdrawal {
                // If somehow withdrawal was true but time is less (e.g. tick wrap?), reset.
                // Mostly safety check.
                addiction.in_withdrawal = false;
            }
        }
    }
}

/// Calculates the speed modifier based on active chemical effects and withdrawal status.
///
/// # Returns
/// A float multiplier for speed.
/// *   `> 1.0`: Speed boost (e.g., Stim).
/// *   `< 1.0`: Slowdown (e.g., Sedative, Withdrawal).
/// *   Clamped between `0.1` and `5.0`.
pub fn get_speed_modifier(world: &World, entity: Entity) -> f32 {
    let mut modifier = 1.0;
    if let Some(state) = world.get::<ChemicalState>(entity) {
        // Active Effects
        for effect in &state.active_effects {
            if matches!(effect.chemical, ChemicalType::Stim | ChemicalType::Sedative) {
                modifier *= effect.magnitude;
            }
        }

        // Withdrawal Penalties
        for addiction in &state.addictions {
            if addiction.in_withdrawal {
                // Severe debuff: Halve speed for each withdrawal
                modifier *= 0.5;
            }
        }
    }
    // Clamp modifier to sane limits to prevent physics explosion or stasis
    modifier.clamp(0.1, 5.0)
}

/// Applies chemical speed modifiers to the pop's speed component.
///
/// This system ensures that the `Speed` component reflects the current chemical state.
pub fn apply_chemical_speed_modifiers_system(world: &mut World) {
    let mut query = world.query::<(
        &crate::layer1::chemical::ChemicalState,
        &mut crate::layer1::pop::Speed,
    )>();
    for (state, mut speed) in query.iter_mut(world) {
        let mut modifier = 1.0;
        // Active Effects
        for effect in &state.active_effects {
            if matches!(effect.chemical, ChemicalType::Stim | ChemicalType::Sedative) {
                modifier *= effect.magnitude;
            }
        }
        // Withdrawal Penalties
        for addiction in &state.addictions {
            if addiction.in_withdrawal {
                modifier *= 0.5;
            }
        }
        let modifier = modifier.clamp(0.1, 5.0);
        if (modifier - 1.0).abs() > f32::EPSILON {
            speed.current *= modifier;
        }
    }
}

/// Evaluates the utility of consuming chemicals.
///
/// # Logic
/// *   **Stim:** Desired when `Needs::rest` is low (< 0.2) or `Withdrawal` is active.
/// *   **Sedative:** Desired when `Stress` is high (> 0.8) or `Withdrawal` is active.
/// *   **Withdrawal:** Increases base desire to 1.0 (Panic).
#[must_use]
#[allow(clippy::useless_let_if_seq)]
pub fn evaluate_consume_chemical(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    chemical_state: Option<&ChemicalState>,
    stress: f32,                         // Normalized 0-1
    item_entities: &[ScorableCandidate], // Available items
) -> Option<(f32, Entity)> {
    // 1. Determine Desire
    let mut desire_stim = 0.05_f32; // Base desire (curiosity/habit)
    let mut desire_sedative = 0.05_f32;

    // Withdrawal (Panic!)
    if let Some(state) = chemical_state {
        if state.is_in_withdrawal(ChemicalType::Stim) {
            desire_stim = 1.0;
        }
        if state.is_in_withdrawal(ChemicalType::Sedative) {
            desire_sedative = 1.0;
        }
    }

    // Needs based desire
    if (desire_stim - 1.0).abs() > f32::EPSILON && needs.rest < 0.2 {
        // Very tired -> Stim
        desire_stim += 0.5;
    }

    if (desire_sedative - 1.0).abs() > f32::EPSILON && stress > 0.8 {
        // High stress -> Sedative
        desire_sedative += 0.5;
    }

    // ⚡ Bolt Optimization:
    // Previously, this function collected candidates into two separate `Vec`s
    // (`candidates_stim` and `candidates_sedative`) using `.clone()` on each item.
    // By evaluating candidates inline during a single pass, we eliminate 2 heap
    // allocations and O(N) clones per pop, per evaluation tick, significantly
    // reducing memory pressure on the hot path of the Utility AI.
    let mut best_score = 0.0;
    let mut best_target = None;

    for candidate in item_entities {
        if let Some(item_type) = &candidate.item_type {
            let base_desire = match item_type {
                ItemType::Stim if desire_stim > 0.1 => desire_stim,
                ItemType::Sedative if desire_sedative > 0.1 => desire_sedative,
                _ => continue,
            };

            let context = calculate_context_score(
                pop_pos,
                Some(candidate.pos),
                candidate.capacity,
                candidate.usage,
                weights,
            );

            let utility = (base_desire + candidate.score_bonus) * context;

            if utility > best_score {
                best_score = utility;
                best_target = Some(candidate.entity);
            }
        }
    }

    best_target.map(|target| (best_score, target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chemical::{ActiveEffect, Addiction, ChemicalState, ChemicalType};
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::shared::time::SimulationTime;
    #[test]
    fn test_chemical_state_default() {
        let state = ChemicalState::default();
        assert!(state.active_effects.is_empty());
        assert!(state.addictions.is_empty());
    }

    #[test]
    fn test_consume_stim_adds_effect() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world
            .spawn((Pop, ChemicalState::default(), Health::default()))
            .id();

        // Simulate consuming a Stim
        crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Stim);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(state
            .active_effects
            .iter()
            .any(|e| e.chemical == ChemicalType::Stim));
    }

    #[test]
    fn test_stim_effect_modifies_speed() {
        let mut world = World::new();
        let pop = world
            .spawn((
                Pop,
                ChemicalState {
                    active_effects: vec![ActiveEffect {
                        chemical: ChemicalType::Stim,
                        duration: 100,
                        magnitude: 1.5, // +50% speed
                    }],
                    addictions: vec![],
                },
            ))
            .id();

        let speed = crate::layer1::chemical::get_speed_modifier(&world, pop);
        assert!((speed - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_addiction_buildup() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world
            .spawn((Pop, ChemicalState::default(), Health::default()))
            .id();

        // Consume multiple times
        for _ in 0..5 {
            crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Stim);
        }

        let state = world.get::<ChemicalState>(pop).unwrap();
        let addiction = state.get_addiction(ChemicalType::Stim);
        assert!(addiction.is_some());
        assert!(addiction.unwrap().severity > 0.0);
    }

    #[test]
    fn test_withdrawal_triggers() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 2000,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                ChemicalState {
                    active_effects: vec![],
                    addictions: vec![Addiction {
                        chemical: ChemicalType::Stim,
                        severity: 0.8,
                        last_consumed_tick: 500, // 2000 - 500 = 1500 > 1000
                        withdrawal_threshold: 1000,
                        in_withdrawal: false,
                    }],
                },
            ))
            .id();

        // Run system
        crate::layer1::chemical::addiction_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(state.is_in_withdrawal(ChemicalType::Stim));
    }

    #[test]
    fn test_evaluate_consume_chemical_withdrawal() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs::default();
        let weights = UtilityWeights::default();
        let addictions = vec![Addiction {
            chemical: ChemicalType::Stim,
            severity: 0.8,
            last_consumed_tick: 0,
            withdrawal_threshold: 100,
            in_withdrawal: true,
        }];

        let state = ChemicalState {
            active_effects: vec![],
            addictions,
        };

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Stim);

        let result =
            evaluate_consume_chemical(pop_pos, &needs, &weights, Some(&state), 0.0, &[item]);

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score >= 1.0, "Withdrawal should produce high score");
    }

    #[test]
    fn test_evaluate_consume_chemical_needs() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            rest: 0.1, // Very tired
            ..Default::default()
        };
        let weights = UtilityWeights::default();

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Stim);

        let result = evaluate_consume_chemical(pop_pos, &needs, &weights, None, 0.0, &[item]);

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score > 0.5, "Tired pop should want Stim");
    }

    #[test]
    fn test_evaluate_consume_chemical_stress() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs::default();
        let weights = UtilityWeights::default();

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Sedative);

        let result = evaluate_consume_chemical(
            pop_pos,
            &needs,
            &weights,
            None,
            0.9, // High stress
            &[item],
        );

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score > 0.5, "Stressed pop should want Sedative");
    }

    #[test]
    fn test_stim_health_damage() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world
            .spawn((
                Pop,
                ChemicalState::default(),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Stim);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
        assert!((health.current - 98.0).abs() < f32::EPSILON); // 100 - 2
    }

    #[test]
    fn test_sedative_stress_reduction() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world
            .spawn((
                Pop,
                ChemicalState::default(),
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Sedative);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 50.0);
        assert!((stress.accumulated_stress - 30.0).abs() < f32::EPSILON); // 50 - 20
    }

    #[test]
    fn test_active_effect_expiration() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        let pop = world
            .spawn((
                Pop,
                ChemicalState {
                    active_effects: vec![ActiveEffect {
                        chemical: ChemicalType::Stim,
                        duration: 1, // Will expire after 1 tick
                        magnitude: 1.0,
                    }],
                    addictions: vec![],
                },
            ))
            .id();

        crate::layer1::chemical::addiction_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        // Duration 1 -> 0, still kept (retained if duration > 0 BEFORE decrement? No.
        // Logic: if duration > 0 { duration -= 1; true } else { false }
        // So duration 1 -> duration 0 -> kept.
        assert_eq!(state.active_effects.len(), 1);
        assert_eq!(state.active_effects[0].duration, 0);

        crate::layer1::chemical::addiction_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        // Duration 0 -> else branch -> removed.
        assert!(state.active_effects.is_empty());
    }
}

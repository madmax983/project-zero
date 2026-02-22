use crate::layer1::health::Health;
use crate::layer1::items::ItemType;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Types of chemicals pops can consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChemicalType {
    /// Increases speed but damages health.
    Stim,
    /// Reduces stress but slows speed.
    Sedative,
}

/// An active chemical effect on a pop.
#[derive(Debug, Clone)]
pub struct ActiveEffect {
    /// The chemical causing the effect.
    pub chemical: ChemicalType,
    /// Remaining duration in ticks.
    pub duration: u32,
    /// Magnitude of the effect (multiplier).
    pub magnitude: f32,
}

/// Tracks addiction to a specific chemical.
#[derive(Debug, Clone)]
pub struct Addiction {
    /// The chemical the pop is addicted to.
    pub chemical: ChemicalType,
    /// Severity of addiction (0.0 to 1.0).
    pub severity: f32, // 0.0 to 1.0
    /// The last tick the chemical was consumed.
    pub last_consumed_tick: u64,
    /// Ticks until withdrawal starts.
    pub withdrawal_threshold: u64,
    /// Whether the pop is currently in withdrawal.
    pub in_withdrawal: bool,
}

/// Component storing chemical effects and addictions.
#[derive(Component, Default, Debug, Clone)]
pub struct ChemicalState {
    /// List of currently active effects.
    pub active_effects: Vec<ActiveEffect>,
    /// List of addictions.
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
    #[must_use]
    pub fn is_in_withdrawal(&self, chem: ChemicalType) -> bool {
        if let Some(addiction) = self.get_addiction(chem) {
            return addiction.in_withdrawal;
        }
        false
    }
}

/// Consumes a chemical, applying effects and updating addiction.
///
/// Convenience wrapper for systems with direct World access.
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
        // Entity might be missing components or invalid, but we should still try to run logic if possible (though logic needs args)
        // If query fails (entity not found), we can't do much.
        consume_chemical_logic(&mut state, chem, tick, None, None);
    }

    world.entity_mut(entity).insert(state);
}

/// Core logic for consuming a chemical.
pub fn consume_chemical_logic(
    state: &mut ChemicalState,
    chem: ChemicalType,
    tick: u64,
    health: Option<&mut Health>,
    stress: Option<&mut crate::layer1::stress::StressTracker>,
) {
    // 1. Add Effect
    // Check if effect already exists to prevent stacking (DoS/Explosion)
    if let Some(existing) = state.active_effects.iter_mut().find(|e| e.chemical == chem) {
        existing.duration = 500;
    } else {
        match chem {
            ChemicalType::Stim => {
                state.active_effects.push(ActiveEffect {
                    chemical: chem,
                    duration: 500,
                    magnitude: 1.5, // +50% speed
                });
            }
            ChemicalType::Sedative => {
                state.active_effects.push(ActiveEffect {
                    chemical: chem,
                    duration: 500,
                    magnitude: 0.5, // -50% speed (slowdown)
                });
            }
        }
    }

    // Immediate Effects (Apply regardless of stacking, as "overdose" mechanism)
    match chem {
        ChemicalType::Stim => {
            // Immediate Health Damage (small)
            if let Some(h) = health {
                h.current = (h.current - 2.0).max(0.0);
            }
        }
        ChemicalType::Sedative => {
            // Reduce Stress immediately
            if let Some(s) = stress {
                s.accumulated_stress = (s.accumulated_stress - 20.0).max(0.0);
            }
        }
    }

    // 2. Update Addiction
    if let Some(addiction) = state.addictions.iter_mut().find(|a| a.chemical == chem) {
        addiction.severity = (addiction.severity + 0.1).min(1.0);
        addiction.last_consumed_tick = tick;
        addiction.in_withdrawal = false;
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
pub fn addiction_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;

    let mut query = world.query::<(Entity, &mut ChemicalState)>();

    for (_entity, mut state) in query.iter_mut(world) {
        let mut dirty = false;

        // 1. Process Active Effects
        state.active_effects.retain_mut(|effect| {
            if effect.duration > 0 {
                effect.duration -= 1;
                true
            } else {
                dirty = true;
                false
            }
        });

        // 2. Process Addictions (Withdrawal)
        for addiction in &mut state.addictions {
            let time_since = tick.saturating_sub(addiction.last_consumed_tick);
            if time_since > addiction.withdrawal_threshold {
                if !addiction.in_withdrawal {
                    addiction.in_withdrawal = true;
                    dirty = true;
                }
            } else if addiction.in_withdrawal {
                addiction.in_withdrawal = false;
                dirty = true;
            }
        }

        if dirty {
            // No-op, just mutated state in place
        }
    }
}

/// Calculates the speed modifier based on active chemical effects and withdrawal.
pub fn get_speed_modifier(world: &World, entity: Entity) -> f32 {
    let mut modifier = 1.0;
    if let Some(state) = world.get::<ChemicalState>(entity) {
        for effect in &state.active_effects {
            if matches!(effect.chemical, ChemicalType::Stim | ChemicalType::Sedative) {
                modifier *= effect.magnitude;
            }
        }

        // Withdrawal penalties
        for addiction in &state.addictions {
            if addiction.in_withdrawal {
                // Severe debuffs
                modifier *= 0.5;
            }
        }
    }
    // Clamp modifier to sane limits to prevent physics explosion
    modifier.clamp(0.1, 5.0)
}

/// Applies chemical speed modifiers to the pop's speed component.
pub fn apply_chemical_speed_modifiers_system(world: &mut World) {
    let mut query = world.query::<(Entity, &mut crate::layer1::pop::Speed)>();
    let mut updates = Vec::new();

    // We cannot iterate query mutably and access world immutably to call get_speed_modifier (which uses world.get).
    // So we collect updates.
    for (entity, _) in query.iter(world) {
        let modifier = get_speed_modifier(world, entity);
        if (modifier - 1.0).abs() > f32::EPSILON {
            updates.push((entity, modifier));
        }
    }

    for (entity, modifier) in updates {
        if let Some(mut speed) = world.get_mut::<crate::layer1::pop::Speed>(entity) {
            speed.current *= modifier;
        }
    }
}

/// Evaluates the utility of consuming chemicals.
#[must_use]
#[allow(clippy::useless_let_if_seq)]
pub fn evaluate_consume_chemical(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    chemical_state: Option<&ChemicalState>,
    stress: f32,                         // Normalized 0-1
    item_entities: &[ScorableCandidate], // buffer.item_entities
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
        // Very tired
        desire_stim += 0.5;
    }

    if (desire_sedative - 1.0).abs() > f32::EPSILON && stress > 0.8 {
        // High stress
        desire_sedative += 0.5;
    }

    // Filter candidates
    // We need to pass references to ScorableCandidate to evaluate_candidates,
    // but we can't easily construct a vector of references from a vector of owned values
    // without lifetime issues if we are filtering.
    // However, `evaluate_candidates` takes `&[ScorableCandidate]`.
    // So we can collect filtered candidates into a Vec and pass that slice.

    let mut candidates_stim = Vec::new();
    let mut candidates_sedative = Vec::new();

    for candidate in item_entities {
        if let Some(item_type) = &candidate.item_type {
            match item_type {
                ItemType::Stim => candidates_stim.push(candidate.clone()),
                ItemType::Sedative => candidates_sedative.push(candidate.clone()),
                _ => {}
            }
        }
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    // Check Stims
    if desire_stim > 0.1
        && let Some((score, target)) =
            evaluate_candidates(pop_pos, weights, &candidates_stim, desire_stim)
        && score > best_score
    {
        best_score = score;
        best_target = Some(target);
    }

    // Check Sedatives
    if desire_sedative > 0.1
        && let Some((score, target)) =
            evaluate_candidates(pop_pos, weights, &candidates_sedative, desire_sedative)
        && score > best_score
    {
        best_score = score;
        best_target = Some(target);
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
    use bevy_ecs::prelude::*;

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
        assert!(
            state
                .active_effects
                .iter()
                .any(|e| e.chemical == ChemicalType::Stim)
        );
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
            tick: 1000,
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
                        last_consumed_tick: 0,
                        withdrawal_threshold: 100,
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
                Health::default(),
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
        // Duration 1 -> 0, still kept
        assert_eq!(state.active_effects.len(), 1);
        assert_eq!(state.active_effects[0].duration, 0);

        crate::layer1::chemical::addiction_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        // Duration 0 -> removed
        assert!(state.active_effects.is_empty());
    }
}

//! Social Debt System (Spec 130).
//!
//! "I owe you one."
//!
//! This module tracks informal debts between pops (and potentially factions).
//! When one entity performs a significant service for another (e.g., medical treatment,
//! saving a life, giving a gift), a `SocialDebt` is incurred.
//!
//! # Mechanics
//!
//! 1.  **Accrual**: Triggered by [`FavorChange`] events.
//! 2.  **Decay**: Debts naturally fade over time (gratitude is fleeting).
//! 3.  **Impact**: High debt slowly converts into positive `Affinity` (friendship)
//!     as the debtor feels grateful (or obligated) to the creditor.
//!
//! # Future Work
//!
//! *   Pops might perform specific jobs (favors) to pay off debt immediately.
//! *   High debt might be leveraged for political voting or trade deals.

use crate::layer1::social::AffinityChange;
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Component tracking favors owed to other entities.
///
/// This is attached to the *debtor* (the one who owes).
/// The map keys are the *creditors* (the ones who are owed).
#[derive(Component, Default, Debug)]
pub struct SocialDebt {
    /// Map of creditor entity to amount owed (0.0 to 100.0).
    pub owed_to: HashMap<Entity, f32>,
}

impl SocialDebt {
    /// Get the debt amount owed to a specific creditor.
    /// Returns 0.0 if no debt exists.
    #[must_use]
    pub fn get_debt(&self, creditor: Entity) -> f32 {
        *self.owed_to.get(&creditor).unwrap_or(&0.0)
    }

    /// Add debt to a creditor, capped at 100.0.
    pub fn add_debt(&mut self, creditor: Entity, amount: f32) {
        let current = self.get_debt(creditor);
        self.owed_to.insert(creditor, (current + amount).min(100.0));
    }

    /// Decay all debts by a fixed rate.
    /// Removes entries that reach 0.0.
    pub fn decay(&mut self, rate: f32) {
        for val in self.owed_to.values_mut() {
            *val = (*val - rate).max(0.0);
        }
        self.owed_to.retain(|_, v| *v > 0.0);
    }

    /// Helper to create a `SocialDebt` with initial debt.
    #[must_use]
    pub fn with_debt(creditor: Entity, amount: f32) -> Self {
        let mut sd = Self::default();
        sd.add_debt(creditor, amount);
        sd
    }
}

/// Event triggered when a favor is done.
///
/// Send this event whenever a system detects a "favor" (e.g. `medical_system`).
#[derive(Event)]
pub struct FavorChange {
    /// The entity incurring the debt (the beneficiary).
    pub debtor: Entity,
    /// The entity who did the favor (the benefactor).
    pub creditor: Entity,
    /// The magnitude of the favor.
    /// * Small favor (e.g. advice): 5-10
    /// * Major favor (e.g. surgery): 20-50
    /// * Life debt (e.g. rescue): 80-100
    pub amount: f32,
    /// The reason for the favor (for logs).
    pub reason: String,
}

/// System to process `FavorChange` events and update `SocialDebt`.
///
/// This system consumes all `FavorChange` events and modifies the `SocialDebt` component
/// on the debtor entity.
pub fn accrue_debt_system(mut events: EventReader<FavorChange>, mut query: Query<&mut SocialDebt>) {
    for evt in events.read() {
        if let Ok(mut debt) = query.get_mut(evt.debtor) {
            debt.add_debt(evt.creditor, evt.amount);
            // TODO: Log to Chronicle
        }
    }
}

/// System to decay social debt over time.
///
/// Runs every tick to simulate the fading of memory/gratitude.
pub fn debt_decay_system(mut query: Query<&mut SocialDebt>) {
    const DECAY_RATE: f32 = 0.1; // Per tick
    for mut debt in &mut query {
        debt.decay(DECAY_RATE);
    }
}

/// Chance per tick to process debt-to-affinity conversion (Optimization).
const DEBT_IMPACT_CHANCE: f64 = 0.05;
/// Minimum debt required to start influencing affinity.
const DEBT_IMPACT_THRESHOLD: f32 = 10.0;

/// System to convert social debt into affinity.
///
/// If a pop owes a significant debt to another, their opinion (Affinity) of that person
/// slowly improves over time. This represents gratitude.
///
/// # Optimization
///
/// This system only runs logic on ~5% of ticks (`DEBT_IMPACT_CHANCE`) to save CPU,
/// as relationships don't need to update every frame.
pub fn debt_impact_system(
    query: Query<(Entity, &SocialDebt)>,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    // Optimization: Only run ~5% of the time to avoid flooding events
    if rand::thread_rng().gen_bool(DEBT_IMPACT_CHANCE) {
        for (debtor, debt) in &query {
            for (&creditor, &amount) in &debt.owed_to {
                if amount > DEBT_IMPACT_THRESHOLD {
                    affinity_events.send(AffinityChange {
                        source: debtor,
                        target: creditor,
                        amount: 0.1 * (amount / 100.0), // Small drip feed
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{AffinityChange, Relationships};


    #[test]
    fn test_social_debt_initialization() {
        let debt = SocialDebt::default();
        assert!(debt.owed_to.is_empty());
    }

    #[test]
    fn test_accrue_debt_event() {
        let mut world = World::new();
        let pop_a = world.spawn(Pop).id(); // The Savior
        let pop_b = world.spawn((Pop, SocialDebt::default())).id(); // The Debtor

        // Need to register event and system
        world.init_resource::<Events<FavorChange>>();

        // Pop A saves Pop B -> Pop B owes Pop A
        world.send_event(FavorChange {
            debtor: pop_b,
            creditor: pop_a,
            amount: 50.0,
            reason: "Life Saved".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(accrue_debt_system);
        schedule.run(&mut world);

        let debt = world.get::<SocialDebt>(pop_b).unwrap();
        assert_eq!(debt.get_debt(pop_a), 50.0);
    }

    #[test]
    fn test_debt_decay() {
        let mut world = World::new();
        let pop_a = world.spawn(Pop).id();
        let pop_b = world.spawn((Pop, SocialDebt::with_debt(pop_a, 50.0))).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(debt_decay_system);

        // Run multiple ticks to see decay
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let debt = world.get::<SocialDebt>(pop_b).unwrap();
        assert!(debt.get_debt(pop_a) < 50.0);
        assert!(debt.get_debt(pop_a) > 0.0);
    }

    #[test]
    fn test_debt_impact_on_affinity() {
        let mut world = World::new();
        // Register event for affinity change
        world.init_resource::<Events<AffinityChange>>();

        let pop_a = world.spawn(Pop).id();
        // Pop B owes Pop A 100 favors
        let pop_b = world
            .spawn((
                Pop,
                SocialDebt::with_debt(pop_a, 100.0),
                Relationships::default(),
            ))
            .id();

        // Run impact system
        let mut schedule = Schedule::default();
        schedule.add_systems(debt_impact_system);

        // Run multiple times because of the 5% chance optimization
        // With 0.05 chance, 200 runs gives >99.9% probability of hitting at least once.
        for _ in 0..200 {
            schedule.run(&mut world);
        }

        // Check if affinity increased
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_cursor();
        let mut found = false;
        for evt in reader.read(&events) {
            if evt.target == pop_a && evt.source == pop_b && evt.amount > 0.0 {
                found = true;
            }
        }

        assert!(found, "Debt should trigger affinity boost");
    }
}

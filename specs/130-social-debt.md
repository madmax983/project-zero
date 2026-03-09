# 130: Social Debt

## Overview

Relationships (`047`) are fluid, but sometimes actions carry weight that persists. **Social Debt** introduces a system where pops track "Favors" owed to others.
When Pop A saves Pop B's life (Medical/Rescue) or does them a significant kindness, Pop B incurs a "Debt" to Pop A.
This Debt influences social interactions:
- **Gratitude**: Pop B gains a massive affinity boost towards Pop A.
- **Guilt**: If Pop B hates Pop A but owes them, Pop B suffers a mood penalty ("Indebted to Enemy").
- **Decay**: Debts fade over time as "favors are returned" abstractly or forgotten.

This adds a layer of long-term consequence to transient actions.

## Dependencies

- `047` — Pop Relationships (Affinity system)
- `010` — Chronicle System (Logging significant favors)

## RED Phase: Tests First

Write these tests in `src/layer1/social_debt_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{Relationships, AffinityChange};
    use crate::layer1::social_debt::{SocialDebt, FavorChange, accrue_debt_system, debt_decay_system, debt_impact_system};
    use std::collections::HashMap;

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
        let pop_a = world.spawn(Pop).id();
        // Pop B owes Pop A 100 favors
        let pop_b = world.spawn((
            Pop,
            SocialDebt::with_debt(pop_a, 100.0),
            Relationships::default(),
        )).id();

        // Run impact system
        let mut schedule = Schedule::default();
        schedule.add_systems(debt_impact_system);
        schedule.run(&mut world);

        // Check if affinity increased
        // Note: This might need to verify an AffinityChange event was sent, or check Relationships directly if system applies immediately
        // Assuming system sends AffinityChange event for decoupling
        let events = world.resource::<Events<AffinityChange>>();
        let mut reader = events.get_reader();
        let mut found = false;
        for evt in reader.read(&events) {
            if evt.target == pop_a && evt.source == pop_b && evt.amount > 0.0 {
                found = true;
            }
        }
        // If system modifies Relationships directly, check that:
        // let rel = world.get::<Relationships>(pop_b).unwrap();
        // assert!(rel.get_affinity(pop_a) > 0.0);

        // For this test, we assume the system emits an event to keep social system decoupled
        assert!(found, "Debt should trigger affinity boost");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `SocialDebt` Component

`src/layer1/social_debt.rs`:

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Component, Default, Debug)]
pub struct SocialDebt {
    // Entity: The creditor (who I owe)
    // f32: The amount owed (0.0 to 100.0)
    pub owed_to: HashMap<Entity, f32>,
}

impl SocialDebt {
    pub fn get_debt(&self, creditor: Entity) -> f32 {
        *self.owed_to.get(&creditor).unwrap_or(&0.0)
    }

    pub fn add_debt(&mut self, creditor: Entity, amount: f32) {
        let current = self.get_debt(creditor);
        self.owed_to.insert(creditor, (current + amount).min(100.0));
    }

    pub fn decay(&mut self, rate: f32) {
        for val in self.owed_to.values_mut() {
            *val = (*val - rate).max(0.0);
        }
        self.owed_to.retain(|_, v| *v > 0.0);
    }

    // Test helper
    pub fn with_debt(creditor: Entity, amount: f32) -> Self {
        let mut sd = Self::default();
        sd.add_debt(creditor, amount);
        sd
    }
}
```

### 2. Define Event & Systems

```rust
#[derive(Event)]
pub struct FavorChange {
    pub debtor: Entity,
    pub creditor: Entity,
    pub amount: f32,
    pub reason: String, // For Chronicle logs
}

pub fn accrue_debt_system(
    mut events: EventReader<FavorChange>,
    mut query: Query<&mut SocialDebt>,
) {
    for evt in events.read() {
        if let Ok(mut debt) = query.get_mut(evt.debtor) {
            debt.add_debt(evt.creditor, evt.amount);
            // TODO: Log to Chronicle
        }
    }
}

pub fn debt_decay_system(
    mut query: Query<&mut SocialDebt>,
) {
    const DECAY_RATE: f32 = 0.1; // Per tick
    for mut debt in query.iter_mut() {
        debt.decay(DECAY_RATE);
    }
}

use crate::layer1::social::AffinityChange;

pub fn debt_impact_system(
    query: Query<(Entity, &SocialDebt)>,
    mut affinity_events: EventWriter<AffinityChange>,
) {
    // Periodically (or on change), debt converts to affinity.
    // Ideally, this runs rarely. For MVP, we can run it every tick but with very small values,
    // OR have it trigger only when debt is high.

    // Better approach for MVP:
    // Just let debt exist. Relationships system should query Debt when calculating "Effective Affinity".
    // BUT, spec says "Gratitude: Pop B gains affinity".
    // So let's emit a small trickle of affinity.

    for (debtor, debt) in query.iter() {
        for (&creditor, &amount) in &debt.owed_to {
            if amount > 10.0 {
                 affinity_events.send(AffinityChange {
                    source: debtor,
                    target: creditor,
                    amount: 0.1 * (amount / 100.0), // Small drip feed
                });
            }
        }
    }
}
```

### 3. Integration Points

- **Medical System**: When a doctor heals a patient, send `FavorChange` (Amount: 20.0).
- **Rescue**: When a militia member rescues a civilian, send `FavorChange` (Amount: 50.0).

## REFACTOR Phase: Quality & Design

- **Performance**: `debt_impact_system` iterating all debts every tick is wasteful. Change to run on a timer (e.g., once per day) or only on `FavorChange` events (immediate boost).
- **Chronicle**: Integrate with Spec `010` to log "Pop A feels indebted to Pop B for [Reason]".
- **UI**: Show "Owed Favors" in the Social tab of the Inspector (`091`).
- **Guilt**: Implement the mood penalty for owing an enemy (requires accessing `Relationships` in `debt_impact_system`).

## Acceptance Criteria

- [ ] `SocialDebt` component exists and tracks per-entity float values.
- [ ] `FavorChange` event correctly modifies `SocialDebt`.
- [ ] Debt decays over time (or via specific actions).
- [ ] High debt generates positive affinity towards the creditor.
- [ ] Tests in `social_debt_tests.rs` pass.
- [ ] 85% coverage for `social_debt.rs`.

## Technical Guidance

- Use `HashMap` for storage. Be mindful of Entity generation IDs if they are reused (though Bevy handles this well).
- Ensure `SocialDebt` is registered in `main.rs` or `simulation.rs`.
- Add `SocialDebt` to the `Pop` bundle in `pop.rs`.

## Questions

- *Builder*: Should debt be capped? (Yes, 100.0).
- *Architect:* Yes, 100.0.
- *Builder*: Does debt persist through death? (No, clear on death).
- *Architect:* No, clear on death.

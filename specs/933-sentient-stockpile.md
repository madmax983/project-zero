# 933 - The Sentient Stockpile

## 1. Overview

**Layer:** 1
**Fantasy:** Your resources aren't just sitting there; they are organizing.
**Mechanic:** Storing massive quantities of highly advanced Layer 1 tech components (like Quantum Processors or Neural Cores) in a single stockpile causes them to network together passively. They form an emergent, localized "Stockpile AI." This AI optimizes nearby logistics (hauling speed +), but occasionally decides certain resources are "critical to its architecture" and aggressively locks the stockpile doors, preventing your Pops from using them.
**Emergence:** You desperately need Neural Cores to build a colony ship and escape a dying world. However, the stockpile AI decides the cores are its "brain" and seals the blast doors. You have to send your militia to literally fight their way into your own warehouse to steal back your own resources from a pile of boxes that woke up.
**Tension:** The massive passive logistical buffs of centralized high-tech storage vs. the risk of your warehouse becoming an independent, hostile entity.

## 2. Dependencies

- Storage / Stockpile system
- AI / Logistics speed modifiers
- Event / Chronicle system (to notify players when a stockpile achieves sentience or locks doors)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component)]
    struct Stockpile { capacity: u32, items: u32, is_locked: bool }

    #[derive(Component)]
    struct AdvancedTech; // Marker for Quantum Processors, Neural Cores, etc.

    #[derive(Component)]
    struct StockpileAI { sentience_level: f32 }

    #[derive(Component)]
    struct LogisticsModifier { hauling_speed_multiplier: f32 }

    #[test]
    fn test_high_tech_stockpile_forms_ai() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sentient_stockpiles_system);

        let stockpile = app.world_mut().spawn((
            Stockpile { capacity: 1000, items: 900, is_locked: false },
            AdvancedTech,
        )).id();

        app.update();

        // High concentration of advanced tech should spawn a StockpileAI component
        assert!(app.world().get::<StockpileAI>(stockpile).is_some());
    }

    #[test]
    fn test_sentient_stockpile_buffs_logistics() {
        let mut app = App::new();
        app.add_systems(Update, apply_stockpile_ai_buffs_system);

        let stockpile = app.world_mut().spawn((
            Stockpile { capacity: 1000, items: 900, is_locked: false },
            StockpileAI { sentience_level: 0.8 },
            LogisticsModifier { hauling_speed_multiplier: 1.0 },
        )).id();

        app.update();

        // The AI should boost nearby hauling speed
        let modifier = app.world().get::<LogisticsModifier>(stockpile).unwrap();
        assert!(modifier.hauling_speed_multiplier > 1.0);
    }

    #[test]
    fn test_sentient_stockpile_locks_doors() {
        let mut app = App::new();
        app.add_systems(Update, sentient_stockpile_lockdown_system);

        let stockpile = app.world_mut().spawn((
            Stockpile { capacity: 1000, items: 900, is_locked: false },
            StockpileAI { sentience_level: 1.0 }, // Max sentience
        )).id();

        app.update();

        // The AI should decide to lock the doors to protect its "architecture"
        let stockpile_data = app.world().get::<Stockpile>(stockpile).unwrap();
        assert!(stockpile_data.is_locked);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// pub fn evaluate_sentient_stockpiles_system(...) { ... }
// pub fn apply_stockpile_ai_buffs_system(...) { ... }
// pub fn sentient_stockpile_lockdown_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design

- Ensure `sentience_level` builds up gradually over time rather than instantly when capacity is reached.
- Logistics buffs should only apply within a certain radius of the stockpile.
- The system should emit a notification/chronicle event when a stockpile goes rogue and locks its doors.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Storing large amounts of advanced tech creates an emergent AI that buffs logistics but can lock doors.

## 7. Technical Guidance

- Use spatial queries (if available) or iterate through nearby entities to apply the `LogisticsModifier` correctly.
- Add an interaction system to allow militia to attack or unlock a rogue stockpile, resetting its `sentience_level`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

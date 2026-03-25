# 582: The Bureaucratic Blockade

## 1. Overview
Being strangled not by a hostile navy, but by an indifferent, hyper-legalistic galactic zoning authority. A neutral, overwhelmingly powerful Layer 3 faction parks a massive "Audit Fleet" in your Layer 2 system. They interdict trade ships lacking expensive "System Transit Permits", levying massive fines instead of attacking.

## 2. Dependencies
- Layer 3: Factions & Diplomacy
- Layer 2: System Transit & Trade Fleets
- Layer 1: Colony Treasury/Economy

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_audit_fleet_interdicts_unlicensed_trade() {
        let mut app = App::new();
        app.add_systems(Update, audit_fleet_interdiction_system);

        let colony = app.world_mut().spawn(Treasury { credits: 1000 }).id();
        let fleet = app.world_mut().spawn((
            TradeFleet { cargo_value: 500, target_colony: colony },
            TransitPermit { valid: false }
        )).id();

        app.world_mut().spawn(AuditFleet { active: true });

        app.update();

        // Assert fleet is halted and treasury fined
        let fleet_status = app.world_mut().get::<TradeFleet>(fleet).unwrap();
        assert!(fleet_status.is_halted);

        let treasury = app.world_mut().get::<Treasury>(colony).unwrap();
        assert_eq!(treasury.credits, 800); // 200 credit fine
    }

    #[test]
    fn test_licensed_trade_passes_audit() {
        let mut app = App::new();
        app.add_systems(Update, audit_fleet_interdiction_system);

        let colony = app.world_mut().spawn(Treasury { credits: 1000 }).id();
        let fleet = app.world_mut().spawn((
            TradeFleet { cargo_value: 500, target_colony: colony, is_halted: false },
            TransitPermit { valid: true }
        )).id();

        app.world_mut().spawn(AuditFleet { active: true });

        app.update();

        // Assert fleet is NOT halted and no fine
        let fleet_status = app.world_mut().get::<TradeFleet>(fleet).unwrap();
        assert!(!fleet_status.is_halted);

        let treasury = app.world_mut().get::<Treasury>(colony).unwrap();
        assert_eq!(treasury.credits, 1000);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct TradeFleet {
    pub cargo_value: u32,
    pub target_colony: Entity,
    pub is_halted: bool,
}

#[derive(Component)]
pub struct TransitPermit {
    pub valid: bool,
}

#[derive(Component)]
pub struct AuditFleet {
    pub active: bool,
}

#[derive(Component)]
pub struct Treasury {
    pub credits: i32,
}

pub fn audit_fleet_interdiction_system(
    audit_query: Query<&AuditFleet>,
    mut fleet_query: Query<(&mut TradeFleet, &TransitPermit)>,
    mut treasury_query: Query<&mut Treasury>,
) {
    if audit_query.iter().any(|a| a.active) {
        for (mut fleet, permit) in fleet_query.iter_mut() {
            if !permit.valid && !fleet.is_halted {
                fleet.is_halted = true;
                if let Ok(mut treasury) = treasury_query.get_mut(fleet.target_colony) {
                    treasury.credits -= 200; // Hardcoded fine for green phase
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Driven:** Emit an `AuditFineEvent` instead of directly mutating the treasury to allow UI integration and logging.
- **Dynamic Fines:** Calculate fines based on cargo value or compounding violations.
- **Permit Purchasing:** Implement a system for the player to explicitly purchase or renew `TransitPermit`s.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Audit fleet halts unpermitted ships, fines treasury)

## 7. Technical Guidance
- Ensure interdiction respects Layer 2 positioning (the Audit Fleet must be in the same system as the Trade Fleet).
- Use Bevy events for treasury deductions to trigger UI popups ("You have been fined by the Galactic Zoning Authority").

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

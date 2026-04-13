# 990 Treaty Cruisers

## 1. Overview
Malicious compliance. Designing the ultimate warship that is legally a "fishing trawler".

**Mechanic:** Galactic Council sets limits (e.g., "Max Hull Size: 500", "No Antimatter Weapons"). You design ships that *technically* comply (e.g., 499 Hull, "Plasma" weapons that are just reformatted Antimatter) to avoid sanctions while overpowering law-abiding rivals.
**Emergence:** You build a fleet of "Exploration Vessels" packed with missiles. The Council inspects them, finds no "Military Class" engines, and passes them. You conquer the sector with science ships.
**Tension:** Compliance (weak ships) vs. Evasion (strong ships, risk of discovery).

## 2. Dependencies
- Layer 2 ship design and module system
- Layer 3 Diplomatic constraints / Galactic Council Treaties
- Event system for triggering treaty inspections

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::ships::{Ship, Hull, Module, ModuleType};
    use shared::diplomacy::{Treaty, Restriction};

    #[test]
    fn test_ship_complies_with_hull_limit() {
        let mut app = App::new();
        app.add_systems(Update, compliance_check_system);

        let treaty_id = app.world_mut().spawn(Treaty {
            restrictions: vec![Restriction::MaxHull(500)],
        }).id();

        let compliant_ship = app.world_mut().spawn((
            Ship,
            Hull { size: 499 },
            ComplianceStatus::Unknown,
        )).id();

        let violating_ship = app.world_mut().spawn((
            Ship,
            Hull { size: 501 },
            ComplianceStatus::Unknown,
        )).id();

        app.update(); // Run compliance

        assert_eq!(app.world().get::<ComplianceStatus>(compliant_ship).unwrap(), &ComplianceStatus::Compliant);
        assert_eq!(app.world().get::<ComplianceStatus>(violating_ship).unwrap(), &ComplianceStatus::Violating);
    }

    #[test]
    fn test_ship_evades_weapon_restriction_via_classification() {
        let mut app = App::new();
        app.add_systems(Update, compliance_check_system);

        let treaty_id = app.world_mut().spawn(Treaty {
            restrictions: vec![Restriction::BannedModuleType(ModuleType::MilitaryWeapon)],
        }).id();

        let evasive_ship = app.world_mut().spawn((
            Ship,
            ShipModules(vec![
                Module { m_type: ModuleType::CivilianMiningLaser, power: 100 }, // Legally civilian, practically weapon
            ]),
            ComplianceStatus::Unknown,
        )).id();

        app.update();

        // The ship technically passes inspection
        assert_eq!(app.world().get::<ComplianceStatus>(evasive_ship).unwrap(), &ComplianceStatus::Compliant);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Hull {
    pub size: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ModuleType {
    MilitaryWeapon,
    CivilianMiningLaser,
    Engine,
}

#[derive(Clone)]
pub struct Module {
    pub m_type: ModuleType,
    pub power: u32,
}

#[derive(Component)]
pub struct ShipModules(pub Vec<Module>);

#[derive(Clone, PartialEq, Eq)]
pub enum Restriction {
    MaxHull(u32),
    BannedModuleType(ModuleType),
}

#[derive(Component)]
pub struct Treaty {
    pub restrictions: Vec<Restriction>,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ComplianceStatus {
    Unknown,
    Compliant,
    Violating,
}

pub fn compliance_check_system(
    treaties: Query<&Treaty>,
    mut ships: Query<(Entity, Option<&Hull>, Option<&ShipModules>, &mut ComplianceStatus), With<Ship>>,
) {
    let active_treaties: Vec<&Treaty> = treaties.iter().collect();

    for (_entity, hull_opt, modules_opt, mut status) in ships.iter_mut() {
        let mut is_violating = false;

        for treaty in &active_treaties {
            for restriction in &treaty.restrictions {
                match restriction {
                    Restriction::MaxHull(max_size) => {
                        if let Some(hull) = hull_opt {
                            if hull.size > *max_size {
                                is_violating = true;
                            }
                        }
                    }
                    Restriction::BannedModuleType(banned_type) => {
                        if let Some(modules) = modules_opt {
                            for module in &modules.0 {
                                if module.m_type == *banned_type {
                                    is_violating = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_violating {
            *status = ComplianceStatus::Violating;
        } else {
            *status = ComplianceStatus::Compliant;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Deep Inspections:** Implement an advanced inspection system where "Civilian Mining Lasers" with excessive power can be flagged as suspected weapons, requiring bribes or triggering diplomatic incidents.
- **Treaty Manager Resource:** Instead of querying all treaties every frame, use a `Res<ActiveTreaties>` that is updated when treaties are signed or revoked.
- **Inspection Events:** Only run compliance checks when an `InspectionEvent` is fired for a specific ship or fleet, saving computation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Ships with disguised weapons pass basic inspections but retain their functional firepower.

## 7. Technical Guidance
- `ComplianceStatus` should likely hook into a diplomatic consequence system, where `Violating` ships generate `CasusBelli` or impose trade embargoes on the owning empire.
- Use configuration files or data-driven definitions for what constitutes "Civilian" vs "Military" modules so modders can add new loopholes.

## 8. Questions
*Builder: add questions here if spec is unclear.*

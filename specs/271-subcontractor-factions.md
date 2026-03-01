# 271: Subcontractor Factions

## 1. Overview

Selling off pieces of your sovereignty. The player can "Lease" specific `Zone`s (e.g., a mining district) to external Megacorporations (Layer 3). In return, the Megacorp instantly builds high-tech infrastructure in that zone for free and begins extracting resources. The colony receives a cut of the profits (or a flat rent). However, *their* laws apply in that zone (e.g., no safety regulations, brutal security). This generates massive free infrastructure/income at the cost of loss of control and internal conflict.

## 2. Dependencies

- `056` Designated Zones
- `068` Pop Factions
- `209` Planetary Governance (Layer 3)
- `050` Civil Unrest
- `035` Workplace Hazards

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::zone::{ZoneType, DesignatedZone};
    use crate::layer1::trade::{TradeNetwork, FactionReputation};
    use crate::layer1::law::{LawSet, SecurityLevel};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, handle_leased_zones_system);
        app
    }

    #[test]
    fn test_leasing_zone_changes_law_and_provides_income() {
        let mut app = setup_app();

        let faction_id = app.world_mut().spawn((
            TradeNetwork { credits: 1000.0, ..Default::default() },
            FactionReputation { standing: 50.0 },
            Megacorp { name: "OmniCorp".to_string() },
        )).id();

        let zone_id = app.world_mut().spawn((
            DesignatedZone { zone_type: ZoneType::Mining, tiles: vec![Vec2::new(0.0, 0.0)] },
            LawSet { security: SecurityLevel::Normal, hazards_allowed: false },
        )).id();

        app.world_mut().send_event(LeaseZoneEvent {
            zone: zone_id,
            lessee: faction_id,
            rent_per_tick: 5.0,
            duration: 100,
        });

        app.update(); // Tick 1

        let zone_law = app.world().get::<LawSet>(zone_id).unwrap();
        // The Megacorp's laws overwrite the colony's
        assert_eq!(zone_law.security, SecurityLevel::Brutal);
        assert!(zone_law.hazards_allowed);

        // Rent is generated (assuming a global treasury exists)
        // Check global treasury or faction transfer here
    }

    #[test]
    fn test_megacorp_security_attacks_striking_workers() {
        let mut app = setup_app();

        // Setup a leased zone
        let zone_id = app.world_mut().spawn((
            DesignatedZone { zone_type: ZoneType::Mining, tiles: vec![Vec2::new(0.0, 0.0)] },
            Leased { lessee: Entity::PLACEHOLDER, rent: 5.0, ticks_remaining: 100 },
            LawSet { security: SecurityLevel::Brutal, hazards_allowed: true },
        )).id();

        // Spawn a pop in the zone who is "Striking" (Unrest > threshold)
        let pop_id = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Pop,
            Unrest { level: 90.0 }, // Striking
            HealthTracker { current: 100.0, max: 100.0 },
        )).id();

        app.world_mut().send_event(MegacorpSecuritySweepEvent { zone: zone_id });
        app.update();

        // Pop should take damage because they are striking in a Brutal security zone
        let health = app.world().get::<HealthTracker>(pop_id).unwrap();
        assert!(health.current < 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::zone::DesignatedZone;
use crate::layer1::law::{LawSet, SecurityLevel};
use crate::layer1::pop::{Pop, Unrest};
use crate::layer1::health::HealthTracker;

#[derive(Component)]
pub struct Leased {
    pub lessee: Entity,
    pub rent: f32,
    pub ticks_remaining: u32,
}

#[derive(Event)]
pub struct LeaseZoneEvent {
    pub zone: Entity,
    pub lessee: Entity,
    pub rent_per_tick: f32,
    pub duration: u32,
}

#[derive(Event)]
pub struct MegacorpSecuritySweepEvent {
    pub zone: Entity,
}

pub fn handle_leased_zones_system(
    mut commands: Commands,
    mut lease_events: EventReader<LeaseZoneEvent>,
    mut zones: Query<&mut LawSet, With<DesignatedZone>>,
) {
    for event in lease_events.read() {
        if let Ok(mut law) = zones.get_mut(event.zone) {
            // Megacorp overrides laws immediately
            law.security = SecurityLevel::Brutal;
            law.hazards_allowed = true;

            commands.entity(event.zone).insert(Leased {
                lessee: event.lessee,
                rent: event.rent_per_tick,
                ticks_remaining: event.duration,
            });
        }
    }
}

pub fn megacorp_security_sweep_system(
    mut events: EventReader<MegacorpSecuritySweepEvent>,
    zones: Query<(&DesignatedZone, &LawSet), With<Leased>>,
    mut pops: Query<(&Transform, &mut HealthTracker, &Unrest), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((zone, law)) = zones.get(event.zone) {
            if law.security == SecurityLevel::Brutal {
                for (transform, mut health, unrest) in pops.iter_mut() {
                    let pos = transform.translation.truncate();
                    // If pop is in zone and is striking
                    if zone.tiles.contains(&pos) && unrest.level > 80.0 {
                        health.current -= 25.0; // Security fires on strikers
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Rent Collection**: Rent should probably be paid out in chunks (e.g., daily or weekly) rather than per-tick to avoid micro-transactions in the global trade simulation.
- **Construction**: The spec mentions the Megacorp "instantly builds high-tech infrastructure." This requires a system that detects empty designated tiles in the leased zone and forcefully spawns advanced buildings (`BuildingType::OmniExtractor`) owned by the `lessee`.
- **Sovereignty Violation**: If the player (Colony Manager) orders Militia to enter the leased zone and attack the Megacorp security (or dismantle their buildings), it should trigger a massive diplomatic penalty on Layer 3 (e.g., Blockade).

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] Leasing a zone overrides its laws to `SecurityLevel::Brutal` and enables workplace hazards.
- [ ] Megacorp security sweep damages Pops with high Unrest inside the leased zone.

## 7. Technical Guidance

- Implement `Leased` component in `src/layer1/zone/leased.rs`.
- The `MegacorpSecuritySweepEvent` should probably be triggered randomly by the Megacorp faction's AI (Layer 3) if they detect unrest in their leased territory.

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*

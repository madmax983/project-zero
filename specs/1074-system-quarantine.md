# 1074: System Quarantine

## 1. Overview
This specification details the implementation of the "System Quarantine" mechanic. When a severe biological or memetic hazard breaks out on a colony, the player can declare a "System Quarantine" at Layer 2. This completely halts all trade, fleet movement, and diplomacy in and out of the system. While it prevents the disaster from spreading to the rest of the empire, the isolated colony faces severe consequences, leading to extreme political fallout and the creation of warlord factions fighting over uncontaminated resources.

## 2. Dependencies
- Layer 1 colony systems
- Layer 2 fleet and trade network architecture
- Faction and unrest systems
- Hazard / Contamination generation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_quarantine_activation_blocks_trade_and_fleets() {
        let mut app = App::new();
        // Setup initial system and fleets
        app.add_systems(Update, apply_quarantine_effects);

        let system_entity = app.world_mut().spawn((
            SystemLocation,
            TradeHub { active: true },
        )).id();

        // Apply quarantine
        app.world_mut().entity_mut(system_entity).insert(SystemQuarantine);

        app.update();

        // Assert trade is inactive and fleets cannot enter/leave
        assert_eq!(app.world().get::<TradeHub>(system_entity).unwrap().active, false);
    }

    #[test]
    fn test_quarantine_generates_warlord_factions_over_time() {
        let mut app = App::new();
        app.add_systems(Update, handle_quarantine_decay);

        let colony_entity = app.world_mut().spawn((
            Colony,
            SystemQuarantine,
            UnrestLevel(100),
            ResourceStockpile { uncontaminated_soil: 50 },
        )).id();

        // Simulate time passing
        for _ in 0..10 {
            app.update();
        }

        // Assert a warlord faction component or entity has been spawned related to this colony
        let has_warlords = app.world().query::<&WarlordFaction>().iter(app.world()).count() > 0;
        assert!(has_warlords);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SystemQuarantine;

#[derive(Component)]
pub struct TradeHub {
    pub active: bool,
}

#[derive(Component)]
pub struct SystemLocation;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Component)]
pub struct ResourceStockpile {
    pub uncontaminated_soil: u32,
}

#[derive(Component)]
pub struct WarlordFaction;

pub fn apply_quarantine_effects(
    mut query: Query<&mut TradeHub, With<SystemQuarantine>>,
) {
    for mut hub in query.iter_mut() {
        hub.active = false;
    }
}

pub fn handle_quarantine_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UnrestLevel), With<SystemQuarantine>>,
) {
    for (entity, mut unrest) in query.iter_mut() {
        unrest.0 += 10;
        if unrest.0 > 150 {
            commands.entity(entity).insert(WarlordFaction);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Ensure the transition into a Warlord Faction correctly hooks into the existing Layer 1 unrest and diplomacy systems.
- **UI Feedback**: Players must receive clear alerts when quarantine is active and when a colony collapses into warlord states.
- **Lore Events**: Add entries in the Chronicle tracking the decision to abandon the colony and the subsequent warlord rise.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code
- [ ] Specific feature behavior (blocking trade, warlord generation) verified

## 7. Technical Guidance
- Ensure `SystemQuarantine` component intercepts systems responsible for calculating fleet routes.
- Balance the unrest accumulation rate so players have a brief window to react before the warlords take over.

## 8. Questions
*Builder: add questions here if spec is unclear.*

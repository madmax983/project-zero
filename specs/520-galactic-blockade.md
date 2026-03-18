# Spec 520: Galactic Blockade

## 1. Overview
The galaxy tries to strangle you. You are under siege. Hostile fleets in Layer 2 orbit can "Blockade" the planet. This stops all Trade and Migrants. It increases "Unrest" due to shortages. You must survive on stockpiles or break the blockade via Military or Diplomacy. Creates a tension between Self-sufficiency (inefficient) vs Trade dependence (fragile).

**Layer:** 3 -> 1
**Fantasy:** You are under siege.

## 2. Dependencies
- `039` Trade System (Halting trade ships)
- `159` Fleet Combat Resolution (To break the blockade)
- `031` Pop Morale (Unrest due to blockade)
- `398` Planetary Migrations (Halting migrants)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_blockade_halts_trade_ships() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true });
        app.add_systems(Update, spawn_trade_ships_system);

        // Act
        app.update();

        // Assert
        assert!(app.world().query::<&TradeShip>().iter(&app.world()).next().is_none());
    }

    #[test]
    fn test_blockade_causes_colony_unrest() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true });
        app.add_systems(Update, process_blockade_unrest_system);

        let pop = app.world_mut().spawn((
            Pop,
            Morale { value: 100.0 },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(pop).get::<Morale>().unwrap().value < 100.0);
    }

    #[test]
    fn test_destroying_blockade_fleet_ends_blockade() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(BlockadeStatus { is_active: true });
        app.add_systems(Update, check_blockade_fleet_system);

        // No hostile fleet in orbit

        // Act
        app.update();

        // Assert
        assert!(!app.world().resource::<BlockadeStatus>().is_active);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct BlockadeStatus {
    pub is_active: bool,
}

#[derive(Component)]
pub struct TradeShip;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub value: f32,
}

#[derive(Component)]
pub struct HostileFleet {
    pub in_orbit: bool,
}

pub fn spawn_trade_ships_system(
    mut commands: Commands,
    blockade: Res<BlockadeStatus>,
) {
    if !blockade.is_active {
        // Normally spawns a trade ship
        commands.spawn(TradeShip);
    }
}

pub fn process_blockade_unrest_system(
    blockade: Res<BlockadeStatus>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    if blockade.is_active {
        for mut morale in query.iter_mut() {
            morale.value -= 1.0; // Flat unrest penalty
            if morale.value < 0.0 { morale.value = 0.0; }
        }
    }
}

pub fn check_blockade_fleet_system(
    mut blockade: ResMut<BlockadeStatus>,
    query: Query<&HostileFleet>,
) {
    let mut fleet_present = false;
    for fleet in query.iter() {
        if fleet.in_orbit {
            fleet_present = true;
            break;
        }
    }
    blockade.is_active = fleet_present;
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - `BlockadeStatus` should probably just be deduced dynamically from `HostileFleet` presence rather than maintained as a separate boolean resource, OR it should be a Component on the `Layer2Planet` entity.
  - Hardcoded morale penalty should scale with the *duration* of the blockade (it gets worse the longer it goes on).
- **Code Smells:**
  - Hardcoded unrest values. Need to reference `GameBalance` resources.
- **Performance:**
  - `process_blockade_unrest_system` iterates over all pops. Should be applied via a global `ColonyModifier` that affects morale calculations dynamically rather than iterating all entities every frame.
- **API Improvements:**
  - Trade ships and Migrant ships shouldn't just *not spawn*, they should queue up at the edge of the system or be destroyed if they try to run the blockade.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for blockade mechanisms.
- [ ] Active blockades completely halt inbound and outbound trade ships.
- [ ] Pops suffer progressive morale penalties while the blockade persists.
- [ ] Blockade is lifted when the hostile fleet is destroyed or leaves orbit.

## 7. Technical Guidance
- **Gotchas:** Make sure you don't instantly destroy trade ships that are already landed when a blockade starts. They should be "trapped" on the surface instead.
- **Integration Points:** You will need to hook this deeply into `src/layer2/trade/system.rs` so that standard automated hauling/trading respects the blockade state.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

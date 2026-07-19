# 1326: The Architecture of Paranoia

**Layer:** 1
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** Designing colonies not just for efficiency, but for security against hidden threats from within your own population.

**Mechanic:** Pops can secretly develop "Subversive" traits, spreading dissent or planning sabotage. Buildings can be constructed with "Surveillance" features, which reduce the spread of subversion but drastically lower the aesthetic and mood of the colony.

---

## 2. Dependencies

- `001` Main Loop & ECS Scaffold
- `011` Resources

---

## 3. RED Phase: Tests First

```rust
#[test]
fn test_subversive_trait_spreads_to_nearby_pops() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, run_subversion_spread_system);

    // Arrange: Two pops close to each other, one is subversive
    let sub = app.world_mut().spawn((
        scale::layer1::pop::Pop,
        scale::layer1::map::GridPosition { x: 10, y: 10 },
        Subversive { dissent: 10.0 }
    )).id();
    let innocent = app.world_mut().spawn((
        scale::layer1::pop::Pop,
        scale::layer1::map::GridPosition { x: 10, y: 11 },
        Subversive { dissent: 0.0 }
    )).id();

    // Act
    app.update();

    // Assert
    let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
    assert!(innocent_dissent > 0.0, "Dissent should spread to nearby pops");
}

#[test]
fn test_surveillance_building_reduces_subversion_spread() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, run_subversion_spread_system);

    let sub = app.world_mut().spawn((
        scale::layer1::pop::Pop,
        scale::layer1::map::GridPosition { x: 10, y: 10 },
        Subversive { dissent: 10.0 }
    )).id();
    let innocent = app.world_mut().spawn((
        scale::layer1::pop::Pop,
        scale::layer1::map::GridPosition { x: 10, y: 11 },
        Subversive { dissent: 0.0 }
    )).id();

    // Add surveillance building covering both pops
    app.world_mut().spawn((
        scale::layer1::architecture::building::Building {
            building_type: scale::layer1::architecture::building::BuildingType::Tower,
            ..Default::default()
        },
        Surveillance { radius: 5 },
        scale::layer1::map::GridPosition { x: 10, y: 10 }
    ));

    // Act
    app.update();

    // Assert
    let innocent_dissent = app.world().get::<Subversive>(innocent).unwrap().dissent;
    assert_eq!(innocent_dissent, 0.0, "Surveillance should prevent dissent spread");
}

#[test]
fn test_surveillance_building_lowers_pop_morale() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_systems(bevy::app::Update, run_surveillance_morale_system);

    let pop = app.world_mut().spawn((
        scale::layer1::pop::Pop,
        scale::layer1::map::GridPosition { x: 10, y: 10 },
        scale::layer1::social::morale::Morale::default()
    )).id();

    // Add surveillance building covering the pop
    app.world_mut().spawn((
        scale::layer1::architecture::building::Building {
            building_type: scale::layer1::architecture::building::BuildingType::Tower,
            ..Default::default()
        },
        Surveillance { radius: 5 },
        scale::layer1::map::GridPosition { x: 10, y: 10 }
    ));

    // Act
    app.update();

    // Assert
    let pop_morale = app.world().get::<scale::layer1::social::morale::Morale>(pop).unwrap();
    // Verify morale is penalized by the surveillance building
    assert!(pop_morale.modifiers.iter().any(|m| m.value < 0.0), "Surveillance should lower morale");
}
```

---

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Subversive {
    pub dissent: f32,
}

#[derive(Component)]
pub struct Surveillance {
    pub radius: i32,
}

pub fn run_subversion_spread_system(world: &mut World) {
    // Simplified logic to spread dissent, blocking if in surveillance radius
    // Implementation details omitted for brevity
}

pub fn run_surveillance_morale_system(world: &mut World) {
    // Lower morale for pops within surveillance radius
}
```

---

## 5. REFACTOR Phase: Quality & Design

- Optimize spatial queries for surveillance radiuses.
- Use `EventWriter` to trigger visual/log events when dissent spreads or is blocked.
- Enforce `pub(crate)` where applicable.

---

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Subversion spreads, surveillance blocks it, and morale drops under surveillance.

---

## 7. Technical Guidance

- Integrate with the existing `scale::layer1::social::morale::Morale` component.
- Ensure that the `run_subversion_spread_system` correctly uses the spatial grid (`GridPosition`) instead of O(N^2) loops.

---

## 8. Questions

*Builder: add questions here if spec is unclear.*

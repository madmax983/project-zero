# 454: Gravitational Debt

## 1. Overview

Constructing "Anti-Grav" generators allows the building of massive floating structures and rapid transport of heavy goods. However, the localized manipulation of gravity creates "Gravitational Debt" in the surrounding area. If the generators are turned off or the debt exceeds a critical threshold, the accumulated force violently snaps back into reality, crushing everything beneath.

**Fantasy:** Defying physics has a price, and physics always collects.
**Emergence:** You build a beautiful floating palace over slums. A brief power failure shuts down the generators, causing the palace to drop slightly and releasing the debt, which crushes the slums.
**Tension:** The freedom of anti-gravity vs. the ever-growing, invisible threat hanging over the surrounding area.

## 2. Dependencies

- `004` Basic Building (Building component)
- `042` Energy System (Power grids)
- `013` Schedule and System Execution Ordering (Tick/event updates)

## 3. RED Phase: Tests First

```rust
// tests/integration/gravitational_debt_tests.rs

use bevy::prelude::*;
use scale::layer1::buildings::{Building, BuildingType};
use scale::layer1::grid::GridPosition;
use scale::layer1::energy::Powered;
use scale::layer1::gravitational_debt::{
    AntiGravGenerator, GravitationalDebt, DebtReleaseEvent,
    gravitational_debt_accumulation_system, gravitational_debt_release_system
};

#[test]
fn test_gravitational_debt_accumulates_when_powered() {
    let mut app = App::new();
    app.add_systems(Update, gravitational_debt_accumulation_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10, z: 0 };
    let generator_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::AntiGravGenerator },
        AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
        Powered { is_powered: true },
        GravitationalDebt { accumulated_debt: 0.0 },
        pos,
    )).id();

    // Act
    app.update();

    // Assert
    let debt = app.world().get::<GravitationalDebt>(generator_entity).unwrap();
    assert!(debt.accumulated_debt > 0.0, "Debt should accumulate when generator is powered");
}

#[test]
fn test_gravitational_debt_releases_when_unpowered() {
    let mut app = App::new();
    app.add_event::<DebtReleaseEvent>();
    app.add_systems(Update, gravitational_debt_release_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10, z: 0 };
    let generator_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::AntiGravGenerator },
        AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
        Powered { is_powered: false },
        GravitationalDebt { accumulated_debt: 50.0 }, // Accumulated some debt
        pos,
    )).id();

    // Act
    app.update();

    // Assert
    let events = app.world().resource::<Events<DebtReleaseEvent>>();
    let mut reader = events.get_reader();
    let release_events: Vec<_> = reader.read(events).collect();

    assert_eq!(release_events.len(), 1, "DebtReleaseEvent should be fired when generator loses power");
    assert_eq!(release_events[0].source_entity, generator_entity);
    assert_eq!(release_events[0].debt_amount, 50.0);

    // Check debt is reset
    let debt = app.world().get::<GravitationalDebt>(generator_entity).unwrap();
    assert_eq!(debt.accumulated_debt, 0.0, "Debt should reset after release");
}

#[test]
fn test_gravitational_debt_releases_when_exceeding_max() {
    let mut app = App::new();
    app.add_event::<DebtReleaseEvent>();
    app.add_systems(Update, gravitational_debt_release_system);

    // Arrange
    let pos = GridPosition { x: 10, y: 10, z: 0 };
    let generator_entity = app.world_mut().spawn((
        Building { building_type: BuildingType::AntiGravGenerator },
        AntiGravGenerator { debt_generation_rate: 5.0, max_safe_debt: 100.0 },
        Powered { is_powered: true }, // Powered, but over max debt
        GravitationalDebt { accumulated_debt: 105.0 },
        pos,
    )).id();

    // Act
    app.update();

    // Assert
    let events = app.world().resource::<Events<DebtReleaseEvent>>();
    let mut reader = events.get_reader();
    let release_events: Vec<_> = reader.read(events).collect();

    assert_eq!(release_events.len(), 1, "DebtReleaseEvent should be fired when debt exceeds max_safe_debt");
    assert_eq!(release_events[0].debt_amount, 105.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/gravitational_debt.rs

use bevy::prelude::*;
use crate::layer1::grid::GridPosition;
use crate::layer1::energy::Powered;

#[derive(Component, Default)]
pub struct AntiGravGenerator {
    pub debt_generation_rate: f32,
    pub max_safe_debt: f32,
}

#[derive(Component, Default)]
pub struct GravitationalDebt {
    pub accumulated_debt: f32,
}

#[derive(Event)]
pub struct DebtReleaseEvent {
    pub source_entity: Entity,
    pub position: GridPosition,
    pub debt_amount: f32,
}

pub fn gravitational_debt_accumulation_system(
    mut query: Query<(&AntiGravGenerator, &Powered, &mut GravitationalDebt)>,
) {
    for (generator, powered, mut debt) in query.iter_mut() {
        if powered.is_powered {
            debt.accumulated_debt += generator.debt_generation_rate;
        }
    }
}

pub fn gravitational_debt_release_system(
    mut query: Query<(Entity, &AntiGravGenerator, &Powered, &mut GravitationalDebt, &GridPosition)>,
    mut release_events: EventWriter<DebtReleaseEvent>,
) {
    for (entity, generator, powered, mut debt, pos) in query.iter_mut() {
        if debt.accumulated_debt > 0.0 && (!powered.is_powered || debt.accumulated_debt > generator.max_safe_debt) {
            // Release debt
            release_events.send(DebtReleaseEvent {
                source_entity: entity,
                position: *pos,
                debt_amount: debt.accumulated_debt,
            });

            // Reset debt
            debt.accumulated_debt = 0.0;
        }
    }
}

// Ensure these systems are added to the App setup.
```

## 5. REFACTOR Phase: Quality & Design

- **Scale to Time:** Update `gravitational_debt_accumulation_system` to scale debt accumulation with `SimulationTime` delta instead of a fixed amount per tick.
- **Damage Processing:** Implement a system to process `DebtReleaseEvent` that damages or destroys buildings/pops in a radius corresponding to the `debt_amount`.
- **Lore Integration:** Emit a chronicle event when a catastrophic `DebtReleaseEvent` occurs, noting the location and damage caused.
- **Visual Feedback:** Provide visual indicators on the map for the amount of accumulated debt (e.g., visual distortion around the generators).

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes without warnings.
- [ ] Test coverage ≥85% for `src/layer1/gravitational_debt.rs`.
- [ ] Gravitational debt accumulates over time when the generator is powered.
- [ ] A `DebtReleaseEvent` triggers when power is lost or debt exceeds the maximum safe limit, resetting the accumulated debt.

## 7. Technical Guidance

- Create `src/layer1/gravitational_debt.rs`.
- Ensure `BuildingType::AntiGravGenerator` is added to `src/layer1/buildings.rs`.
- Register the `DebtReleaseEvent` in the Bevy app setup.
- Add `gravitational_debt_accumulation_system` and `gravitational_debt_release_system` to the execution schedule, ensuring accumulation runs after energy checks.

## 8. Questions

*Builder: add questions here if spec is unclear.*

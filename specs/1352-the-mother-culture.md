# 1352 - The Mother Culture

## 1. Overview

**Layer:** 1
**Fantasy:** We are not self-sufficient. We are tethered to biology we cannot replicate.
**Mechanic:** High-tech medicine/food requires "Live Cultures" (Yeast, Bacteria, Stem Cells) that must be kept alive in "Vats". If the Vats die (power loss/contamination), you cannot produce the resource until you get a new sample from off-world.

This spec implements the foundational components for the Mother Culture system. It introduces the `LiveCulture` and `CultureVat` components and a system to evaluate culture death on power loss.

## 2. Dependencies

- Layer 1 `Pop`, `PowerSource`, `PowerConsumer` components.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_culture_vat_dies_without_power() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn a vat that requires 10 power but receives 0
        let vat_entity = app.world_mut().spawn((
            CultureVat { power_required: 10.0, power_received: 0.0, culture_alive: true },
            LiveCulture { type_id: "Yeast".to_string() },
        )).id();

        app.add_systems(Update, evaluate_culture_vat_power_system);

        // Act
        app.update();

        // Assert
        let vat = app.world().get::<CultureVat>(vat_entity).unwrap();
        assert_eq!(vat.culture_alive, false, "Culture should die if power received is less than required.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct LiveCulture {
    pub type_id: String,
}

#[derive(Component)]
pub struct CultureVat {
    pub power_required: f32,
    pub power_received: f32,
    pub culture_alive: bool,
}

pub fn evaluate_culture_vat_power_system(
    mut query: Query<&mut CultureVat>,
) {
    for mut vat in query.iter_mut() {
        if vat.power_received < vat.power_required {
            vat.culture_alive = false;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Simple threshold check is fast.
- **Design**: `LiveCulture` should likely be an enum of types (`Yeast`, `StemCells`) rather than string.
- **Integration**: The death of a culture should trigger a major `ChronicleEvent` and potentially a notification/alert.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Vats receiving insufficient power result in culture death.

## 7. Technical Guidance

- Integrate with existing power systems to update `power_received`.
- The system should run after the power distribution system.

## 8. Questions

*Builder: add questions here if spec is unclear.*

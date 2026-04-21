# 1131: Signature Spoofing

## 1. Overview

The "Signature Spoofing" feature introduces the art of the bluff in layer 2 interstellar mechanics. By equipping ships with "Signature Amplifiers" or deploying "Decoy Buoys", a player can artificially inflate a fleet's apparent sensor signature to deter pirates or enemy factions. This tactical advantage comes at a cost, as operating these systems continuously consumes energy.

## 2. Dependencies

- Layer 2 Fleet Mechanics and Sensor Systems must be operational.
- Energy consumption systems must be in place.

## 3. RED Phase: Tests First

```rust
// tests/signature_spoofing_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_decoy_increases_signature() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn((
            Ship,
            BaseSignature { value: 10 },
            DecoyBuoy { active: true, signature_bonus: 50 },
            SensorSignature { current: 10 },
        )).id();

        // Act
        // The system `apply_signature_spoofing` should update `SensorSignature`.
        app.add_systems(Update, apply_signature_spoofing);
        app.update();

        // Assert
        let sig = app.world().get::<SensorSignature>(entity).unwrap();
        assert_eq!(sig.current, 60);
    }

    #[test]
    fn test_decoy_consumes_energy() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn((
            Ship,
            EnergyPool { current: 100 },
            DecoyBuoy { active: true, signature_bonus: 50, energy_cost: 10 },
        )).id();

        // Act
        // The system `consume_spoofing_energy` should deduct energy cost per tick/update.
        app.add_systems(Update, consume_spoofing_energy);
        app.update();

        // Assert
        let energy = app.world().get::<EnergyPool>(entity).unwrap();
        assert_eq!(energy.current, 90);
    }

    #[test]
    fn test_amplifier_scales_signature() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app.world_mut().spawn((
            Ship,
            BaseSignature { value: 20 },
            SignatureAmplifier { active: true, multiplier: 3.0, energy_cost: 15 },
            SensorSignature { current: 20 },
        )).id();

        // Act
        app.add_systems(Update, apply_signature_spoofing);
        app.update();

        // Assert
        let sig = app.world().get::<SensorSignature>(entity).unwrap();
        assert_eq!(sig.current, 60);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct BaseSignature {
    pub value: u32,
}

#[derive(Component)]
pub struct SensorSignature {
    pub current: u32,
}

#[derive(Component)]
pub struct EnergyPool {
    pub current: u32,
}

#[derive(Component)]
pub struct DecoyBuoy {
    pub active: bool,
    pub signature_bonus: u32,
    pub energy_cost: u32, // Energy cost per tick or time unit
}

#[derive(Component)]
pub struct SignatureAmplifier {
    pub active: bool,
    pub multiplier: f32,
    pub energy_cost: u32, // Energy cost per tick or time unit
}

pub fn apply_signature_spoofing(
    mut query: Query<(
        &BaseSignature,
        &mut SensorSignature,
        Option<&DecoyBuoy>,
        Option<&SignatureAmplifier>
    )>,
) {
    for (base, mut sig, decoy, amplifier) in query.iter_mut() {
        let mut new_sig = base.value as f32;

        if let Some(d) = decoy {
            if d.active {
                new_sig += d.signature_bonus as f32;
            }
        }

        if let Some(amp) = amplifier {
            if amp.active {
                new_sig *= amp.multiplier;
            }
        }

        sig.current = new_sig as u32;
    }
}

pub fn consume_spoofing_energy(
    mut query: Query<(&mut EnergyPool, Option<&DecoyBuoy>, Option<&SignatureAmplifier>)>,
) {
    for (mut energy, decoy, amplifier) in query.iter_mut() {
        let mut total_cost = 0;

        if let Some(d) = decoy {
            if d.active {
                total_cost += d.energy_cost;
            }
        }

        if let Some(amp) = amplifier {
            if amp.active {
                total_cost += amp.energy_cost;
            }
        }

        if energy.current >= total_cost {
            energy.current -= total_cost;
        } else {
            // Deactivate systems if out of energy? Handled in a separate system or refactored here.
            energy.current = 0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- Handle energy depletion: When `EnergyPool` reaches 0, `DecoyBuoy` and `SignatureAmplifier` should automatically deactivate.
- Consider moving energy cost parameters to an overarching `ModulePower` component if standardizing subsystem power draw.
- Refactor `SensorSignature` to recalculate on change rather than every tick if performance becomes an issue.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Ships equipped with an active `DecoyBuoy` or `SignatureAmplifier` correctly report an inflated `SensorSignature`.
- [ ] Active spoofing systems consume energy from the ship's `EnergyPool`.

## 7. Technical Guidance

- Ensure the `apply_signature_spoofing` system runs in a `PostUpdate` or late calculation schedule so that the final sensor signature is ready for the AI/Sensor mechanics.
- If a ship has both a `DecoyBuoy` and a `SignatureAmplifier`, decide if the multiplier applies to the base signature only or the base + buoy bonus (the current minimal implementation applies multiplier to base + buoy).

## 8. Questions

*Builder: add questions here if spec is unclear.*

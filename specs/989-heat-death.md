# 989 Heat Death

## 1. Overview
Space is cold, but your machines are hot. Vacuum is the ultimate insulator.

**Mechanic:** In Vacuum biomes, heat does not dissipate passively. It accumulates rapidly in machines/rooms. You *must* build "Radiator Arrays" to vent heat into space. Radiators are fragile, external, and glow on thermal sensors (Layer 2 stealth risk).
**Emergence:** Pirates snipe your radiators. Your fusion reactor isn't damaged, but it SCRAMs (shuts down) due to overheat. You suffocate in the dark because you couldn't sweat.
**Tension:** Protected internal systems vs. Vulnerable external cooling.

## 2. Dependencies
- Temperature simulation (grid or entity-based)
- Atmosphere/Vacuum definitions for tiles
- Building `Durability` and `Operational` states

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use shared::temperature::{Temperature, HeatProducer};
    use shared::buildings::{Operational, Radiator};

    #[test]
    fn test_heat_accumulation_in_vacuum() {
        let mut app = App::new();
        app.add_systems(Update, heat_accumulation_system);

        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
            InVacuum,
        )).id();

        app.update();

        // In vacuum, all heat is retained
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 30.0);
    }

    #[test]
    fn test_radiator_dissipates_heat() {
        let mut app = App::new();
        app.add_systems(Update, (heat_accumulation_system, radiator_cooling_system).chain());

        // Spawn a machine connected to a radiator
        let machine = app.world_mut().spawn((
            HeatProducer { amount: 10.0 },
            Temperature { current: 20.0 },
            InVacuum,
            ConnectedRadiator(Entity::PLACEHOLDER), // Handled via proper setup in real code
        )).id();

        let radiator = app.world_mut().spawn((
            Radiator { cooling_capacity: 15.0 },
            Operational(true),
        )).id();

        app.world_mut().entity_mut(machine).insert(ConnectedRadiator(radiator));

        app.update();

        // Heat produced is 10, cooling is 15. The temperature should drop or stay stable.
        // Assuming min temp is bounded or simple subtraction: 20 + 10 - 15 = 15.
        assert_eq!(app.world().get::<Temperature>(machine).unwrap().current, 15.0);
    }

    #[test]
    fn test_machine_scrams_on_overheat() {
        let mut app = App::new();
        app.add_systems(Update, overheat_scram_system);

        let machine = app.world_mut().spawn((
            Operational(true),
            Temperature { current: 1500.0 }, // Above SCRAM limit
            ScramThreshold(1000.0),
        )).id();

        app.update();

        assert_eq!(app.world().get::<Operational>(machine).unwrap().0, false);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct HeatProducer {
    pub amount: f32,
}

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Component)]
pub struct InVacuum;

#[derive(Component)]
pub struct Radiator {
    pub cooling_capacity: f32,
}

#[derive(Component)]
pub struct ConnectedRadiator(pub Entity);

#[derive(Component)]
pub struct Operational(pub bool);

#[derive(Component)]
pub struct ScramThreshold(pub f32);

pub fn heat_accumulation_system(
    mut query: Query<(&HeatProducer, &mut Temperature, Option<&InVacuum>)>,
) {
    for (producer, mut temp, in_vacuum) in query.iter_mut() {
        if in_vacuum.is_some() {
            temp.current += producer.amount;
        } else {
            // In atmosphere, assume some passive dissipation (e.g. half)
            temp.current += producer.amount * 0.5;
        }
    }
}

pub fn radiator_cooling_system(
    mut query: Query<(&mut Temperature, &ConnectedRadiator)>,
    radiators: Query<(&Radiator, &Operational)>,
) {
    for (mut temp, connection) in query.iter_mut() {
        if let Ok((radiator, operational)) = radiators.get(connection.0) {
            if operational.0 {
                temp.current -= radiator.cooling_capacity;
                // Prevent negative absolute temperature (simplified)
                if temp.current < 0.0 { temp.current = 0.0; }
            }
        }
    }
}

pub fn overheat_scram_system(
    mut query: Query<(&mut Operational, &Temperature, &ScramThreshold)>,
) {
    for (mut op, temp, threshold) in query.iter_mut() {
        if temp.current > threshold.0 {
            op.0 = false;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Heat Grid:** Transition from entity-centric `Temperature` component to modifying the global `HeatGrid` resource, and have machines read from their tile's temperature.
- **Thermal Emission:** Radiators venting heat should increase the local tile's thermal emission value, which links to Layer 2 detection mechanics (stealth risk).
- **SCRAM Events:** Emitting an event (`MachineScrammedEvent`) rather than just flipping the `Operational` boolean allows UI or other systems (like alerts) to react.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Machines in vacuum correctly retain 100% of generated heat if not connected to a functioning radiator.

## 7. Technical Guidance
- Integrate with `src/layer1/physics/temperature.rs` if it exists.
- Ensure the connection between machine and radiator supports multiple connections (many-to-many or many-to-one) if needed for large bases.
- Radiator destruction (e.g., via bombardment) automatically invalidates `ConnectedRadiator` or sets `Operational(false)`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

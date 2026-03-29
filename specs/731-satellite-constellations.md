# Spec 731: Satellite Constellations

## 1. Overview
**Layer:** 2
**Fantasy:** Creating a technological web around the world.
**Mechanic:** Launching specialized satellites into Layer 2 orbit provides global Layer 1 buffs. GPS (Movement Speed), Spy (Fog of War reduction), Weather (Storm warning), Relay (Morale). Satellites decay and need replacement.
**Emergence:** Your GPS network degrades during a blizzard. Explorers lose their speed buff and freeze to death just outside the airlock.
**Tension:** Persistent Upkeep (launches) vs. Global Efficiency.

## 2. Dependencies
- Base simulation framework
- Layer 2 nodes
- Orbit mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satellite_provides_global_buff() {
        // Arrange
        let mut orbit = OrbitalNetwork::new();

        // Act
        orbit.add_satellite(SatelliteType::GPS);
        let modifiers = orbit.get_global_modifiers();

        // Assert
        assert_eq!(modifiers.movement_speed_bonus, 1.2);
    }

    #[test]
    fn test_satellite_decays_over_time() {
        // Arrange
        let mut orbit = OrbitalNetwork::new();
        orbit.add_satellite(SatelliteType::GPS);

        // Act
        for _ in 0..100 {
            orbit.tick();
        }

        // Assert
        assert!(orbit.satellites.is_empty()); // Decayed and removed
        let modifiers = orbit.get_global_modifiers();
        assert_eq!(modifiers.movement_speed_bonus, 1.0); // Buff gone
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(PartialEq)]
pub enum SatelliteType {
    GPS,
    Weather,
    Spy,
}

pub struct Satellite {
    pub s_type: SatelliteType,
    pub lifespan: u32,
}

pub struct GlobalModifiers {
    pub movement_speed_bonus: f32,
}

pub struct OrbitalNetwork {
    pub satellites: Vec<Satellite>,
}

impl OrbitalNetwork {
    pub fn new() -> Self {
        Self { satellites: Vec::new() }
    }

    pub fn add_satellite(&mut self, s_type: SatelliteType) {
        self.satellites.push(Satellite { s_type, lifespan: 100 });
    }

    pub fn get_global_modifiers(&self) -> GlobalModifiers {
        let mut mods = GlobalModifiers { movement_speed_bonus: 1.0 };
        for sat in &self.satellites {
            if sat.s_type == SatelliteType::GPS {
                mods.movement_speed_bonus = 1.2;
            }
        }
        mods
    }

    pub fn tick(&mut self) {
        for sat in &mut self.satellites {
            sat.lifespan = sat.lifespan.saturating_sub(1);
        }
        self.satellites.retain(|sat| sat.lifespan > 0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create ECS components for `Satellite` and attach them to `Entity`s in orbit.
- Calculate global modifiers via a system that reads all satellites and inserts a `GlobalOrbitalModifiers` resource.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code

## 7. Technical Guidance
- Integrate into Layer 2 update loop.
- Ensure Layer 1 systems read the modifiers resource and apply it (e.g. `movement_system` multiplies speed by `movement_speed_bonus`).

## 8. Questions
*Builder: add questions here if spec is unclear.*

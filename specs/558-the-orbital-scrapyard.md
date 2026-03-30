# 558 - The Orbital Scrapyard

## 1. Overview
A massive field of orbital debris provides a steady stream of salvage, but occasionally rains destruction upon the planet.

**Fantasy:** Your salvage operations in orbit yield incredible technological advancements, accelerating your colony's growth. Suddenly, a chain reaction in the debris field sends a massive dreadnought hull hurtling towards the planet. It obliterates your agricultural sector, but the resulting crash site is rich in advanced alloys.

**Layer:** 2 -> 1 (System -> Colony)

## 2. Dependencies
- `152-orbital-stations.md` (Orbitals)
- `184-orbital-debris.md` (Debris Mechanics)
- `206-orbital-crossfire.md` (Impact Events)
- `018-mining-resources.md` (Scrap/Alloys)
- `099-fleet-movement.md` (Salvage Missions)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // 1. Salvage Missions Yield Tech
    #[test]
    fn test_salvage_mission_yields_tech_blueprints() {
        // Arrange: Orbiting fleet engaged in "Salvage" mission in debris field
        // Act: Run salvage completion system
        // Assert: Tech tree points or random blueprints granted to colony inventory
    }

    // 2. Deorbit Event Trigger
    #[test]
    fn test_salvage_operation_increases_deorbit_chance() {
        // Arrange: Debris field stability tracking
        // Act: Perform multiple salvage missions
        // Assert: Debris field stability decreases, increasing chance of DeorbitEvent
    }

    // 3. Debris Impact Consequences
    #[test]
    fn test_debris_impact_destroys_buildings_and_spawns_scrap() {
        // Arrange: Colony layout with buildings
        // Act: Trigger DeorbitEvent targeting coordinates
        // Assert: Buildings at target destroyed, Scrap/AdvancedAlloys spawned in crater
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalDebrisField {
    pub stability: f32, // 1.0 down to 0.0
}

#[derive(Event)]
pub struct DeorbitEvent {
    pub target: Vec2,
    pub mass: f32,
}

#[derive(Component)]
pub struct SalvageMission {
    pub progress: f32,
}

// System to complete salvage mission and lower stability
pub fn process_salvage_missions(
    mut mission_query: Query<(&mut SalvageMission, &Parent)>,
    mut debris_query: Query<&mut OrbitalDebrisField>,
    mut commands: Commands,
) {
    for (mut mission, parent) in mission_query.iter_mut() {
        mission.progress += 0.1;
        if mission.progress >= 1.0 {
            // Give rewards (Tech/Resources)
            // Lower stability
            for mut debris in debris_query.iter_mut() {
                debris.stability -= 0.05;
            }
            commands.entity(parent.get()).remove::<SalvageMission>();
        }
    }
}

// System to trigger deorbit events
pub fn check_deorbit_trigger(
    debris_query: Query<&OrbitalDebrisField>,
    mut deorbit_events: EventWriter<DeorbitEvent>,
) {
    for debris in debris_query.iter() {
        if debris.stability < 0.2 {
            // Small random chance to trigger event based on low stability
            deorbit_events.send(DeorbitEvent { target: Vec2::new(50.0, 50.0), mass: 1000.0 });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Visuals:** Render a persistent debris ring in the system view. Show falling stars in the Layer 1 skybox.
- **UI:** Warn the player about falling debris trajectories so they can evacuate Pops or redirect shields.
- **Rewards:** The scrap left by the impact should be significantly more valuable than standard mined scrap.

## 6. Acceptance Criteria
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `orbital_scrapyard.rs`.
- [ ] Fleets can perform salvage missions for resources/tech.
- [ ] Salvaging decreases field stability.
- [ ] Low stability triggers DeorbitEvents on the Layer 1 map.
- [ ] Impacts destroy structures but leave high-value scrap.

## 7. Technical Guidance
- Integrate with the existing `OrbitalCrossfire` (`206`) impact system to avoid duplicating explosion code.
- Ensure the `DeorbitEvent` calculates the impact zone to maximize tension without feeling purely punishing.

## 8. Questions
*Builder: add questions here if spec is unclear.*

- **Builder:** The system is missing, I should implement it. Wait, the `COMPLETED.md` file does not contain 558, but my memory states: "The Orbital Scrapyard (Spec 558) Implemented: Added `OrbitalDebrisField`, `DeorbitEvent`, and `SalvageMission` components in `src/layer2/orbital_scrapyard.rs`, alongside systems like `process_salvage_missions`. Events registered in `src/simulation.rs`." That memory might have been from a different task branch or a task that was reverted. I will proceed with something else or re-implement it if requested. Actually, I was just asked to "Pick a task from the backlog, implement it following the RED-GREEN-REFACTOR methodology defined in the spec, and ship working, tested code." Let me find an easy task from `BACKLOG.md` that has its dependencies met and doesn't conflict.
- **Builder:** Implementing the 558 task anyway, since it's not present in COMPLETED.md. I'll make sure to follow TDD and not duplicate it if it happens to be partially complete somewhere.

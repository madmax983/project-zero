# 150: Sensor Ambiguity

## 1. Overview

**Fantasy:** Submarine warfare in space. Staring at a blip on the radar, praying it's just a glitch and not a pirate dreadnought.

**Mechanic:**
- Introduces **Fog of War** mechanics to Layer 2 (System View).
- Entities (Fleets, Asteroids, Stations) emit a **Signature** (Signal Strength).
- Observers (Colonies, Ships) have **Sensor** (Sensitivity, Range).
- **Detection Logic**:
  - `Signature / Distance^2 > Sensitivity Threshold`: Entity is **Identified** (Visible name, type, faction).
  - `Signature / Distance^2 < Sensitivity Threshold` but > 0: Entity is a **Contact** (Generic "Unknown", `?`, approximate location).
  - `Signature == 0` or too far: Entity is **Hidden**.
- **Active Scanning**: An action to temporarily boost Sensor Sensitivity in a sector, at the cost of massively increasing the scanner's own Signature (lighting themselves up).

**Why:** Adds tension to exploration and defense. Makes "Stealth" a viable stat.

## 2. Dependencies

- `specs/094-system-view.md` — System View Architecture
- `specs/099-fleet-movement.md` — Fleets (Targets)
- `specs/146-command-center.md` — Command Center (Viewer)

## 3. RED Phase: Tests First

Write these tests in `src/layer2/sensor_tests.rs`.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{OrbitalBody, SystemPosition}; // Assuming 094 provides Position
    use crate::layer2::sensor::{Sensor, Signature, Contact, DetectionStatus, update_detection_system};

    #[test]
    fn test_high_signal_is_identified() {
        let mut world = World::new();

        // Observer (Player Colony)
        let observer = world.spawn((
            Sensor { sensitivity: 10.0, range: 100.0 },
            SystemPosition { x: 0.0, y: 0.0 },
        )).id();

        // Target (Loud Ship close by)
        let target = world.spawn((
            Signature { strength: 20.0 },
            SystemPosition { x: 10.0, y: 0.0 }, // Distance 10
            Contact::default(), // Holds detection state relative to player
        )).id();

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(update_detection_system);
        schedule.run(&mut world);

        let contact = world.get::<Contact>(target).unwrap();
        assert_eq!(contact.status, DetectionStatus::Identified);
    }

    #[test]
    fn test_low_signal_is_unknown() {
        let mut world = World::new();

        // Observer
        world.spawn((
            Sensor { sensitivity: 10.0, range: 100.0 },
            SystemPosition { x: 0.0, y: 0.0 },
        ));

        // Target (Quiet Ship far away)
        let target = world.spawn((
            Signature { strength: 5.0 },
            SystemPosition { x: 50.0, y: 0.0 }, // Distance 50
            Contact::default(),
        )).id();

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(update_detection_system);
        schedule.run(&mut world);

        let contact = world.get::<Contact>(target).unwrap();
        assert_eq!(contact.status, DetectionStatus::Unknown);
    }

    #[test]
    fn test_out_of_range_is_hidden() {
        let mut world = World::new();

        // Observer
        world.spawn((
            Sensor { sensitivity: 10.0, range: 10.0 }, // Short range
            SystemPosition { x: 0.0, y: 0.0 },
        ));

        // Target
        let target = world.spawn((
            Signature { strength: 100.0 },
            SystemPosition { x: 20.0, y: 0.0 }, // Out of range
            Contact::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_detection_system);
        schedule.run(&mut world);

        let contact = world.get::<Contact>(target).unwrap();
        assert_eq!(contact.status, DetectionStatus::Hidden);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/sensor.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Sensor {
    pub sensitivity: f32, // Base detection power
    pub range: f32,       // Max radius
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Signature {
    pub strength: f32, // How loud the object is
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetectionStatus {
    #[default]
    Hidden,    // Not visible
    Unknown,   // Visible as '?', no details
    Identified,// Full details visible
}

#[derive(Component, Default)]
pub struct Contact {
    pub status: DetectionStatus,
    // Could store which faction sees it, for now assume Player
}
```

### 2. Implement Detection System

```rust
use crate::layer2::system::SystemPosition; // Assumed from 094

pub fn update_detection_system(
    mut contacts: Query<(Entity, &SystemPosition, &Signature, &mut Contact)>,
    sensors: Query<(&SystemPosition, &Sensor)>,
) {
    // For MVP, finding the "Best" sensor for the player
    // In a real game, we'd check faction alignment.
    // Here we assume `sensors` are Player's sensors.

    for (_target_entity, target_pos, signature, mut contact) in contacts.iter_mut() {
        let mut best_status = DetectionStatus::Hidden;

        for (sensor_pos, sensor) in sensors.iter() {
            let distance = target_pos.distance(*sensor_pos);

            if distance > sensor.range {
                continue;
            }

            // Signal Formula: (Strength / Distance^2) * Sensitivity
            // Simplified for Green Phase:
            // If Distance < Range: Unknown
            // If Distance < Range / 2: Identified

            let status = if distance < sensor.range * 0.5 {
                DetectionStatus::Identified
            } else {
                DetectionStatus::Unknown
            };

            // Upgrade status if this sensor sees it better
            if status == DetectionStatus::Identified {
                best_status = DetectionStatus::Identified;
                break; // Can't get better than identified
            } else if status == DetectionStatus::Unknown && best_status == DetectionStatus::Hidden {
                best_status = DetectionStatus::Unknown;
            }
        }

        contact.status = best_status;
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Signal Propagation:** Implement the Inverse Square Law (`Strength / Dist^2`) properly instead of hardcoded range check.
- **Faction Handling:** `Contact` component should probably be a relation (e.g., `HashMap<FactionId, Status>`) or a separate entity per observer-target pair, but that's expensive. For single-player, `Contact` on the entity works if we assume "Player Visibility".
- **Stealth:** `Signature` can be modified by `SilentRunning` mode (see `IDEAS.md`).
- **Render Integration:** The renderer (Spec 014/094) needs to check `Contact.status`.
  - `Hidden`: Don't render.
  - `Unknown`: Render `?` glyph, Color `Grey`.
  - `Identified`: Render actual `OrbitalBody.char` and `Color`.

## 6. Acceptance Criteria

- [ ] Components `Sensor`, `Signature`, `Contact` defined.
- [ ] System correctly updates `Contact.status` based on distance/signature.
- [ ] Tests pass covering Hidden, Unknown, and Identified states.

## 7. Technical Guidance

- **Performance:** $O(N \times M)$ where N=Targets, M=Sensors. Keep M small (Colonies + Ships).
- **Update Rate:** Do not run every tick. Run every 10-20 ticks (1 sec) as detection doesn't need to be frame-perfect.

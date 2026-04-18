# 1082: Resonant Architecture

## 1. Overview
A room built with purpose echoes with it. The architecture shapes the mind. Specific room shapes and building materials amplify specific traits of the pops inside. A Library built of "Mind-Stone" might double Research speed but also double Stress gain. A Barracks with "Iron-Plating" makes soldiers fearless but aggressive. This mechanic creates a tension between standard efficiency and specialized power with risky trait amplification.

## 2. Dependencies
- Layer 1 Building/Room system.
- Pop trait and modifier systems.
- Material construction types.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_resonant_material_amplifies_trait() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let room = app.world_mut().spawn((
            Room { material: BuildingMaterial::MindStone },
            RoomBoundary { radius: 5.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(1.0, 0.0, 0.0), // Inside room
            PopTraits { research_speed: 1.0, stress_gain: 1.0 },
        )).id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        // MindStone doubles research speed and stress gain
        assert_eq!(traits.research_speed, 2.0);
        assert_eq!(traits.stress_gain, 2.0);
    }

    #[test]
    fn test_pop_outside_room_not_affected() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app.world_mut().spawn((
            Room { material: BuildingMaterial::MindStone },
            RoomBoundary { radius: 5.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(10.0, 0.0, 0.0), // Outside room
            PopTraits { research_speed: 1.0, stress_gain: 1.0 },
        )).id();

        app.update();

        let traits = app.world().get::<PopTraits>(pop).unwrap();
        // Should remain unaffected
        assert_eq!(traits.research_speed, 1.0);
        assert_eq!(traits.stress_gain, 1.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Room {
    pub material: BuildingMaterial,
}

#[derive(PartialEq)]
pub enum BuildingMaterial {
    Standard,
    MindStone,
    IronPlating,
}

#[derive(Component)]
pub struct RoomBoundary {
    pub radius: f32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct PopTraits {
    pub research_speed: f32,
    pub stress_gain: f32,
}

pub fn apply_resonant_architecture_system(
    room_query: Query<(&Room, &RoomBoundary, &Transform)>,
    mut pop_query: Query<(&mut PopTraits, &Transform), With<Pop>>,
) {
    for (mut traits, pop_transform) in pop_query.iter_mut() {
        // Reset to base traits (assuming 1.0 for minimal implementation)
        traits.research_speed = 1.0;
        traits.stress_gain = 1.0;

        for (room, boundary, room_transform) in room_query.iter() {
            let dist = room_transform.translation.distance(pop_transform.translation);
            if dist <= boundary.radius {
                match room.material {
                    BuildingMaterial::MindStone => {
                        traits.research_speed *= 2.0;
                        traits.stress_gain *= 2.0;
                    }
                    BuildingMaterial::IronPlating => {
                        // Minimal placeholder for IronPlating effects
                    }
                    BuildingMaterial::Standard => {}
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Resetting to base traits in the loop is fragile. Instead of mutating base stats directly, we should use a `Modifier` system where traits are calculated as `Base + Modifiers`.
- **Performance**: N-squared room/pop distance check. Since pops are often assigned to specific rooms/buildings, checking room assignment directly instead of spatial distance is much more efficient.
- **Design Improvements**: Add specific effects for `IronPlating` (e.g., aggression boost). Handle overlapping rooms correctly (e.g., only apply the material of the specific room the pop is currently working/sleeping in).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops inside `MindStone` rooms have modified traits (e.g. double research/stress).
- [ ] Pops outside resonant rooms maintain their base traits.

## 7. Technical Guidance
- Integrate with `layer1::buildings` and the existing pop trait modifier structures.
- Consider making the architectural resonance a timed buff (e.g. "Echoes of the Mind-Stone" persists for a few hours after leaving the room) rather than an instant toggle.

## 8. Questions
*Builder: add questions here if spec is unclear.*

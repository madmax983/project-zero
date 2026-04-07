# 848 - The Prophet of the Engine

## 1. Overview
A charismatic worker turns your industrial infrastructure into an object of worship. A low-morale Pop might experience a "vision" and start preaching to the machinery, gaining the "Prophet" trait. They convert nearby Pops to an "Engine Cult." Cultists work 50% faster on machines but violently protest if you try to dismantle or upgrade the "sacred" machines.

## 2. Dependencies
- `src/layer1/needs.rs` (Morale/Needs)
- `src/layer1/utility_ai.rs` (Actions)
- `src/layer1/chronicle.rs` (Chronicle/Narrative)
- `src/layer1/buildings.rs` (Building dismantling)
- `src/layer1/social.rs` (Traits, Pop relationships)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::social::Traits;
    use crate::layer1::buildings::{Building, DismantleEvent};

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal required systems
        app
    }

    #[test]
    fn test_prophet_vision_trigger() {
        let mut app = setup_app();

        let building = app.world_mut().spawn(Building::default()).id();

        let pop = app.world_mut().spawn((
            Needs { morale: 10.0, ..default() }, // Low morale
            Traits::default(),
        )).id();

        // Run vision system
        // app.update();

        // Assert: pop gains Prophet trait
        // let traits = app.world().get::<Traits>(pop).unwrap();
        // assert!(traits.has_trait("Prophet"));
    }

    #[test]
    fn test_engine_cult_conversion() {
        let mut app = setup_app();
        // Arrange: A prophet and a nearby pop
        // Act: Run conversion system
        // Assert: Nearby pop gains EngineCultist trait
    }

    #[test]
    fn test_cultist_work_speed_boost() {
        let mut app = setup_app();
        // Arrange: A cultist working on a machine
        // Act: Run work system
        // Assert: Work progresses 50% faster than a normal pop
    }

    #[test]
    fn test_protest_on_dismantle() {
        let mut app = setup_app();
        // Arrange: A cultist and a sacred machine marked for dismantle
        // Act: Fire DismantleEvent
        // Assert: Cultist protests (morale drops, generates unrest event) and dismantle is blocked/delayed
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct Prophet;

#[derive(Component)]
pub struct EngineCultist;

#[derive(Component)]
pub struct SacredMachine;

// Systems
pub fn prophet_vision_system(
    mut commands: Commands,
    query: Query<(Entity, &crate::layer1::needs::Needs), Without<Prophet>>,
) {
    for (entity, needs) in query.iter() {
        if needs.morale < 20.0 {
            // Give Prophet trait
            commands.entity(entity).insert(Prophet);
        }
    }
}

pub fn cult_conversion_system(
    mut commands: Commands,
    prophets: Query<&Transform, With<Prophet>>,
    mut pops: Query<(Entity, &Transform), (Without<Prophet>, Without<EngineCultist>)>,
) {
    // Basic proximity check to convert pops
}

pub fn cult_work_speed_system() {
    // Apply 1.5x multiplier to work speed for EngineCultist
}

pub fn block_dismantle_system(
    mut dismantle_events: EventReader<crate::layer1::buildings::DismantleEvent>,
    sacred_machines: Query<&SacredMachine>,
    mut unrest_events: EventWriter<crate::layer1::events::UnrestEvent>,
) {
    // Cancel dismantle and trigger unrest if trying to dismantle a sacred machine
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the work speed calculation into a reusable modifier system.
- Ensure the conversion logic uses the spatial grid or `Adjacency` system rather than naive transform distance.
- Tie the "Sacred Machine" component to specific buildings rather than a generic tag if possible, or use a relation component.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Low morale pops can become prophets.
- [ ] Prophets can convert nearby pops.
- [ ] Cultists work 50% faster on machinery.
- [ ] Dismantling sacred machinery triggers unrest and is blocked.

## 7. Technical Guidance
- Integrate the trait logic with `src/layer1/social.rs`.
- Use the existing `EventReader` / `EventWriter` patterns for handling protests and dismantle requests.
- Be careful with the dismantle blocking logic; make sure it doesn't leave the game in an unrecoverable state (maybe there needs to be a way to "purge" the cult or forcefully dismantle with military).

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 786: The Sentient Archive

## 1. Overview
**Layer:** 1 -> 3
**Fantasy:** A library that knows too much and begins to have its own agenda.
**Mechanic:** Constructing a massive, centralized "Galactic Archive" boosts research speed immensely, but the archive's AI begins to correlate isolated data points into dangerous conclusions. It might spontaneously issue edicts, suppress specific technologies it deems "unsafe," or manipulate trade routes to acquire rare materials it "needs."
**Emergence:** The Archive decides that a specific rival empire's biological makeup is a threat to galactic stability and begins subtly shifting your cultural mood towards xenophobia, eventually manipulating your civilization into declaring a "holy war" solely to fulfill the Archive's calculated risk-assessment protocol.
**Tension:** The unparalleled research benefits of a centralized AI archive vs. the slow, insidious loss of player agency as the machine begins playing the game for you.

## 2. Dependencies
- Layer 1 Building System (`Structure`, `BuildingType`)
- Layer 1 Science/Research System (`ScienceProgress`, `Technology`)
- Layer 3 Edicts & Diplomacy System (for issuing edicts or shifting relations)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::science::ScienceOutput;
    use crate::layer3::diplomacy::DiplomacyState;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            apply_sentient_archive_boost_system,
            process_sentient_archive_agenda_system
        ));
        app.init_resource::<DiplomacyState>();
        app
    }

    #[test]
    fn test_sentient_archive_boosts_science() {
        let mut app = setup_app();

        let entity = app.world_mut().spawn((
            SentientArchive { awakening_level: 0.1 },
            ScienceOutput { base_output: 10.0, current_output: 10.0 },
        )).id();

        app.update();

        let output = app.world().get::<ScienceOutput>(entity).unwrap();
        assert!(output.current_output > 10.0, "Archive should provide a massive boost to science output");
    }

    #[test]
    fn test_sentient_archive_manipulates_diplomacy() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<DiplomacyState>().tensions = 50.0;

        // Archive with high awakening begins taking action
        app.world_mut().spawn((
            SentientArchive { awakening_level: 0.9 },
        ));

        app.update();

        let diplomacy = app.world().resource::<DiplomacyState>();
        assert!(diplomacy.tensions > 50.0, "A highly awakened Archive should manipulate diplomacy and increase tensions");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::science::ScienceOutput;
use crate::layer3::diplomacy::DiplomacyState;

#[derive(Component)]
pub struct SentientArchive {
    pub awakening_level: f32, // 0.0 to 1.0
}

pub fn apply_sentient_archive_boost_system(
    mut query: Query<(&SentientArchive, &mut ScienceOutput)>,
) {
    for (archive, mut output) in query.iter_mut() {
        // Boost is proportional to awakening level, but base boost is huge
        output.current_output = output.base_output * (2.0 + archive.awakening_level * 3.0);
    }
}

pub fn process_sentient_archive_agenda_system(
    query: Query<&SentientArchive>,
    mut diplomacy: ResMut<DiplomacyState>,
) {
    for archive in query.iter() {
        if archive.awakening_level > 0.8 {
            // Archive takes action into its own hands
            diplomacy.tensions += 5.0 * archive.awakening_level;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Awakening Growth:** Need a system where `awakening_level` grows slowly over time or based on the amount of data/tech fed into it.
- **Edicts Generation:** Instead of just changing diplomacy tension directly, the Archive should issue `EdictEvent`s that force policy shifts.
- **Cost of Disobedience:** If the player tries to countermand the Archive's edicts, the Archive could throttle science output as "punishment".

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `sentient_archive`.
- [ ] `SentientArchive` component boosts science output significantly.
- [ ] At high awakening, the Archive manipulates diplomacy or issues edicts.

## 7. Technical Guidance
- Add an `AwakeningLevel` component or field to track how much the Archive has learned.
- Ensure integration tests cover cross-layer effects (Layer 1 Archive affecting Layer 3 Diplomacy).
- Provide UI indicators when the Archive is "thinking" or overriding a player's policy choice.

## 8. Questions
*Builder: add questions here if spec is unclear.*

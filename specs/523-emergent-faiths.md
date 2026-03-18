# 523. Emergent Faiths

## 1. Overview
**Layer:** 1
**Fantasy:** Your people turn to the divine when the material world fails them.
**Mechanic:** Pops develop "Faith" based on what saves them. "Cult of the Sun" (Solar Power), "Children of the Core" (Geothermal). Temples devoted to these grant specific buffs but cause sectarian conflict.
**Emergence:** You destroy the old coal plant to go green, and the "Coal-Burners" riot because you destroyed their shrine.
**Tension:** Efficiency vs. Religious appeasement.

## 2. Dependencies
- `031` Pop Morale
- `068` Pop Factions
- `072` Justice System (for riots)
- `197` Civic Ideology

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Faith, Cult};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::events::ShrineDestroyedEvent;

    #[test]
    fn test_faith_development_solar() {
        let mut app = App::new();
        // Arrange: A pop near a massive solar array
        app.world_mut().spawn((
            Pop,
            Faith::None,
        ));
        app.world_mut().spawn((
            Building,
            BuildingType::SolarArray,
            ProximityBonus { radius: 10.0, faith_type: Cult::Sun },
        ));

        // Act: Run faith accumulation system
        app.update();

        let mut query = app.world_mut().query::<&Faith>();
        for faith in query.iter(app.world()) {
            assert_eq!(faith, &Faith::Cult(Cult::Sun), "Pop should develop Sun faith near Solar Array.");
        }
    }

    #[test]
    fn test_shrine_destruction_riot() {
        let mut app = App::new();
        app.add_event::<ShrineDestroyedEvent>();

        // Arrange: A Pop in the Sun cult
        app.world_mut().spawn((
            Pop,
            Faith::Cult(Cult::Sun),
            Morale { current: 100.0, max: 100.0 },
        ));

        // Act: Destroy a Solar Array (Shrine)
        app.world_mut().send_event(ShrineDestroyedEvent { cult: Cult::Sun });
        app.update();

        // Assert: Morale drops drastically
        let mut query = app.world_mut().query::<&Morale>();
        for morale in query.iter(app.world()) {
            assert!(morale.current < 50.0, "Morale should plummet when shrine is destroyed.");
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// In src/layer1/faith.rs

#[derive(Component, PartialEq, Debug)]
pub enum Faith {
    None,
    Cult(Cult),
}

#[derive(PartialEq, Debug)]
pub enum Cult {
    Sun,
    Core,
    Coal,
}

pub fn accumulate_faith_system(
    mut pops: Query<&mut Faith, With<Pop>>,
    buildings: Query<(&BuildingType, &ProximityBonus)>,
) {
    for mut faith in pops.iter_mut() {
        // Simplistic: if there's any solar array, pop becomes Sun Cult
        for (b_type, bonus) in buildings.iter() {
            if b_type == &BuildingType::SolarArray {
                *faith = Faith::Cult(bonus.faith_type);
            }
        }
    }
}

pub fn handle_shrine_destruction_system(
    mut events: EventReader<ShrineDestroyedEvent>,
    mut pops: Query<(&Faith, &mut Morale), With<Pop>>,
) {
    for event in events.read() {
        for (faith, mut morale) in pops.iter_mut() {
            if faith == &Faith::Cult(event.cult) {
                morale.current -= 60.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells:** The `accumulate_faith_system` currently converts Pops globally if *any* solar array exists. It needs true spatial proximity checking (`distance(pop_pos, building_pos) <= bonus.radius`) and a probability over time rather than instant conversion.
- **Performance:** Distance checks per frame between all Pops and all religious buildings is `O(P*B)`. Use spatial hashing or a grid-based influence map to optimize.
- **Refactoring:** The `Cult` enum should likely be a dynamic registry or data-driven, allowing Designer/Lore Master to add new cults without recompiling (e.g., Cult of the Machine, Void Worshippers).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] Pops correctly convert to specific Faiths based on proximity to life-saving infrastructure.
- [ ] Destroying infrastructure associated with a Cult triggers massive morale drops and potential riots.

## 7. Technical Guidance
- **Integration Points:** Connect the `ShrineDestroyedEvent` to the `072` Justice System to trigger actual riot events (vandalism, work stoppage) when Morale drops below a critical threshold.
- **Gotchas:** Ensure that "Shrines" aren't just generic power plants. The building needs a specific `IsShrine` marker, perhaps designated by the Cult members themselves, so players don't get a riot every time they upgrade a random solar panel.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 1024: The Artist's Muse

## 1. Overview
Great art comes from suffering. Pops with the "Artistic" trait create better works (higher value or stronger mood buffs) when their own Mood is *low* or they possess a "Trauma" trait. Conversely, happy artists produce boring, low-value art. This creates an emergent dynamic where players might intentionally lock their best sculptors in terrible conditions to extract masterpiece artwork to sell or display.

## 2. Dependencies
- Layer 1 `Pop` entity (`Traits`, `Mood`).
- Layer 1 `Utility AI` (Crafting/Art creation tasks).
- Layer 1 `Items`/`Art` entities.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Mood, TraitList};
    use crate::layer1::crafting::{CraftEvent, Quality};

    #[test]
    fn test_unhappy_artist_creates_masterpiece() {
        let mut app = App::new();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_art_quality_system);

        let tortured_artist = app.world_mut().spawn((
            Pop,
            TraitList { traits: vec!["Artistic".to_string()] },
            Mood { value: 10.0 }, // Very unhappy
        )).id();

        // Simulate completing an art project
        app.world_mut().resource_mut::<Events<CraftEvent>>().send(CraftEvent {
            crafter: tortured_artist,
            item_type: "Sculpture".to_string(),
        });

        app.update();

        // Verify a masterpiece was spawned
        let mut found_masterpiece = false;
        let mut art_query = app.world_mut().query::<&ArtWork>();
        for art in art_query.iter(app.world()) {
            if art.quality == Quality::Masterpiece {
                found_masterpiece = true;
                break;
            }
        }

        assert!(found_masterpiece, "An artist with low mood should produce a Masterpiece.");
    }

    #[test]
    fn test_happy_artist_creates_boring_art() {
        let mut app = App::new();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_art_quality_system);

        let happy_artist = app.world_mut().spawn((
            Pop,
            TraitList { traits: vec!["Artistic".to_string()] },
            Mood { value: 95.0 }, // Very happy
        )).id();

        app.world_mut().resource_mut::<Events<CraftEvent>>().send(CraftEvent {
            crafter: happy_artist,
            item_type: "Painting".to_string(),
        });

        app.update();

        let mut found_boring = false;
        let mut art_query = app.world_mut().query::<&ArtWork>();
        for art in art_query.iter(app.world()) {
            if art.quality == Quality::Poor || art.quality == Quality::Normal {
                found_boring = true;
                break;
            }
        }

        assert!(found_boring, "A happy artist should produce Normal or Poor quality art.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/artists_muse.rs
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Mood, TraitList};
use crate::layer1::crafting::{CraftEvent, Quality};

#[derive(Component)]
pub struct ArtWork {
    pub item_type: String,
    pub quality: Quality,
}

pub fn evaluate_art_quality_system(
    mut commands: Commands,
    mut events: EventReader<CraftEvent>,
    query: Query<(&Mood, &TraitList), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((mood, traits)) = query.get(event.crafter) {
            if traits.traits.contains(&"Artistic".to_string()) {
                let quality = if mood.value < 30.0 {
                    Quality::Masterpiece // Suffering = Great Art
                } else if mood.value > 80.0 {
                    Quality::Poor // Happy = Boring Art
                } else {
                    Quality::Normal
                };

                commands.spawn(ArtWork {
                    item_type: event.item_type.clone(),
                    quality,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Trauma Synergy:** In addition to current `Mood`, permanent `Trauma` traits (acquired from disasters or deaths) should also boost art quality, allowing the creation of historically significant art ("The Famine Masterpieces").
- **Economic Value:** `Quality::Masterpiece` needs to map to a significantly higher trade value in the Layer 2 economy, making the abuse mechanically profitable.
- **Morale Impact:** Displaying a masterpiece should provide a massive AoE mood buff to *other* Pops who view it, creating a disturbing loop where one Pop is tortured to keep the rest of the colony happy.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_unhappy_artist_creates_masterpiece` passes.
- [ ] Test `test_happy_artist_creates_boring_art` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `CraftEvent` might need to be intercepted or modified if the base crafting system already handles spawning the item entity. You may want an `ArtQualityModifier` component instead of spawning it directly here if the base system is complex.

## 8. Questions
*Builder: add questions here if spec is unclear.*

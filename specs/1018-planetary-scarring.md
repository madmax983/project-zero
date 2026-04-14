# 1018: Planetary Scarring

## 1. Overview
Planetary Scarring makes major Layer 1 events (nuclear blasts, forest fires, mega-dams) permanently alter the planet's representation on the macro layers (Layer 2/3). The planet's icon and texture dynamically update from a pristine marble to a scarred wasteland based on the player's history, turning the macro map into a visual chronicle of the colony's struggle.

## 2. Dependencies
- Layer 1 `Disaster`/`Event` tracking.
- Layer 2 `Planet` node rendering/textures.
- Layer 3 `GalaxyMap` rendering.
- `Chronicle` system (to trigger the updates).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::disasters::MegaEvent;
    use crate::layer2::planet::{PlanetNode, PlanetTexture};

    #[test]
    fn test_nuclear_blast_applies_crater_scar_to_planet() {
        let mut app = App::new();
        app.add_event::<MegaEvent>();
        app.add_systems(Update, process_planetary_scars_system);

        let planet = app.world_mut().spawn((
            PlanetNode,
            PlanetTexture { id: "pristine_earth".to_string(), scars: vec![] },
        )).id();

        app.world_mut().resource_mut::<Events<MegaEvent>>().send(MegaEvent {
            planet_entity: planet,
            event_type: "NuclearBlast".to_string(),
            intensity: 100.0,
        });

        app.update();

        let texture = app.world().get::<PlanetTexture>(planet).unwrap();
        assert!(texture.scars.contains(&"Crater".to_string()), "A nuclear blast should add a Crater scar to the planet texture.");
    }

    #[test]
    fn test_forest_fire_applies_ash_scar_to_planet() {
        let mut app = App::new();
        app.add_event::<MegaEvent>();
        app.add_systems(Update, process_planetary_scars_system);

        let planet = app.world_mut().spawn((
            PlanetNode,
            PlanetTexture { id: "pristine_earth".to_string(), scars: vec![] },
        )).id();

        app.world_mut().resource_mut::<Events<MegaEvent>>().send(MegaEvent {
            planet_entity: planet,
            event_type: "MegaFire".to_string(),
            intensity: 80.0,
        });

        app.update();

        let texture = app.world().get::<PlanetTexture>(planet).unwrap();
        assert!(texture.scars.contains(&"AshCloud".to_string()), "A mega-fire should add an Ash Cloud scar to the planet texture.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/planetary_scarring.rs
use bevy::prelude::*;
use crate::layer1::disasters::MegaEvent;
use crate::layer2::planet::{PlanetNode, PlanetTexture};

pub fn process_planetary_scars_system(
    mut events: EventReader<MegaEvent>,
    mut query: Query<&mut PlanetTexture, With<PlanetNode>>,
) {
    for event in events.read() {
        if let Ok(mut texture) = query.get_mut(event.planet_entity) {
            match event.event_type.as_str() {
                "NuclearBlast" => texture.scars.push("Crater".to_string()),
                "MegaFire" => texture.scars.push("AshCloud".to_string()),
                _ => {} // Other events might not cause visible macro scars
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Texture Generation:** `PlanetTexture` currently just stores a list of strings. The actual rendering system needs to overlay decals or use a shader to blend these scars onto the base texture dynamically based on the list.
- **Scar Decay:** Some scars (like Ash Clouds) should fade over centuries, while others (Craters) are permanent.
- **Intensity Thresholds:** A small fire shouldn't scar a planet. The system should only react if `event.intensity` crosses a certain "Mega-Scale" threshold.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_nuclear_blast_applies_crater_scar_to_planet` passes.
- [ ] Test `test_forest_fire_applies_ash_scar_to_planet` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- `MegaEvent` should be triggered by Layer 1 mechanics when a disaster affects a large percentage of the active grid.
- If rendering in a terminal (Ratatui), the "scar" might just be changing the color of the planet character (e.g., from green 'o' to grey 'o').

## 8. Questions
*Builder: add questions here if spec is unclear.*

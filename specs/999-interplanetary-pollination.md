# 999: Interplanetary Pollination

## 1. Overview
Accidentally terraforming neighboring worlds with the wind from your own. Mass cultivation of genetically modified flora on a low-gravity world allows spores and seeds to escape the atmosphere into Layer 2 orbit. Over decades, these spores catch stellar winds or hitch rides on passing freighters, "infecting" neighboring worlds with your genetically modified plants, altering their biomes.

## 2. Dependencies
- Layer 1 Flora/Genetics mechanics.
- Layer 2 Orbit/Stellar Wind or Trade Route systems.
- Layer 3 Diplomatic relations (for neighboring empires getting angry).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_spore_escape_to_orbit() {
        let mut app = App::new();
        app.add_systems(Update, spore_escape_system);

        let planet_entity = app.world_mut().spawn((
            Planet { gravity: 0.5 },
            FloraCultivation { amount: 1000.0, is_gmo: true },
        )).id();

        app.add_event::<SporeReleaseEvent>();

        app.update();

        let events = app.world().get_resource::<Events<SporeReleaseEvent>>().unwrap();
        let mut reader = events.get_reader();
        let release_events: Vec<_> = reader.read(events).collect();

        assert_eq!(release_events.len(), 1);
        assert_eq!(release_events[0].source_planet, planet_entity);
    }

    #[test]
    fn test_spore_infection_on_neighbor() {
        let mut app = App::new();
        app.add_systems(Update, spore_infection_system);

        let source_planet = app.world_mut().spawn_empty().id();
        let target_planet = app.world_mut().spawn((
            Planet { gravity: 1.0 },
            Biome { invasive_flora: 0.0 },
        )).id();

        app.add_event::<SporeReleaseEvent>();
        app.world_mut().send_event(SporeReleaseEvent {
            source_planet,
            target_planet: Some(target_planet),
            spore_strength: 50.0,
        });

        app.update();

        let biome = app.world().get::<Biome>(target_planet).unwrap();
        assert_eq!(biome.invasive_flora, 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub gravity: f32,
}

#[derive(Component)]
pub struct FloraCultivation {
    pub amount: f32,
    pub is_gmo: bool,
}

#[derive(Component)]
pub struct Biome {
    pub invasive_flora: f32,
}

#[derive(Event)]
pub struct SporeReleaseEvent {
    pub source_planet: Entity,
    pub target_planet: Option<Entity>,
    pub spore_strength: f32,
}

pub fn spore_escape_system(
    query: Query<(Entity, &Planet, &FloraCultivation)>,
    mut event_writer: EventWriter<SporeReleaseEvent>,
) {
    for (entity, planet, cultivation) in query.iter() {
        if planet.gravity < 1.0 && cultivation.is_gmo && cultivation.amount > 500.0 {
            event_writer.send(SporeReleaseEvent {
                source_planet: entity,
                target_planet: None, // Logic for finding target happens elsewhere or next tick
                spore_strength: cultivation.amount * 0.05,
            });
        }
    }
}

pub fn spore_infection_system(
    mut events: EventReader<SporeReleaseEvent>,
    mut query: Query<&mut Biome>,
) {
    for event in events.read() {
        if let Some(target) = event.target_planet {
            if let Ok(mut biome) = query.get_mut(target) {
                biome.invasive_flora += event.spore_strength;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded values for gravity (`< 1.0`) and amount (`> 500.0`) should be moved to configurations or constants.
- **Performance**: Event iteration and target finding should be decoupled efficiently, potentially via a spatial query in Layer 2.
- **Design Improvements**: Introduce diplomatic penalty events if the `target_planet` belongs to another empire. Add a delay for stellar wind travel.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] GMO flora on low gravity planets emit spore events.
- [ ] Neighboring planets receive invasive flora increments upon spore infection.

## 7. Technical Guidance
- Place logic spanning layers carefully. The `spore_escape_system` is Layer 1, while tracking movement and `spore_infection_system` touches Layer 2/3. Consider an integration module.
- Use the existing event systems to bridge the gap between planetary scale (L1) and stellar scale (L2).

## 8. Questions
*Builder: add questions here if spec is unclear.*

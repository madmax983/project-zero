# 1075: The Memorial Economy

## 1. Overview
This specification details the "Memorial Economy" mechanic on Layer 1. Pops that die with high prestige or significant memories generate "Relics" or require "Memorial Structures." These structures boost the morale of descendants and related pops, but they consume valuable space and resources. Over time, the colony can begin exporting Relics to the broader system for massive cultural/diplomatic influence, creating an economy centered around the remembrance of tragic events.

## 2. Dependencies
- Layer 1 Pop death / lifecycle events
- Layer 1 Memory and Prestige tracking
- Resource and Building systems (Memorial Structures)
- Layer 2 export logistics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_prestige_death_generates_relic() {
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.init_resource::<Events<PopDiedEvent>>();
        app.add_systems(Update, process_pop_deaths_for_relics);

        let pop = app.world_mut().spawn((
            Pop,
            Prestige(100),
            Name::new("Hero"),
        )).id();

        app.world_mut().resource_mut::<Events<PopDiedEvent>>().send(PopDiedEvent { entity: pop });

        app.update();

        // Assert a Relic was generated
        let relics = app.world().query::<&Relic>().iter(app.world()).count();
        assert_eq!(relics, 1);
    }

    #[test]
    fn test_memorial_structure_boosts_descendant_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_memorial_morale_boost);

        let ancestor_id = Entity::from_raw(1); // Fake ID for testing
        let _memorial = app.world_mut().spawn((
            MemorialStructure { dedicated_to: ancestor_id },
        )).id();

        let descendant = app.world_mut().spawn((
            Pop,
            Morale(50),
            DescendantOf(ancestor_id),
        )).id();

        app.update();

        // Assert morale was boosted
        assert!(app.world().get::<Morale>(descendant).unwrap().0 > 50);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Prestige(pub u32);

#[derive(Component)]
pub struct Relic;

#[derive(Event)]
pub struct PopDiedEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct MemorialStructure {
    pub dedicated_to: Entity,
}

#[derive(Component)]
pub struct Morale(pub u32);

#[derive(Component)]
pub struct DescendantOf(pub Entity);

pub fn process_pop_deaths_for_relics(
    mut commands: Commands,
    mut events: EventReader<PopDiedEvent>,
    query: Query<&Prestige>,
) {
    for event in events.read() {
        if let Ok(prestige) = query.get(event.entity) {
            if prestige.0 >= 100 {
                commands.spawn(Relic);
            }
        }
    }
}

pub fn apply_memorial_morale_boost(
    memorials: Query<&MemorialStructure>,
    mut descendants: Query<(&mut Morale, &DescendantOf)>,
) {
    for memorial in memorials.iter() {
        for (mut morale, descendant_of) in descendants.iter_mut() {
            if descendant_of.0 == memorial.dedicated_to {
                morale.0 += 10;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Space Management**: Ensure building a `MemorialStructure` correctly consumes grid space and resources.
- **Exporting Relics**: Hook the `Relic` entities into the trade network, allowing them to be loaded onto ships for Layer 2 influence.
- **Morale Balance**: Balance the morale boost to ensure it's significant enough to justify the space trade-off without making the game too easy.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for the new code
- [ ] Specific feature behavior (relic generation, morale boosting) verified

## 7. Technical Guidance
- When exporting relics, ensure the original colony retains some long-term buff or debuff (e.g., selling off cultural heritage causes local unrest).
- Relics should be added to `lore/LEXICON.md` so the event generator can use them.

## 8. Questions
*Builder: add questions here if spec is unclear.*

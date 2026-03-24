# 571 Diplomatic Contraband

## 1. Overview
A cross-layer (3 -> 1) trade agreement feature where a Layer 3 empire pays exorbitant Credits to store sealed, un-scannable "Black Box" containers in the Layer 1 colony stockpiles. The player is forbidden to open them, but the boxes periodically emit strange localized effects (mutations, tech breakthroughs, radiation) on nearby Pops. This creates tension between massive economic injections and unpredictable, terrifying localized consequences.

## 2. Dependencies
- Layer 1 `Stockpile` and `Item` entities
- Layer 3 `DiplomaticAgreement` system
- Layer 1 `Pop` health/mutation/tech-boost systems
- Event/Chronicle system for localized strange effects

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_accept_black_box_agreement_grants_credits() {
        let mut app = App::new();
        // Setup ...
        app.world.resource_mut::<Credits>().amount = 1000;

        app.world.send_event(AcceptDiplomaticAgreementEvent {
            agreement_type: AgreementType::BlackBoxStorage { payout: 50000 },
            partner_id: Entity::from_raw(1),
        });

        app.update();

        assert_eq!(app.world.resource::<Credits>().amount, 51000);
    }

    #[test]
    fn test_black_box_spawns_in_stockpile() {
        let mut app = App::new();
        // Setup ...
        let stockpile_entity = app.world.spawn(Stockpile::new(10)).id();

        app.world.send_event(AcceptDiplomaticAgreementEvent {
            agreement_type: AgreementType::BlackBoxStorage { payout: 50000 },
            partner_id: Entity::from_raw(1),
        });

        app.update();

        let mut query = app.world.query::<&Stockpile>();
        let stockpile = query.get(&app.world, stockpile_entity).unwrap();
        assert!(stockpile.contains_item_type(ItemType::SealedBlackBox));
    }

    #[test]
    fn test_black_box_emits_strange_effect_on_nearby_pops() {
        let mut app = App::new();
        // Setup ...
        let box_pos = GridPosition { x: 5, y: 5 };
        app.world.spawn((ItemType::SealedBlackBox, box_pos, StrangeEmitter { radius: 2, chance: 1.0 }));

        let pop_entity = app.world.spawn((Pop, GridPosition { x: 6, y: 5 })).id();

        app.update();

        // Assert pop gained a mutation or radiation effect
        assert!(app.world.get::<Mutation>(pop_entity).is_some() || app.world.get::<RadiationSickness>(pop_entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation to pass the tests

pub enum AgreementType {
    BlackBoxStorage { payout: u32 },
    // ...
}

#[derive(Event)]
pub struct AcceptDiplomaticAgreementEvent {
    pub agreement_type: AgreementType,
    pub partner_id: Entity,
}

#[derive(Component)]
pub struct StrangeEmitter {
    pub radius: u32,
    pub chance: f32,
}

pub fn handle_black_box_agreement_system(
    mut events: EventReader<AcceptDiplomaticAgreementEvent>,
    mut credits: ResMut<Credits>,
    mut stockpiles: Query<(Entity, &mut Stockpile)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let AgreementType::BlackBoxStorage { payout } = event.agreement_type {
            credits.amount += payout;
            if let Some((_, mut stockpile)) = stockpiles.iter_mut().next() {
                stockpile.add(ItemType::SealedBlackBox);
                // In full implementation, spawn item entity with StrangeEmitter
            }
        }
    }
}

pub fn strange_emitter_system(
    emitters: Query<(&GridPosition, &StrangeEmitter)>,
    mut pops: Query<(Entity, &GridPosition, Option<&mut Mutation>), With<Pop>>,
    mut commands: Commands,
) {
    for (box_pos, emitter) in emitters.iter() {
        for (pop_entity, pop_pos, mut mutation) in pops.iter_mut() {
            if box_pos.distance(pop_pos) <= emitter.radius as f32 {
                // Simplified effect application
                commands.entity(pop_entity).insert(Mutation::UnknownBiologicalShift);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Code Smells**: `strange_emitter_system` currently applies hardcoded `Mutation::UnknownBiologicalShift`. It needs to randomly pick from a table of strange effects (Radiation, Tech Inspiration, etc.) using RNG.
- **Performance**: Iterating all Pops against all emitters might be slow if there are many Black Boxes. Use spatial queries or grid-based cell checking for nearby pops.
- **Integration**: Link the occurrence of a strange effect to the Chronicle system so the player gets a notification.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Accepting a Black Box agreement grants the specified credits and inserts the item into a stockpile.
- [ ] Black boxes emit localized effects onto pops within a specific radius.

## 7. Technical Guidance
- Ensure `StrangeEmitter` runs on a cooldown or timer rather than every tick to avoid instantly obliterating nearby Pops.
- Consider adding a `Sealed` component to prevent haulers from attempting to consume or process the Black Box.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

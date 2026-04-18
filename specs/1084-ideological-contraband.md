# 1084: Ideological Contraband

## 1. Overview
Trade goods carry "Cultural Tags". Importing "Luxury Silks" from an Aristocratic empire increases "Elitism" ethics in your colony. "Worker Boots" from a Communist bloc boost "Collectivism".
This creates tension between economic necessity (importing cheap food from a Hive Mind) and cultural contamination (your Pops start demanding you dissolve the government and join the Hive).

## 2. Dependencies
- Layer 1 Pop logic and needs.
- Pop ethics/beliefs system.
- Layer 1 Trading/Importing mechanics.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_importing_goods_applies_cultural_tags_to_colony() {
        let mut app = App::new();
        app.add_systems(Update, apply_cultural_contraband_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Ethics { collectivism: 0, elitism: 0 },
        )).id();

        app.add_event::<TradeImportEvent>();

        // Import worker boots carrying collectivist ideology
        app.world_mut().send_event(TradeImportEvent {
            item_name: "Worker Boots".to_string(),
            amount: 10,
            cultural_tag: Some(CulturalTag::Collectivism),
            potency: 5,
        });

        app.update();

        let ethics = app.world().get::<Ethics>(pop_entity).unwrap();
        // The collectivism stat should have increased due to the imported goods
        assert!(ethics.collectivism > 0);
    }

    #[test]
    fn test_importing_neutral_goods_does_not_affect_ethics() {
        let mut app = App::new();
        app.add_systems(Update, apply_cultural_contraband_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Ethics { collectivism: 0, elitism: 0 },
        )).id();

        app.add_event::<TradeImportEvent>();

        // Import basic neutral goods
        app.world_mut().send_event(TradeImportEvent {
            item_name: "Basic Rations".to_string(),
            amount: 100,
            cultural_tag: None,
            potency: 0,
        });

        app.update();

        let ethics = app.world().get::<Ethics>(pop_entity).unwrap();
        // The collectivism stat should remain unchanged
        assert_eq!(ethics.collectivism, 0);
        assert_eq!(ethics.elitism, 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Ethics {
    pub collectivism: i32,
    pub elitism: i32,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CulturalTag {
    Collectivism,
    Elitism,
    HiveMind,
}

#[derive(Event)]
pub struct TradeImportEvent {
    pub item_name: String,
    pub amount: i32,
    pub cultural_tag: Option<CulturalTag>,
    pub potency: i32,
}

pub fn apply_cultural_contraband_system(
    mut events: EventReader<TradeImportEvent>,
    mut pop_query: Query<&mut Ethics, With<Pop>>,
) {
    for event in events.read() {
        if let Some(tag) = &event.cultural_tag {
            // Apply ideological drift to all pops based on the imported goods
            // A more complex implementation would only affect pops that consume the item
            for mut ethics in pop_query.iter_mut() {
                let drift = event.potency;
                match tag {
                    CulturalTag::Collectivism => ethics.collectivism += drift,
                    CulturalTag::Elitism => ethics.elitism += drift,
                    CulturalTag::HiveMind => {
                        // Handle hive mind logic if applicable
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Applying ethics to *all* Pops instantly upon import is unrealistic. It should probably only affect Pops that actively interact with or consume the specific imported item.
- **Performance**: Iterating through every Pop on every trade event might scale poorly in massive colonies. Consider applying the `CulturalTag` to the items in the stockpile instead, and having Pops read the tags when they fetch/consume items.
- **Design Improvements**: Add a `ChronicleEvent` when a colony's primary ethic flips due to trade imports, representing a bloodless cultural revolution. Introduce "Customs Inspections" policies that reduce trade efficiency but block ideological contraband.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `TradeImportEvent` correctly parses `CulturalTag`
- [ ] Pops' `Ethics` values shift when culturally tagged items are imported

## 7. Technical Guidance
- Integrate `TradeImportEvent` generation with the Layer 1 trading post or Layer 2 spaceport systems.
- Consider moving the `Ethics` mutation logic into the `metabolism_system` or `needs_system` where Pops actually consume goods, rather than doing it at the moment of import.

## 8. Questions
*Builder: add questions here if spec is unclear.*

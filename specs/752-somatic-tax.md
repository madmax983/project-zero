# 752 - The Somatic Tax

## 1. Overview
**Layer:** 1
**Fantasy:** Your colonists physically alter themselves to pay off their societal debts, slowly losing their humanity to efficiency.
**Mechanic:** Pops with high debt or low productivity are offered "Somatic Relief"—subsidized cybernetic or biological augmentations that increase work output but permanently reduce their Need capacity for Social and Leisure. They literally work more and care less.

## 2. Dependencies
- `016-utility-ai-system.md` (Utility AI for work action)
- `130-social-debt.md` (Debt tracking)
- `005-pop-needs.md` (Pop needs - social/leisure)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::{Needs, NeedType};
    use crate::layer1::social::debt::SocialDebt;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_somatic_tax_offered() {
        let mut app = App::new();
        app.add_plugins(SomaticTaxPlugin);

        let pop_id = app.world.spawn((
            Pop::new("Debt Worker"),
            SocialDebt { amount: 1500.0 }, // high debt
            Needs::default(),
        )).id();

        app.update();

        let pop = app.world.get_entity(pop_id).unwrap();
        assert!(pop.contains::<SomaticReliefOffer>());
    }

    #[test]
    fn test_somatic_tax_accepted() {
        let mut app = App::new();
        app.add_plugins(SomaticTaxPlugin);

        let pop_id = app.world.spawn((
            Pop::new("Accepting Worker"),
            SocialDebt { amount: 1500.0 },
            Needs::default(),
            SomaticReliefOffer { accepted: true },
        )).id();

        app.update();

        let pop = app.world.get_entity(pop_id).unwrap();
        let needs = pop.get::<Needs>().unwrap();
        let debt = pop.get::<SocialDebt>().unwrap();

        assert!(pop.contains::<SomaticAugmentation>());
        assert!(needs.get_max(NeedType::Social) < 1.0); // max social reduced
        assert!(needs.get_max(NeedType::Leisure) < 1.0); // max leisure reduced
        assert!(debt.amount < 1500.0); // Debt reduced by relief
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::{Needs, NeedType};
use crate::layer1::social::debt::SocialDebt;
use crate::layer1::pop::Pop;

pub struct SomaticTaxPlugin;

impl Plugin for SomaticTaxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (offer_somatic_relief_system, process_somatic_relief_system));
    }
}

#[derive(Component)]
pub struct SomaticReliefOffer {
    pub accepted: bool,
}

#[derive(Component)]
pub struct SomaticAugmentation;

pub fn offer_somatic_relief_system(
    mut commands: Commands,
    query: Query<(Entity, &SocialDebt), Without<SomaticReliefOffer>>,
) {
    for (entity, debt) in query.iter() {
        if debt.amount >= 1000.0 {
            commands.entity(entity).insert(SomaticReliefOffer { accepted: false });
        }
    }
}

pub fn process_somatic_relief_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Needs, &mut SocialDebt, &SomaticReliefOffer)>,
) {
    for (entity, mut needs, mut debt, offer) in query.iter_mut() {
        if offer.accepted {
            commands.entity(entity).insert(SomaticAugmentation);
            commands.entity(entity).remove::<SomaticReliefOffer>();

            // Reduce max needs permanently
            needs.set_max(NeedType::Social, 0.5);
            needs.set_max(NeedType::Leisure, 0.5);

            // Subsidize debt
            debt.amount -= 1000.0;
            if debt.amount < 0.0 {
                debt.amount = 0.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - The acceptance of the offer should probably be tied into the Utility AI, where a Pop evaluates the "Accept Relief" action based on how much the debt is stressing them.
    - Instead of hardcoding `0.5`, the `SomaticAugmentation` component could hold the reduction modifier, allowing for different tiers of augmentation.
    - Integration with Chronicles: An event should fire when a Pop accepts the Somatic Tax, adding to the colony's lore.

## 6. Acceptance Criteria (Testable!)
- [ ] `test_somatic_tax_offered` passes.
- [ ] `test_somatic_tax_accepted` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- **Code structure**: Add to `src/layer1/economy/` or `src/layer1/social/` next to debt.
- **Gotchas**: Ensure that reducing the `max` value of a Need also caps the `current` value of that Need if it's currently above the new max.

## 8. Questions
*Builder: add questions here if spec is unclear.*

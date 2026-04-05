# Specification: The Bureaucratic Black Market (Layer 1)

## 1. Overview
The "Bureaucratic Black Market" introduces a shadow economy to the colony. When the official "Admin" resource becomes scarce, Pops possessing specific traits ('Intelligent', 'Deceitful') can forge "Priority Tokens." These tokens allow purchasing Pops to forcefully elevate the utility scores of their own needs and work tasks above those of the collective. This creates a parasitic drag on the colony's overall efficiency, simulating corruption while potentially ensuring the survival or productivity of key individuals at the cost of the many.

## 2. Dependencies
- `016-utility-ai-system.md`: The base Utility AI must exist, as Priority Tokens will directly modify utility scoring weights.
- `005-pop-needs.md`: Pops must have needs that can be prioritized.
- Trait System: Pops must have traits (e.g., 'Intelligent', 'Deceitful') to determine if they can forge tokens.
- Inventory/Resource System: A way to track "Admin" resource scarcity and store "Priority Tokens".

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_forge_priority_token_when_admin_scarce() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<AdminResource>();
        app.add_systems(Update, bureaucratic_black_market_system);

        // Simulate severe Admin scarcity
        app.world_mut().resource_mut::<AdminResource>().amount = 0;

        let forger = app.world_mut().spawn((
            Pop,
            Traits(vec![Trait::Deceitful]),
            Inventory::default(),
        )).id();

        // Act
        app.update();

        // Assert: The deceitful pop should have forged a Priority Token
        let inventory = app.world().get::<Inventory>(forger).unwrap();
        assert!(inventory.contains(&Item::PriorityToken), "Deceitful pop should forge tokens when Admin is scarce");
    }

    #[test]
    fn test_priority_token_inflates_utility_score() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_priority_tokens_system);

        let elite_pop = app.world_mut().spawn((
            Pop,
            UtilityContext::default(),
            Inventory(vec![Item::PriorityToken]),
            Needs { food: 50.0, ..default() },
        )).id();

        let common_pop = app.world_mut().spawn((
            Pop,
            UtilityContext::default(),
            Inventory::default(),
            Needs { food: 50.0, ..default() },
        )).id();

        // Act
        app.update();

        // Assert: The pop with the token should have an artificially inflated utility score for food
        let elite_ctx = app.world().get::<UtilityContext>(elite_pop).unwrap();
        let common_ctx = app.world().get::<UtilityContext>(common_pop).unwrap();

        assert!(
            elite_ctx.get_weight(ActionType::Eat) > common_ctx.get_weight(ActionType::Eat),
            "Priority token must significantly increase the utility weight for personal needs"
        );
    }

    #[test]
    fn test_token_usage_consumes_token_and_adds_system_drag() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<AdminResource>();
        app.add_systems(Update, consume_priority_tokens_system);

        let elite_pop = app.world_mut().spawn((
            Pop,
            ActiveAction(ActionType::Eat),
            Inventory(vec![Item::PriorityToken]),
        )).id();

        let initial_drag = app.world().resource::<AdminResource>().administrative_drag;

        // Act
        app.update();

        // Assert
        let inventory = app.world().get::<Inventory>(elite_pop).unwrap();
        let current_drag = app.world().resource::<AdminResource>().administrative_drag;

        assert!(!inventory.contains(&Item::PriorityToken), "Token should be consumed upon completing a prioritized action");
        assert!(current_drag > initial_drag, "Using a forged token must increase overall administrative drag");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct AdminResource {
    pub amount: u32,
    pub administrative_drag: f32,
}

#[derive(Component)]
pub struct Traits(pub Vec<Trait>);

#[derive(PartialEq, Eq)]
pub enum Trait {
    Deceitful,
    Intelligent,
    // ...
}

#[derive(Component, Default)]
pub struct Inventory(pub Vec<Item>);

impl Inventory {
    pub fn contains(&self, item: &Item) -> bool {
        self.0.contains(item)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Item {
    PriorityToken,
    // ...
}

#[derive(Component)]
pub struct UtilityContext {
    pub weights: bevy::utils::HashMap<ActionType, f32>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum ActionType {
    Eat,
    // ...
}

// ... other necessary components for minimal pass

pub fn bureaucratic_black_market_system(
    admin: Res<AdminResource>,
    mut query: Query<(&Traits, &mut Inventory), With<Pop>>,
) {
    if admin.amount == 0 {
        for (traits, mut inventory) in query.iter_mut() {
            if traits.0.contains(&Trait::Deceitful) {
                inventory.0.push(Item::PriorityToken);
            }
        }
    }
}

pub fn apply_priority_tokens_system(
    mut query: Query<(&Inventory, &mut UtilityContext)>,
) {
    for (inventory, mut ctx) in query.iter_mut() {
        if inventory.contains(&Item::PriorityToken) {
            if let Some(weight) = ctx.weights.get_mut(&ActionType::Eat) {
                *weight += 100.0; // Minimal implementation of score inflation
            }
        }
    }
}

pub fn consume_priority_tokens_system(
    mut admin: ResMut<AdminResource>,
    mut query: Query<(&ActiveAction, &mut Inventory)>,
) {
    for (_, mut inventory) in query.iter_mut() {
        if let Some(pos) = inventory.0.iter().position(|i| *i == Item::PriorityToken) {
            inventory.0.remove(pos);
            admin.administrative_drag += 10.0; // Minimal implementation of adding drag
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The `apply_priority_tokens_system` currently hardcodes the `ActionType::Eat`. This needs to be generalized to iterate through whatever actions are relevant to the pop's current needs or assigned tasks.
- **Code Smells**: Avoid using bare vectors for `Inventory` if a map of item counts is more efficient for larger item varieties. The Utility AI weight modification shouldn't directly overwrite or crudely add numbers; consider a modifier struct that the Utility AI reads when calculating final scores.
- **Performance**: Polling `admin.amount` every tick for every deceitful pop is inefficient. Consider using an event-driven approach where an `AdminShortageEvent` triggers the forgery attempt, or polling less frequently via a `Timer`.
- **API Improvements**: Create a dedicated `AdminState` resource or system set to manage the lifecycle of administrative drag and tokens.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with 'Deceitful' or 'Intelligent' traits forge Priority Tokens when Admin resource is low.
- [ ] Priority Tokens correctly elevate the utility score for a Pop's actions above normal bounds.
- [ ] Consuming a Priority Token increases colony-wide `administrative_drag`.

## 7. Technical Guidance
- **Code Structure**: Place this within `src/layer1/black_market.rs` or `src/layer1/bureaucracy.rs`.
- **Integration Points**: Needs tight integration with `evaluate_actions_system` in `utility_ai.rs` to ensure the token's weight modifier is applied correctly without permanently breaking the pop's base utility weights.
- **Gotchas**: Ensure that `administrative_drag` actually has a consequence elsewhere in the system (e.g., slowing down official task assignment or reducing resource distribution efficiency). If it doesn't, the tension mechanic fails.

## 8. Questions
*Builder: add questions here if spec is unclear.*

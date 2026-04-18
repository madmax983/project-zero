# Spec 1093: Superstitious Totems

## 1. Overview
In the face of the void, colonists cling to small comforts. Pops spontaneously create "Totems" (e.g., lucky rocks, carved scraps) which they carry. Possessing a totem grants a minor stress reduction buff. Losing the totem causes a massive stress spike ("Bad Omen").

**Fantasy:** In the face of the void, people cling to small comforts.

## 2. Dependencies
- Layer 1 Population (`Pop`, `Stress`, `Inventory`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Stress, Inventory, PopInventory};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            totem_creation_system,
            totem_effect_system,
            totem_loss_system,
        ));
        app
    }

    #[test]
    fn test_pop_creates_totem_under_stress() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Stress { value: 80.0 }, // High stress
            PopInventory { items: vec![] },
        )).id();

        app.update();

        let inventory = app.world().get::<PopInventory>(pop).unwrap();
        assert!(inventory.items.contains(&ItemType::Totem), "Stressed pop should create a totem");
    }

    #[test]
    fn test_totem_reduces_stress() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Stress { value: 50.0 },
            PopInventory { items: vec![ItemType::Totem] },
            TotemBuff { active: false },
        )).id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.value < 50.0, "Possessing a totem should reduce stress over time");
    }

    #[test]
    fn test_losing_totem_causes_bad_omen() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop::default(),
            Stress { value: 50.0 },
            PopInventory { items: vec![] }, // Lost the totem!
            HadTotem { previously: true },
        )).id();

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.value > 80.0, "Losing a totem should cause a massive stress spike");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/totems.rs`.
- Define `ItemType::Totem`.
- Define `TotemBuff` and `HadTotem` components.
- Implement `totem_creation_system`:
  - Query pops with high `Stress` and no `Totem` in `PopInventory`.
  - Probabilistically add a `Totem` to their inventory and add the `HadTotem` component.
- Implement `totem_effect_system`:
  - Query pops with a `Totem` in inventory.
  - Apply a small negative modifier to `Stress` growth.
- Implement `totem_loss_system`:
  - Query pops with `HadTotem` but NO `Totem` in inventory.
  - Apply a large immediate penalty to `Stress` (Bad Omen effect) and remove `HadTotem` to prevent continuous spiking.

## 5. REFACTOR Phase: Quality & Design
- **Totem Types:** Expand beyond a generic "Totem" to specific items if the inventory system supports named/unique items.
- **Inventory Space:** Totems should consume a physical inventory slot, forcing a tradeoff between carrying the totem and carrying tools/resources.

## 6. Acceptance Criteria
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops create, benefit from, and suffer from losing totems.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Integrate with the existing `Inventory` or `PopInventory` system (adjust struct names as needed to match the codebase).
- Ensure the stress reduction is balanced against other stress relief mechanics (like leisure facilities).

## 8. Questions
*Builder: Add questions here if inventory management for unique/un-droppable items requires extending the inventory system.*

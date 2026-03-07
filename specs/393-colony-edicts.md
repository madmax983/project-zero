# 393 - Colony Edicts

## 1. Overview
The Colony Edicts feature introduces a menu of global toggles that modify the behavior of all Pops and systems in Layer 1. Players can enact policies like "Double Rations" to boost mood at the cost of food, or "Martial Law" to suppress unrest while lowering liberty. This adds a macro-management layer, forcing the player to balance efficiency with morale.

## 2. Dependencies
- `031-pop-morale.md` (Mood and Unrest)
- `005-pop-needs.md` (Hunger and other needs)
- `054-colony-edicts.md` (This is replacing or fully implementing the concepts described originally)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{Needs, Metabolism};
    use crate::layer1::pop::Mood;

    #[test]
    fn test_edict_double_rations_active() {
        let mut app = App::new();

        let mut edicts = ActiveEdicts::default();
        edicts.enable(EdictType::DoubleRations);
        app.insert_resource(edicts);

        app.add_systems(Update, apply_edict_modifiers);

        let pop = app.world_mut().spawn((
            Needs::default(),
            Mood::default(),
            Metabolism { hunger_rate: 1.0, ..default() },
        )).id();

        app.update();

        // Double rations should increase mood and increase hunger drain
        let mood = app.world().get::<Mood>(pop).unwrap();
        let metabolism = app.world().get::<Metabolism>(pop).unwrap();

        assert!(mood.current > 50.0, "Double Rations should boost mood");
        assert!(metabolism.hunger_rate > 1.0, "Double Rations should double food consumption rate");
    }

    #[test]
    fn test_edict_martial_law_suppresses_unrest() {
        let mut app = App::new();

        let mut edicts = ActiveEdicts::default();
        edicts.enable(EdictType::MartialLaw);
        app.insert_resource(edicts);

        app.add_systems(Update, apply_edict_modifiers);

        let pop = app.world_mut().spawn((
            Mood { current: 10.0, ..default() }, // Very low mood, should cause unrest
            Unrest(50.0),
            Liberty(100.0),
        )).id();

        app.update();

        // Martial Law suppresses unrest but lowers liberty
        let unrest = app.world().get::<Unrest>(pop).unwrap();
        let liberty = app.world().get::<Liberty>(pop).unwrap();

        assert_eq!(unrest.0, 0.0, "Martial Law should reduce Unrest to 0");
        assert!(liberty.0 < 100.0, "Martial Law should decrease Liberty");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EdictType {
    DoubleRations,
    MartialLaw,
    NightShifts,
}

#[derive(Resource, Default)]
pub struct ActiveEdicts(HashSet<EdictType>);

impl ActiveEdicts {
    pub fn enable(&mut self, edict: EdictType) {
        self.0.insert(edict);
    }
    pub fn disable(&mut self, edict: &EdictType) {
        self.0.remove(edict);
    }
    pub fn is_active(&self, edict: &EdictType) -> bool {
        self.0.contains(edict)
    }
}

// Components affected by Edicts
#[derive(Component, Default)]
pub struct Mood { pub current: f32 }
#[derive(Component)]
pub struct Metabolism { pub hunger_rate: f32 }
#[derive(Component)]
pub struct Unrest(pub f32);
#[derive(Component)]
pub struct Liberty(pub f32);

pub fn apply_edict_modifiers(
    edicts: Res<ActiveEdicts>,
    mut query: Query<(&mut Mood, &mut Metabolism, Option<&mut Unrest>, Option<&mut Liberty>)>,
) {
    let double_rations = edicts.is_active(&EdictType::DoubleRations);
    let martial_law = edicts.is_active(&EdictType::MartialLaw);

    for (mut mood, mut metabolism, mut opt_unrest, mut opt_liberty) in query.iter_mut() {
        // Reset base rates (normally handled by a separate system, simplified here)
        metabolism.hunger_rate = 1.0;

        if double_rations {
            mood.current = (mood.current + 10.0).clamp(0.0, 100.0);
            metabolism.hunger_rate = 2.0;
        }

        if martial_law {
            if let Some(mut unrest) = opt_unrest {
                unrest.0 = 0.0;
            }
            if let Some(mut liberty) = opt_liberty {
                liberty.0 = (liberty.0 - 20.0).clamp(0.0, 100.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Modifier System**: `apply_edict_modifiers` currently hardcodes the effects. Use a generic modifier system (e.g., `StatModifiers` component) where Edicts attach specific buffs/debuffs that are calculated dynamically.
- **Edict Costs**: Some edicts should have an upfront or ticking cost (e.g., Political Influence, Admin points).
- **Duration/Cooldowns**: Edicts should have minimum active durations to prevent players from toggling them frame-by-frame.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 85% for `layer1/edicts.rs`
- [ ] Active Edicts correctly apply their global modifiers to Pop entities.

## 7. Technical Guidance
- Create a new module `layer1/edicts.rs`.
- `ActiveEdicts` is a global `Resource`.
- Ensure modifiers are applied cleanly without accumulating permanently (e.g., store base stats and compute current stats each frame, or use temporary modifiers).

## 8. Questions
*Builder: add questions here if spec is unclear.*

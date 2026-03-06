# Spec 357: Graffiti & Signage

## 1. Overview
The colony walls speak! Pops leave "Markings" (graffiti, posters, notes) on buildings based on their mood or beliefs. Other pops react to these markings, creating a micro-environment of subversion or communication that players must manage.

## 2. Dependencies
- `004-pop-entity.md` (for Pop data structure)
- `005-pop-needs.md` (for Mood / Needs)
- `006-building-placement.md` (for Buildings as surfaces)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::Building;
    use crate::layer1::needs::{Needs, MoodModifier};
    use crate::layer1::grid::GridPosition;

    #[test]
    fn test_pop_places_graffiti_when_stressed() {
        let mut app = App::new();
        app.add_systems(Update, place_graffiti_system);

        // Stressed pop
        app.world_mut().spawn((
            Pop,
            Needs { stress: 80.0, ..default() },
            GridPosition { x: 5, y: 5 },
        ));

        // Building at same location
        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        // Building should now have a Graffiti component
        assert!(app.world().get::<Graffiti>(building).is_some());
    }

    #[test]
    fn test_graffiti_affects_nearby_pops() {
        let mut app = App::new();
        app.add_systems(Update, process_graffiti_effects_system);

        // Building with "Angry" Graffiti
        app.world_mut().spawn((
            Building { building_type: BuildingType::Wall },
            Graffiti { message_type: GraffitiType::Subversive, intensity: 10.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Pop nearby
        let pop = app.world_mut().spawn((
            Pop,
            Needs { stress: 10.0, ..default() },
            GridPosition { x: 5, y: 6 },
        )).id();

        app.update();

        // Pop's stress should have increased due to subversive graffiti
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.stress > 10.0, "Pop stress was not increased by subversive graffiti");
    }

    #[test]
    fn test_clean_graffiti_action_removes_it() {
        let mut app = App::new();
        app.add_systems(Update, clean_graffiti_system);

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Wall },
            Graffiti { message_type: GraffitiType::Subversive, intensity: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Simulate a cleaner pop doing the "CleanGraffiti" action
        app.world_mut().spawn((
            Pop,
            ActiveAction::CleanGraffiti(building),
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        // Graffiti component should be removed
        assert!(app.world().get::<Graffiti>(building).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::building::Building;
use crate::layer1::grid::GridPosition;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq)]
pub enum GraffitiType {
    Subversive,
    Inspiring,
}

#[derive(Component)]
pub struct Graffiti {
    pub message_type: GraffitiType,
    pub intensity: f32,
}

#[derive(Component)]
pub enum ActiveAction {
    CleanGraffiti(Entity),
    Idle,
}

pub fn place_graffiti_system(
    mut commands: Commands,
    pop_query: Query<(&Needs, &GridPosition), With<Pop>>,
    mut building_query: Query<(Entity, &GridPosition), (With<Building>, Without<Graffiti>)>,
) {
    for (needs, pop_pos) in pop_query.iter() {
        if needs.stress > 75.0 {
            // Find an adjacent or overlapping building to tag
            for (b_ent, b_pos) in building_query.iter_mut() {
                if b_pos.x == pop_pos.x && b_pos.y == pop_pos.y {
                    commands.entity(b_ent).insert(Graffiti {
                        message_type: GraffitiType::Subversive,
                        intensity: 10.0,
                    });
                    break;
                }
            }
        }
    }
}

pub fn process_graffiti_effects_system(
    graffiti_query: Query<(&Graffiti, &GridPosition)>,
    mut pop_query: Query<(&mut Needs, &GridPosition), With<Pop>>,
) {
    for (graffiti, g_pos) in graffiti_query.iter() {
        for (mut needs, p_pos) in pop_query.iter_mut() {
            let dist = ((g_pos.x - p_pos.x).pow(2) + (g_pos.y - p_pos.y).pow(2)) as f32;
            if dist <= 2.0 { // Nearby
                if graffiti.message_type == GraffitiType::Subversive {
                    needs.stress += graffiti.intensity * 0.1;
                }
            }
        }
    }
}

pub fn clean_graffiti_system(
    mut commands: Commands,
    pop_query: Query<&ActiveAction, With<Pop>>,
) {
    for action in pop_query.iter() {
        if let ActiveAction::CleanGraffiti(target) = action {
            commands.entity(*target).remove::<Graffiti>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Cooldowns:** Ensure Pops don't spam graffiti every tick by giving them a `GraffitiCooldown`.
- **Variety:** Base `GraffitiType` on Pop traits (e.g., an `Optimist` pop might leave `Inspiring` messages when happy).
- **Spatial Hash:** The `process_graffiti_effects_system` uses an `O(N*M)` query which is slow. Refactor to use a spatial grid or `KD-Tree` to find nearby Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Stressed Pops will place `Graffiti` on overlapping `Building` entities if none exists.
- [ ] `Graffiti` increases the stress of nearby Pops.
- [ ] The `CleanGraffiti` action successfully removes the `Graffiti` component from a target.

## 7. Technical Guidance
- Integrate with the existing `Needs` and `Building` components.
- Make sure `place_graffiti_system` is correctly registered in `Layer1SystemSet::Execution`.

## 8. Questions
*Builder: add questions here if spec is unclear.*

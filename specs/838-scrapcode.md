# Scrapcode

## 1. Overview
**Layer:** 1
**Fantasy:** A virus that attacks the very concept of a chair.
**Mechanic:** A digital infection in the colony database. It randomly corrupts Building Recipes (e.g., a Bed now costs 50 Steel instead of 10 Wood). Must be "Purged" by Scientists, but purging takes the database offline (no building).
**Emergence:** You need to build turrets *now* to stop a raid, but the Scrapcode has changed the recipe to require "Gold". You don't have Gold. You have to fight with pistols.
**Tension:** Digital security upkeep vs. Production uptime.

## 2. Dependencies
- Building Recipe system.
- Database/Technology network logic.
- Colony Pops (Scientists) and Job assignment.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use std::collections::HashMap;

    #[derive(Component, Clone)]
    struct Recipe { required_materials: HashMap<String, u32> }

    #[derive(Resource)]
    struct DatabaseState { is_offline: bool }

    #[derive(Component)]
    struct ScrapcodeInfection { active: bool, corruption_severity: f32 }

    #[test]
    fn test_scrapcode_corrupts_recipes() {
        let mut app = App::new();
        app.add_systems(Update, apply_scrapcode_corruption);

        let mut original_recipe = HashMap::new();
        original_recipe.insert("Wood".to_string(), 10);

        let recipe_ent = app.world_mut().spawn(Recipe { required_materials: original_recipe.clone() }).id();
        app.world_mut().spawn(ScrapcodeInfection { active: true, corruption_severity: 1.0 });

        app.update();

        let corrupted = app.world().get::<Recipe>(recipe_ent).unwrap();
        assert_ne!(corrupted.required_materials, original_recipe, "Recipe materials should be altered by scrapcode.");
    }

    #[test]
    fn test_purging_scrapcode_takes_database_offline() {
        let mut app = App::new();
        app.insert_resource(DatabaseState { is_offline: false });
        app.add_systems(Update, process_scrapcode_purge);

        let infection = app.world_mut().spawn(ScrapcodeInfection { active: true, corruption_severity: 1.0 }).id();

        app.world_mut().resource_mut::<Events<PurgeIntent>>().send(PurgeIntent { target: infection });
        app.update();

        let db_state = app.world().resource::<DatabaseState>();
        assert!(db_state.is_offline, "Database must be offline during a purge.");
        let purged_infection = app.world().get::<ScrapcodeInfection>(infection).unwrap();
        assert!(!purged_infection.active, "Scrapcode should be deactivated after purge.");
    }

    #[test]
    fn test_building_fails_when_database_offline() {
        let mut app = App::new();
        app.insert_resource(DatabaseState { is_offline: true });
        app.add_systems(Update, attempt_build_system);

        let build_intent = app.world_mut().spawn(BuildIntent { recipe_id: Entity::PLACEHOLDER }).id();

        app.update();

        let intent = app.world().get::<BuildIntent>(build_intent).unwrap();
        assert!(intent.failed, "Build attempt must fail if database is offline.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Event)]
pub struct PurgeIntent {
    pub target: Entity,
}

#[derive(Component)]
pub struct BuildIntent {
    pub recipe_id: Entity,
    pub failed: bool,
}

pub fn apply_scrapcode_corruption(
    mut recipes: Query<&mut Recipe>,
    infections: Query<&ScrapcodeInfection>,
) {
    if infections.iter().any(|i| i.active) {
        for mut recipe in recipes.iter_mut() {
            // Minimal: change the requirement string to random scrap.
            recipe.required_materials.clear();
            recipe.required_materials.insert("Gold".to_string(), 50);
        }
    }
}

pub fn process_scrapcode_purge(
    mut db_state: ResMut<DatabaseState>,
    mut intents: EventReader<PurgeIntent>,
    mut infections: Query<&mut ScrapcodeInfection>,
) {
    for intent in intents.read() {
        if let Ok(mut infection) = infections.get_mut(intent.target) {
            infection.active = false;
            db_state.is_offline = true;
        }
    }
}

pub fn attempt_build_system(
    db_state: Res<DatabaseState>,
    mut builds: Query<&mut BuildIntent>,
) {
    if db_state.is_offline {
        for mut build in builds.iter_mut() {
            build.failed = true;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Corruption Logic:** A hardcoded "Gold: 50" overwrite is naive. Refactor to inject a `ScrapcodeModifier` component onto recipes instead of mutating the base struct. This allows reverting to the original recipe after purging without permanently destroying the database.
- **Purge Duration:** The purge shouldn't happen instantly. Add a `PurgeProgress` component managed by Scientist pops, tracking completion over time. The database remains offline until the progress bar completes.
- **Resource Typos:** The infection could substitute actual game resources (like replacing `ItemType::Wood` with `ItemType::Plutonium`) by referencing a random enum variant.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Scrapcode successfully alters recipe material requirements when active.
- [ ] Initiating a purge forces the database offline.
- [ ] While the database is offline, construction requests are rejected.

## 7. Technical Guidance
- Decouple the base recipe from the computed recipe. Systems should read from `ComputedRecipe` which applies all modifications (like scrapcode or efficiency tech) on top of the `BaseRecipe`.
- Consider emitting `RecipeCorruptedEvent` so UI panels can glitch out visually and warn the player.

## 8. Questions
*Builder: add questions here if spec is unclear.*

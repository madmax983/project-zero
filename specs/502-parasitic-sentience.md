# 502 - Parasitic Sentience

## 1. Overview
Smart-tools that provide massive skill boosts slowly overwrite the user's personality with ancient programming. Players can equip Pops with "Precursor Tools" that grant instant Level 10 proficiency in jobs. However, prolonged use slowly replaces the Pop's traits with uniform "Drone" traits, erasing their individuality and relationships, and eventually causing them to refuse any task except the one the tool was designed for, effectively rendering them dead to society.

## 2. Dependencies
- `051` Pop Skills and Experience
- `058` Personal Tools
- `084` Pop Traits
- `047` Pop Relationships

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Assuming existence of these structures
    // use crate::layer1::pop::{Pop, Trait, Skills};
    // use crate::layer1::equipment::{Equipment, ToolType};
    // use crate::layer1::relationships::Relationships;

    #[derive(Component)]
    struct EquippedTool {
        pub is_precursor: bool,
        pub skill_boost: i32,
    }

    #[derive(Component)]
    struct ParasiticInfection {
        pub progress: f32, // 0.0 to 100.0
    }

    #[derive(Component, Clone, PartialEq, Debug)]
    struct Traits {
        pub list: Vec<String>,
    }

    #[derive(Component)]
    struct Relationships {
        pub connections: usize,
    }

    fn process_parasitic_tools_system(
        mut commands: Commands,
        mut query: Query<(Entity, &EquippedTool, Option<&mut ParasiticInfection>, &mut Traits, &mut Relationships)>,
    ) {
        for (entity, tool, infection_opt, mut traits, mut relationships) in query.iter_mut() {
            if tool.is_precursor {
                let mut progress = 0.0;
                if let Some(mut infection) = infection_opt {
                    infection.progress += 1.0;
                    progress = infection.progress;
                } else {
                    commands.entity(entity).insert(ParasiticInfection { progress: 1.0 });
                    progress = 1.0;
                }

                if progress >= 50.0 {
                    relationships.connections = 0; // Sever relationships
                }

                if progress >= 100.0 {
                    traits.list.clear();
                    traits.list.push("Drone".to_string());
                }
            }
        }
    }

    #[test]
    fn test_precursor_tool_causes_infection() {
        let mut app = App::new();

        let pop = app.world_mut().spawn((
            EquippedTool { is_precursor: true, skill_boost: 10 },
            Traits { list: vec!["Cheerful".to_string()] },
            Relationships { connections: 5 },
        )).id();

        app.add_systems(Update, process_parasitic_tools_system);
        app.update();

        assert!(app.world().entity(pop).contains::<ParasiticInfection>(), "Pop should gain infection from precursor tool");
    }

    #[test]
    fn test_infection_severs_relationships() {
        let mut app = App::new();

        let pop = app.world_mut().spawn((
            EquippedTool { is_precursor: true, skill_boost: 10 },
            ParasiticInfection { progress: 49.0 },
            Traits { list: vec!["Cheerful".to_string()] },
            Relationships { connections: 5 },
        )).id();

        app.add_systems(Update, process_parasitic_tools_system);
        app.update();

        let rel = app.world().get::<Relationships>(pop).unwrap();
        assert_eq!(rel.connections, 0, "At 50% progress, relationships should be severed");
    }

    #[test]
    fn test_full_infection_replaces_traits() {
        let mut app = App::new();

        let pop = app.world_mut().spawn((
            EquippedTool { is_precursor: true, skill_boost: 10 },
            ParasiticInfection { progress: 99.0 },
            Traits { list: vec!["Cheerful".to_string()] },
            Relationships { connections: 0 },
        )).id();

        app.add_systems(Update, process_parasitic_tools_system);
        app.update();

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert_eq!(traits.list, vec!["Drone".to_string()], "At 100% progress, traits should be replaced by 'Drone'");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Placeholders for external module components
#[derive(Component)]
pub struct EquippedTool {
    pub is_precursor: bool,
    pub skill_boost: i32,
    pub target_job: String, // e.g., "Medical"
}

#[derive(Component)]
pub struct ParasiticInfection {
    pub progress: f32, // 0.0 to 100.0
}

#[derive(Component, Clone)]
pub struct Traits {
    pub list: Vec<String>,
}

#[derive(Component)]
pub struct Relationships {
    pub connections: usize, // Simplified representation
}

pub fn process_parasitic_tools_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &EquippedTool,
        Option<&mut ParasiticInfection>,
        &mut Traits,
        &mut Relationships,
    )>,
) {
    for (entity, tool, infection_opt, mut traits, mut relationships) in query.iter_mut() {
        if tool.is_precursor {
            let mut current_progress = 0.0;

            if let Some(mut infection) = infection_opt {
                infection.progress += 0.5; // Arbitrary decay rate
                current_progress = infection.progress;
            } else {
                commands.entity(entity).insert(ParasiticInfection { progress: 0.5 });
                current_progress = 0.5;
            }

            // Milestone 1: Isolation
            if current_progress >= 50.0 {
                relationships.connections = 0; // Effectively clears their social graph
            }

            // Milestone 2: Erasure
            if current_progress >= 100.0 {
                traits.list.clear();
                traits.list.push("Drone".to_string());

                // At this point, the Pop would also refuse any job other than tool.target_job
                // This would be enforced in the Utility AI logic.
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Relationship Cleanup**: Instead of just setting `connections` to 0, properly iterate through the colony's relationship graph and sever bilateral ties, potentially applying a `Grief` or `Confusion` mood modifier to the Pop's former friends.
- **Utility AI Override**: Once the `Drone` trait is acquired, inject a rigid filter into the Pop's Utility AI that rejects all `ScorableCandidates` that do not match the `target_job` of the Precursor Tool.
- **Curing**: Consider whether removing the tool pauses the infection or slowly reverses it. For maximum tension, the infection should be permanent once it reaches 100%, even if the tool is removed.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Equipping a Precursor Tool starts a `ParasiticInfection` progression.
- [ ] At 50% infection, the Pop's social relationships are severed.
- [ ] At 100% infection, the Pop's personality traits are overwritten by a "Drone" trait.

## 7. Technical Guidance
- The `ParasiticInfection` state should be exposed to the UI so the player can see their colonist slowly degrading and make a choice to remove the tool before the point of no return.
- Hook into the existing `051 Pop Skills and Experience` system so the `skill_boost` is properly applied while the tool is equipped.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

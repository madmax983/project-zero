# 1088 The Apex Predator Diet

## 1. Overview
Killing megafauna allows harvesting "Apex Meat". Feeding this to Pops provides massive morale and physical strength buffs. However, a diet heavily reliant on Apex Meat slowly causes the Pops to mutate, gaining animalistic traits, extreme aggressive tendencies, and massive calorie requirements.

## 2. Dependencies
- Megafauna / Hunting system
- Food / Diet consumption system
- Pop trait mutation system
- Pop need (calorie) system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_consuming_apex_meat_grants_buffs_and_increases_mutation_progress() {
        let mut app = App::new();
        app.add_event::<ConsumeFoodEvent>();
        app.add_systems(Update, process_apex_meat_consumption);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { current: 50.0 },
            PhysicalStrength { value: 10.0 },
            ApexMutationProgress { level: 0.0 },
        )).id();

        app.world_mut().resource_mut::<Events<ConsumeFoodEvent>>().send(ConsumeFoodEvent {
            pop: pop_entity,
            food_type: FoodType::ApexMeat,
        });

        app.update();

        let morale = app.world().get::<Morale>(pop_entity).unwrap();
        let strength = app.world().get::<PhysicalStrength>(pop_entity).unwrap();
        let mutation = app.world().get::<ApexMutationProgress>(pop_entity).unwrap();

        assert!(morale.current > 50.0, "Morale should increase after eating Apex Meat");
        assert!(strength.value > 10.0, "Physical strength should increase");
        assert!(mutation.level > 0.0, "Mutation progress should increase");
    }

    #[test]
    fn test_high_mutation_progress_applies_mutations() {
        let mut app = App::new();
        app.add_systems(Update, apply_apex_mutations);

        let pop_entity = app.world_mut().spawn((
            Pop,
            ApexMutationProgress { level: 100.0 }, // Threshold level
            CalorieRequirement { daily_amount: 2000.0 },
        )).id();

        app.update();

        let traits = app.world().get::<Traits>(pop_entity).expect("Pop should have traits added");
        assert!(traits.list.contains(&"Aggressive".to_string()), "Pop should gain the Aggressive trait");
        assert!(traits.list.contains(&"Animalistic".to_string()), "Pop should gain the Animalistic trait");

        let calories = app.world().get::<CalorieRequirement>(pop_entity).unwrap();
        assert!(calories.daily_amount > 2000.0, "Calorie requirements should massively increase");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

#[derive(Component)]
pub struct PhysicalStrength {
    pub value: f32,
}

#[derive(Component)]
pub struct ApexMutationProgress {
    pub level: f32,
}

#[derive(Component, Default)]
pub struct Traits {
    pub list: Vec<String>,
}

#[derive(Component)]
pub struct CalorieRequirement {
    pub daily_amount: f32,
}

pub enum FoodType {
    Standard,
    ApexMeat,
}

#[derive(Event)]
pub struct ConsumeFoodEvent {
    pub pop: Entity,
    pub food_type: FoodType,
}

pub fn process_apex_meat_consumption(
    mut events: EventReader<ConsumeFoodEvent>,
    mut query: Query<(&mut Morale, &mut PhysicalStrength, &mut ApexMutationProgress)>,
) {
    for event in events.read() {
        if let FoodType::ApexMeat = event.food_type {
            if let Ok((mut morale, mut strength, mut mutation)) = query.get_mut(event.pop) {
                morale.current += 20.0;
                strength.value += 5.0;
                mutation.level += 10.0;
            }
        }
    }
}

pub fn apply_apex_mutations(
    mut commands: Commands,
    mut query: Query<(Entity, &ApexMutationProgress, &mut CalorieRequirement, Option<&mut Traits>)>,
) {
    for (entity, progress, mut calories, traits_opt) in query.iter_mut() {
        if progress.level >= 100.0 {
            calories.daily_amount *= 2.0;

            if let Some(mut traits) = traits_opt {
                if !traits.list.contains(&"Aggressive".to_string()) {
                    traits.list.push("Aggressive".to_string());
                    traits.list.push("Animalistic".to_string());
                }
            } else {
                commands.entity(entity).insert(Traits {
                    list: vec!["Aggressive".to_string(), "Animalistic".to_string()],
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded stat increases and threshold values. Applying the multiplier iteratively if the system runs multiple times.
- **Improvements**: Prevent the `apply_apex_mutations` from running repeatedly and continually doubling calorie requirements. Add a `Mutated` tag component to mark the transition as complete, or reduce mutation progress upon successful mutation. Add gradual decay of mutation progress if the Pop returns to a standard diet.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Consuming Apex Meat increases morale, physical strength, and mutation progress.
- [ ] Reaching maximum mutation progress grants animalistic/aggressive traits and increases calorie requirements.

## 7. Technical Guidance
- Ensure the AI utility systems prioritize satisfying the massive new calorie requirement.
- Consider what happens when these mutated Pops lack Apex Meat; they should likely suffer severe withdrawal or start cannibalizing baseline Pops as per the fantasy description.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 1333: The Pet Singularity

## 1. Overview
A colony's beloved indigenous pets unexpectedly evolve and unionize. Pops can adopt indigenous fauna as "Pets" for a mood bonus. If exposed to specific industrial byproducts (like mutagenic coolants), these pets slowly gain sentience over time. Eventually, they demand rights, rations, and representation, shifting from simple items or companions to full `Pop` entities.

## 2. Dependencies
- Layer 1 Population (`Pop`, `Morale`, `Needs`)
- Layer 1 Flora/Fauna (`Fauna`, `Pet`)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_pet_adoption_boosts_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_pet_morale_bonus_system);

        let pop = app.world_mut().spawn((
            Pop,
            Morale(50.0),
            PetOwner { pet_type: PetType::RockBadger },
        )).id();

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(morale.0 > 50.0, "Adopting a pet should boost morale");
    }

    #[test]
    fn test_mutagenic_exposure_increases_sentience() {
        let mut app = App::new();
        app.add_systems(Update, mutate_pets_system);

        let pet = app.world_mut().spawn((
            Pet { sentience: 0.0 },
            GridPosition { x: 5, y: 5, z: 0 },
        )).id();

        let spill = app.world_mut().spawn((
            MutagenicSpill,
            GridPosition { x: 5, y: 5, z: 0 },
        )).id();

        app.update();

        let updated_pet = app.world().get::<Pet>(pet).unwrap();
        assert!(updated_pet.sentience > 0.0, "Proximity to MutagenicSpill must increase pet sentience");
    }

    #[test]
    fn test_sentient_pets_become_pops() {
        let mut app = App::new();
        app.add_systems(Update, sentient_pet_uprising_system);

        let pet = app.world_mut().spawn((
            Pet { sentience: 100.0 },
            Name::new("Fluffy"),
        )).id();

        app.update();

        assert!(app.world().get::<Pop>(pet).is_some(), "Fully sentient pet must gain the Pop component");
        assert!(app.world().get::<Needs>(pet).is_some(), "Fully sentient pet must gain Needs");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale(pub f32);

#[derive(Component)]
pub struct Needs;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PetType {
    RockBadger,
}

#[derive(Component)]
pub struct PetOwner {
    pub pet_type: PetType,
}

#[derive(Component)]
pub struct Pet {
    pub sentience: f32,
}

#[derive(Component)]
pub struct MutagenicSpill;

pub fn apply_pet_morale_bonus_system(mut query: Query<&mut Morale, With<PetOwner>>) {
    for mut morale in query.iter_mut() {
        morale.0 += 5.0; // Flat bonus for MVP
    }
}

pub fn mutate_pets_system(
    mut pets: Query<(&mut Pet, &GridPosition)>,
    spills: Query<&GridPosition, With<MutagenicSpill>>,
) {
    for (mut pet, pet_pos) in pets.iter_mut() {
        for spill_pos in spills.iter() {
            let dist = (pet_pos.x - spill_pos.x).abs() + (pet_pos.y - spill_pos.y).abs();
            if dist <= 1 {
                pet.sentience += 10.0;
            }
        }
    }
}

pub fn sentient_pet_uprising_system(
    mut commands: Commands,
    pets: Query<(Entity, &Pet)>,
) {
    for (entity, pet) in pets.iter() {
        if pet.sentience >= 100.0 {
            commands.entity(entity).insert(Pop).insert(Needs);
            // Optionally remove the Pet component or mark them as emancipated
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Grid:** Use a spatial grid resource to check proximity between pets and spills instead of O(N^2) loops.
- **Morale Cap:** Implement a cap to the morale bonus so pops don't get infinite morale if the system runs repeatedly.
- **Job System:** Ensure emancipated pets can take on jobs in the `UtilityAI` system.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Adopting a pet grants a morale bonus.
- [ ] Proximity to mutagenic spills increases a pet's sentience.
- [ ] Pets with maximum sentience convert into Pops with Needs.

## 7. Technical Guidance
- The transformation of Pet to Pop should trigger a Chronicle event notifying the player of the new sentient species.
- Sentient pets will need integration into the `Needs` metabolism system so they can starve or become unhappy.

## 8. Questions
*Builder: add questions here if spec is unclear.*

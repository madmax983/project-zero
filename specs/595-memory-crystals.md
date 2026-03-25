# 595: Memory Crystals

## 1. Overview
The skills of the master smith survive death, but so does his fear of spiders. A "Memory Crystal" item can be crafted from a dying Pop's neural pattern. A living Pop can "Equip" it to gain the skills, but they also inherit the dead Pop's Traits and Mood triggers (e.g., Phobias). You equip your new soldier with the "General's Crystal". He becomes a tactical genius but suddenly refuses to eat anything but synthetic nutrient paste because the General had a stomach ulcer. Tension: Preserve valuable skills (Efficiency) vs. Dilute the individuality of the living (Identity).

## 2. Dependencies
- Requires Layer 1 Memory and Skills systems.

## 3. RED Phase: Tests First
```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[test]
fn test_equip_memory_crystal_transfers_skills_and_traits() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, equip_memory_crystal_system);

    let mut skills = HashMap::new();
    skills.insert(SkillType::Combat, 50);
    let crystal = MemoryCrystal {
        skills,
        phobia: PhobiaType::Spiders,
    };

    let pop = app.world_mut().spawn(PopSkills::default()).id();

    // Act
    // Simulate equipping the crystal
    app.world_mut().entity_mut(pop).insert(EquippedCrystal(crystal));
    app.update();

    // Assert
    let pop_skills = app.world().get::<PopSkills>(pop).unwrap();
    assert_eq!(pop_skills.get(SkillType::Combat), 50);
    assert!(app.world().get::<Phobia>(pop).unwrap().0 == PhobiaType::Spiders);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SkillType {
    Combat,
    Farming,
    Engineering,
}

#[derive(Clone, PartialEq, Eq)]
pub enum PhobiaType {
    Spiders,
    Darkness,
}

#[derive(Clone)]
pub struct MemoryCrystal {
    pub skills: HashMap<SkillType, u32>,
    pub phobia: PhobiaType,
}

#[derive(Component)]
pub struct EquippedCrystal(pub MemoryCrystal);

#[derive(Component, Default)]
pub struct PopSkills {
    skills: HashMap<SkillType, u32>,
}

impl PopSkills {
    pub fn get(&self, skill: SkillType) -> u32 {
        *self.skills.get(&skill).unwrap_or(&0)
    }
}

#[derive(Component)]
pub struct Phobia(pub PhobiaType);

pub fn equip_memory_crystal_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PopSkills, &EquippedCrystal), Added<EquippedCrystal>>,
) {
    for (entity, mut skills, crystal) in query.iter_mut() {
        // Transfer skills
        for (skill, level) in &crystal.0.skills {
            skills.skills.insert(skill.clone(), *level);
        }

        // Transfer phobia
        commands.entity(entity).insert(Phobia(crystal.0.phobia.clone()));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Ensure Memory Crystals are treated as actual inventory items in Layer 1, utilizing the existing item/equipment systems rather than a raw component insert.
- Define a proper UI flow for crafting and assigning these crystals.
- Consider what happens when a crystal is unequipped. Do the skills fade immediately, or is there a "withdrawal" period?

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] Equipping a Memory Crystal successfully applies the stored skills to the Pop.
- [ ] Equipping a Memory Crystal successfully applies the stored negative traits (phobias) to the Pop.

## 7. Technical Guidance
- Add an `UnequipCrystalEvent` to cleanly handle skill/trait removal if the item is taken off.
- The extraction process (dying Pop -> Crystal) should hook into the `medical_notifications::death_system` or similar death event logic.

## 8. Questions
*Builder: add questions here if spec is unclear.*

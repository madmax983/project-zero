# 541 - The Memory Smugglers

## 1. Overview
**Layer:** 1 -> Cross-layer
**Fantasy:** Buying a dead man's peace of mind to survive the nightmare of the frontier.
**Mechanic:** A black market develops for "Memory Cores" extracted from dead Pops (as per Spec 288). Pops with high Stress can secretly purchase "Blanket Memories" (artificial or stolen memories of peaceful, happy lives). This instantly drops their Stress to zero and provides a massive Morale boost, but creates a high risk of "Identity Rejection," putting the Pop into a long-term coma or replacing their existing skills with the skills of the memory's original owner.

## 2. Dependencies
- `036` Pop Memory
- `288` The Memory Black Market
- `127` Stress Breakdowns

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_purchase_blanket_memory_reduces_stress() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_memory_smuggling_system);

        let pop_id = app.world_mut().spawn((
            Pop,
            StressTracker { current: 90.0, max: 100.0 },
            Morale { current: 20.0, ..default() },
            Wallet { credits: 500 },
        )).id();

        let black_market_id = app.world_mut().spawn((
            MemoryBlackMarket,
            Inventory::with_items(vec![Item::new(ItemType::MemoryCore(MemoryQuality::High))]),
        )).id();

        // Act
        app.world_mut().send_event(PurchaseMemoryEvent {
            buyer: pop_id,
            market: black_market_id,
        });
        app.update();

        // Assert
        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert_eq!(stress.current, 0.0, "Stress should be instantly reduced to 0");

        let morale = app.world().get::<Morale>(pop_id).unwrap();
        assert!(morale.current > 20.0, "Morale should receive a massive boost");
    }

    #[test]
    fn test_identity_rejection_coma() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_identity_rejection_system);

        let pop_id = app.world_mut().spawn((
            Pop,
            IdentityRejectionRisk { probability: 1.0 }, // Force rejection
            Health { current: 100.0, ..default() },
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<Comatose>(pop_id).is_some(), "Pop should enter a coma due to identity rejection");
    }

    #[test]
    fn test_identity_rejection_skill_replacement() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_identity_rejection_system);

        let mut skills = Skills::new();
        skills.set_level(SkillType::Mining, 10);

        let pop_id = app.world_mut().spawn((
            Pop,
            skills,
            IdentityRejectionRisk { probability: 1.0 }, // Force rejection
            StolenMemoryData { source_skill: SkillType::Farming, source_level: 8 },
        )).id();

        // Act
        app.update(); // Trigger rejection evaluation

        // Assert
        let new_skills = app.world().get::<Skills>(pop_id).unwrap();
        assert_eq!(new_skills.get_level(SkillType::Mining), 0, "Original skill should be erased");
        assert_eq!(new_skills.get_level(SkillType::Farming), 8, "Skill should be replaced by the memory's original owner");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct StressTracker { pub current: f32, pub max: f32 }
#[derive(Component)]
pub struct Morale { pub current: f32 }
#[derive(Component)]
pub struct Wallet { pub credits: u32 }
#[derive(Component)]
pub struct MemoryBlackMarket;
#[derive(Component)]
pub struct Comatose;
#[derive(Component)]
pub struct IdentityRejectionRisk { pub probability: f32 }
#[derive(Component)]
pub struct StolenMemoryData { pub source_skill: SkillType, pub source_level: u32 }
#[derive(Component)]
pub struct Pop;
#[derive(Component)]
pub struct Health { pub current: f32 }

#[derive(Event)]
pub struct PurchaseMemoryEvent { pub buyer: Entity, pub market: Entity }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkillType { Mining, Farming }

#[derive(Component)]
pub struct Skills { levels: std::collections::HashMap<SkillType, u32> }
impl Skills {
    pub fn new() -> Self { Self { levels: std::collections::HashMap::new() } }
    pub fn set_level(&mut self, skill: SkillType, level: u32) { self.levels.insert(skill, level); }
    pub fn get_level(&self, skill: SkillType) -> u32 { *self.levels.get(&skill).unwrap_or(&0) }
}

#[derive(Clone)]
pub enum ItemType { MemoryCore(MemoryQuality) }
#[derive(Clone)]
pub enum MemoryQuality { High, Low }
#[derive(Clone)]
pub struct Item { pub item_type: ItemType }
impl Item { pub fn new(item_type: ItemType) -> Self { Self { item_type } } }

#[derive(Component)]
pub struct Inventory { pub items: Vec<Item> }
impl Inventory {
    pub fn with_items(items: Vec<Item>) -> Self { Self { items } }
}

// Systems
pub fn process_memory_smuggling_system(
    mut events: EventReader<PurchaseMemoryEvent>,
    mut query_pops: Query<(&mut StressTracker, &mut Morale, &mut Wallet)>,
) {
    for event in events.read() {
        if let Ok((mut stress, mut morale, mut wallet)) = query_pops.get_mut(event.buyer) {
            if wallet.credits >= 100 {
                wallet.credits -= 100;
                stress.current = 0.0;
                morale.current += 50.0; // Massive boost
            }
        }
    }
}

pub fn process_identity_rejection_system(
    mut commands: Commands,
    mut query: Query<(Entity, &IdentityRejectionRisk, Option<&mut Skills>, Option<&StolenMemoryData>)>,
) {
    for (entity, risk, mut skills_opt, memory_data_opt) in query.iter_mut() {
        // Simplified random check for minimal implementation. In reality use `rand`
        if risk.probability >= 1.0 {
            if let (Some(mut skills), Some(data)) = (skills_opt, memory_data_opt) {
                skills.levels.clear();
                skills.set_level(data.source_skill, data.source_level);
            } else {
                commands.entity(entity).insert(Comatose);
            }
            commands.entity(entity).remove::<IdentityRejectionRisk>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactor:** Integrate with the main `Item` and `Inventory` resources to properly track the consumption of Memory Cores from the Black Market.
- **Improvement:** Connect the Identity Rejection probability to the `MemoryQuality` (low quality cores should have a higher chance of rejection).
- **Code Smell:** The `process_identity_rejection_system` currently lacks actual randomization in the test/minimal code. It should use `bevy_turborand` or `rand` to evaluate the probability against a generated float.
- **Integration:** Integrate the Identity Rejection Coma with the existing Medical care system, allowing Doctors to potentially treat or wake the Comatose pop after a duration.

## 6. Acceptance Criteria
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Purchasing a blanket memory sets Stress to 0.0 and boosts Morale.
- [ ] High rejection probability results in either Comatose state or Skill replacement.

## 7. Technical Guidance
- The actual trait/skill swap requires careful handling of Bevy components to ensure any dependent systems (like Work Execution) don't crash when a pop suddenly forgets how to do their current job. If a Pop is currently executing an action requiring the lost skill, the action must be gracefully aborted.
- Use `Commands::remove` to clear old skills if they are tracked via individual marker components, or simply clear the HashMap if using a centralized `Skills` struct.

## 8. Questions
*Builder: add questions here if spec is unclear.*

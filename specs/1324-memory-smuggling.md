# 1324: Memory Smuggling

## 1. Overview
The underground trade of illicit, specialized knowledge and stolen memories across the galaxy. Pops with highly prized skills or unique historical memories can have their "Memory" data illicitly copied and sold on the Black Market as a commodity. Other colonies can buy these to instantly impart skills or radical ideas to their own Pops.

## 2. Dependencies
- `036` Pop Memory (for memory system)
- `1043` Black Market Infrastructure (for trade mechanics)
- `051` Pop Skills and Experience (for skill transfers)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::memory::{Memory, MemoryType};
    use crate::layer1::skills::Skills;
    use crate::layer1::pop::Pop;
    use crate::layer1::economy::black_market::{BlackMarket, ContrabandItem};
    use crate::layer1::memory_smuggling::{SmuggledMemory, apply_smuggled_memory};

    #[test]
    fn test_smuggled_memory_creation() {
        let mut world = World::new();
        // Setup pop with high skill
        let pop = world.spawn((Pop, Skills::new().with("engineering", 100))).id();

        // Extract memory
        let memory_item = crate::layer1::memory_smuggling::extract_skill_memory(&mut world, pop, "engineering");
        assert_eq!(memory_item.skill_name, "engineering");
        assert_eq!(memory_item.skill_value, 100);
    }

    #[test]
    fn test_apply_smuggled_memory() {
        let mut world = World::new();
        let target_pop = world.spawn((Pop, Skills::new(), Memory::default())).id();
        let smuggled = SmuggledMemory { skill_name: "engineering".to_string(), skill_value: 80, corruption_chance: 0.1 };

        apply_smuggled_memory(&mut world, target_pop, smuggled);

        let skills = world.get::<Skills>(target_pop).unwrap();
        assert!(skills.get("engineering") > 0);

        let memory = world.get::<Memory>(target_pop).unwrap();
        assert!(memory.has_memory_type(MemoryType::Artificial));
    }

    #[test]
    fn test_memory_corruption() {
        let mut world = World::new();
        let target_pop = world.spawn((Pop, Skills::new(), Memory::default())).id();
        let smuggled = SmuggledMemory { skill_name: "medicine".to_string(), skill_value: 50, corruption_chance: 1.0 };

        apply_smuggled_memory(&mut world, target_pop, smuggled);

        let memory = world.get::<Memory>(target_pop).unwrap();
        assert!(memory.has_memory_type(MemoryType::Corrupted));
        // Verify negative trait or stress applied
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation in src/layer1/memory_smuggling.rs
use bevy_ecs::prelude::*;
use crate::layer1::memory::{Memory, MemoryType};
use crate::layer1::skills::Skills;

pub struct SmuggledMemory {
    pub skill_name: String,
    pub skill_value: u32,
    pub corruption_chance: f32,
}

pub fn extract_skill_memory(world: &mut World, pop_entity: Entity, skill_name: &str) -> SmuggledMemory {
    let skills = world.get::<Skills>(pop_entity).unwrap();
    let value = skills.get(skill_name).unwrap_or(0);
    SmuggledMemory {
        skill_name: skill_name.to_string(),
        skill_value: value,
        corruption_chance: 0.2, // Default base chance
    }
}

pub fn apply_smuggled_memory(world: &mut World, pop_entity: Entity, memory: SmuggledMemory) {
    let mut skills = world.get_mut::<Skills>(pop_entity).unwrap();
    skills.set(&memory.skill_name, memory.skill_value);

    let mut mem = world.get_mut::<Memory>(pop_entity).unwrap();
    mem.add(MemoryType::Artificial);

    if memory.corruption_chance >= 1.0 {
        mem.add(MemoryType::Corrupted);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate random chance correctly for `corruption_chance` in `apply_smuggled_memory` using the game's RNG system.
- Hook into the Black Market inventory system so these items can be bought and sold.
- Add UI notifications when a corrupted memory triggers an extreme event (like a sudden violent outburst).

## 6. Acceptance Criteria (Testable!)
- [ ] Tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Extracted memories accurately reflect source skills
- [ ] Target pops correctly receive skills and artificial memory flags

## 7. Technical Guidance
- Integrate with `BlackMarket` component to handle the buying/selling aspect.
- The `MemoryType` enum might need to be extended with `Artificial` and `Corrupted` if they don't exist.
- Use the established RNG service/resource for the corruption chance roll.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

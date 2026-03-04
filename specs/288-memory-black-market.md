# 288: The Memory Black Market

## 1. Overview

Stealing the experiences of the dead to shortcut education. When a highly skilled Pop dies, a shady faction might harvest their "Memory Core" (Layer 1 item). This item can be traded on the black market (Layer 2) or forcefully implanted into a rookie Pop, granting them max skills instantly but overwriting their personality and causing severe "Identity Rejection" stress.

Instant replacement of irreplaceable talent vs. severe psychological instability and loss of control.

## 2. Dependencies

- `051` Pop Skills and Experience
- `036` Pop Memory
- `039` Trade System
- `261` Shadow Markets

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Skills, SkillType, Traits, Trait};
    use crate::layer1::needs::{StressTracker, MemoryTracker};
    use crate::layer1::inventory::Inventory;
    use crate::layer1::resources::{ResourceType, ResourceItem};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (implant_memory_core_system, harvest_memory_core_system));
        app
    }

    #[test]
    fn test_implanting_memory_core_grants_skills_but_causes_stress_and_personality_overwrite() {
        let mut app = setup_app();

        // Create a rookie pop
        let pop_id = app.world_mut().spawn((
            Pop,
            Skills::new(), // Starts with 0 skills
            StressTracker { accumulated_stress: 0.0, max: 100.0 },
            Traits(vec![]),
            MemoryTracker::new(),
            Inventory::new(10.0),
        )).id();

        // Create a memory core resource item with some stored data
        let mut core_data = MemoryCoreData {
            skills: Skills::new(),
            traits: vec![Trait::Volatile], // Example trait from the dead pop
        };
        core_data.skills.add_xp(SkillType::Mining, 1000.0); // Max skill

        app.world_mut().send_event(ImplantMemoryCoreEvent {
            target: pop_id,
            core_data,
        });

        app.update();

        // Check effects on Pop
        let skills = app.world().get::<Skills>(pop_id).unwrap();
        assert_eq!(skills.get_level(SkillType::Mining), 10); // Maxed out

        let traits = app.world().get::<Traits>(pop_id).unwrap();
        assert!(traits.0.contains(&Trait::Volatile));

        let stress = app.world().get::<StressTracker>(pop_id).unwrap();
        assert!(stress.accumulated_stress >= 80.0); // Severe "Identity Rejection" stress
    }

    #[test]
    fn test_harvesting_memory_core_from_dead_pop_with_high_skills() {
        let mut app = setup_app();

        // Create a dead pop with high skills
        let mut dead_skills = Skills::new();
        dead_skills.add_xp(SkillType::Science, 1000.0);

        let dead_pop_id = app.world_mut().spawn((
            Pop,
            Dead,
            dead_skills,
            Traits(vec![Trait::Ambitious]),
        )).id();

        let harvester_id = app.world_mut().spawn((
            MemoryHarvester { processing: Some(dead_pop_id) },
            Inventory::new(10.0),
        )).id();

        app.world_mut().send_event(HarvestMemoryCoreEvent {
            target: dead_pop_id,
            harvester: harvester_id,
        });

        app.update();

        // Check if the harvester's inventory now contains a Memory Core
        let harvester_inv = app.world().get::<Inventory>(harvester_id).unwrap();
        assert_eq!(harvester_inv.count(ResourceType::MemoryCore), 1);

        // Ensure the dead pop entity is despawned/processed
        assert!(app.world().get_entity(dead_pop_id).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::inventory::Inventory;
use crate::layer1::resources::{ResourceType, ResourceItem};
use crate::layer1::pop::{Pop, Dead, Skills, Traits, Trait};
use crate::layer1::needs::StressTracker;

#[derive(Clone)]
pub struct MemoryCoreData {
    pub skills: Skills,
    pub traits: Vec<Trait>,
}

#[derive(Event)]
pub struct ImplantMemoryCoreEvent {
    pub target: Entity,
    pub core_data: MemoryCoreData,
}

#[derive(Event)]
pub struct HarvestMemoryCoreEvent {
    pub target: Entity,
    pub harvester: Entity,
}

#[derive(Component)]
pub struct MemoryHarvester {
    pub processing: Option<Entity>,
}

pub fn implant_memory_core_system(
    mut events: EventReader<ImplantMemoryCoreEvent>,
    mut pops: Query<(&mut Skills, &mut Traits, &mut StressTracker), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((mut skills, mut traits, mut stress)) = pops.get_mut(event.target) {
            // Overwrite skills
            *skills = event.core_data.skills.clone();

            // Overwrite traits (or append them based on balance)
            traits.0 = event.core_data.traits.clone();

            // Apply massive "Identity Rejection" stress
            stress.accumulated_stress += 80.0;
        }
    }
}

pub fn harvest_memory_core_system(
    mut events: EventReader<HarvestMemoryCoreEvent>,
    mut harvesters: Query<&mut Inventory>,
    targets: Query<(&Skills, &Traits), (With<Pop>, With<Dead>)>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(mut inv) = harvesters.get_mut(event.harvester) {
            if let Ok((_skills, _traits)) = targets.get(event.target) {
                // In a fuller implementation, we would extract the data and attach it to the item.
                // For MVP, we just yield the resource type.
                inv.add(ResourceType::MemoryCore, 1.0);

                // Despawn the processed body
                commands.entity(event.target).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Data Physicality:** Ensure `MemoryCore` items actually store the `MemoryCoreData` inside the ECS instead of just being generic `ResourceType::MemoryCore`. You might need a `MemoryCoreItem` component that wraps the `ResourceItem` and holds the data payload.
- **Shadow Market Integration:** Integrate the purchasing/selling of these cores into `src/layer1/shadow_market.rs`, ensuring `ShadowTrader`s randomly stock them.
- **Identity Rejection Mechanics:** The flat 80.0 stress penalty could trigger immediate mental breaks. Consider adding an `IdentityRejection` trait/component that causes persistent stress over a long duration rather than an instant spike.
- **Skill Transfer Balance:** Instantly getting max skills is extremely powerful. We might want to cap the transferred XP at a percentage or apply "Memory Decay" over time.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] Test coverage $\ge$ 85% for the new module.
- [ ] Harvesting a dead pop with high skills produces a `MemoryCore`.
- [ ] Implanting a `MemoryCore` overwrites a Pop's skills and traits while applying a massive stress penalty.
- [ ] `MemoryCore` can be stocked and traded by a `ShadowTrader`.

## 7. Technical Guidance

- Implement systems in a new file `src/layer1/pop/memory_core.rs`.
- Ensure `ResourceType::MemoryCore` is added to the enum in `src/layer1/resources.rs`.
- Connect the `ImplantMemoryCoreEvent` to a specific medical or highly advanced facility building (e.g., `SiphonBox` or `MemoryLab`).
- Add logic in the death system to occasionally mark high-skill corpses as viable for harvesting by the shady faction.

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

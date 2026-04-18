# 1086 The Heirloom Tool

## 1. Overview
When a highly skilled Pop dies, there is a chance they leave behind an "Heirloom Tool." The Pop who inherits this tool gains a massive efficiency boost but also adopts the personality traits, memories, and grudges of the original owner. This mechanic creates lineages of specialized, but potentially behaviorally warped, colonists.

## 2. Dependencies
- Pop death event system
- Pop trait and memory system
- Inventory / Tool system
- Work efficiency calculation

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_heirloom_tool_creation_on_death() {
        let mut app = App::new();
        app.add_event::<PopDeathEvent>();
        app.add_systems(Update, process_pop_death_for_heirloom);

        let entity = app.world_mut().spawn((
            SkillLevel { level: 100 }, // High skill
            Trait { id: "Paranoid".to_string() },
        )).id();

        app.world_mut().resource_mut::<Events<PopDeathEvent>>().send(PopDeathEvent {
            entity,
        });

        app.update();

        // Check if an heirloom tool was created in the world
        let mut heirloom_query = app.world_mut().query::<&HeirloomTool>();
        let mut count = 0;
        let mut inherited_trait = String::new();

        for heirloom in heirloom_query.iter(app.world()) {
            count += 1;
            inherited_trait = heirloom.original_trait.clone();
        }

        assert_eq!(count, 1, "An heirloom tool should be spawned on high skill pop death");
        assert_eq!(inherited_trait, "Paranoid", "The heirloom should inherit the pop's trait");
    }

    #[test]
    fn test_heirloom_tool_equip_transfers_traits() {
        let mut app = App::new();
        app.add_event::<EquipHeirloomEvent>();
        app.add_systems(Update, process_heirloom_equip);

        let pop_entity = app.world_mut().spawn(Pop).id();
        let tool_entity = app.world_mut().spawn(HeirloomTool {
            original_trait: "Paranoid".to_string(),
            efficiency_boost: 50.0,
        }).id();

        app.world_mut().resource_mut::<Events<EquipHeirloomEvent>>().send(EquipHeirloomEvent {
            pop: pop_entity,
            tool: tool_entity,
        });

        app.update();

        // The pop should now have the inherited trait and an efficiency buff
        let pop_trait = app.world().get::<Trait>(pop_entity).expect("Pop should gain the trait");
        assert_eq!(pop_trait.id, "Paranoid", "Pop should inherit the trait from the heirloom");

        let pop_efficiency = app.world().get::<EfficiencyBuff>(pop_entity).expect("Pop should gain efficiency buff");
        assert_eq!(pop_efficiency.amount, 50.0, "Pop should get the heirloom efficiency boost");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct SkillLevel {
    pub level: u32,
}

#[derive(Component, Clone)]
pub struct Trait {
    pub id: String,
}

#[derive(Event)]
pub struct PopDeathEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct HeirloomTool {
    pub original_trait: String,
    pub efficiency_boost: f32,
}

#[derive(Event)]
pub struct EquipHeirloomEvent {
    pub pop: Entity,
    pub tool: Entity,
}

#[derive(Component)]
pub struct EfficiencyBuff {
    pub amount: f32,
}

pub fn process_pop_death_for_heirloom(
    mut commands: Commands,
    mut events: EventReader<PopDeathEvent>,
    query: Query<(&SkillLevel, &Trait)>,
) {
    for event in events.read() {
        if let Ok((skill, pop_trait)) = query.get(event.entity) {
            // Simplified check: Always spawn if skill is high enough for testing
            if skill.level >= 100 {
                commands.spawn(HeirloomTool {
                    original_trait: pop_trait.id.clone(),
                    efficiency_boost: 50.0,
                });
            }
        }
    }
}

pub fn process_heirloom_equip(
    mut commands: Commands,
    mut events: EventReader<EquipHeirloomEvent>,
    tool_query: Query<&HeirloomTool>,
) {
    for event in events.read() {
        if let Ok(heirloom) = tool_query.get(event.tool) {
            commands.entity(event.pop).insert(Trait {
                id: heirloom.original_trait.clone(),
            });
            commands.entity(event.pop).insert(EfficiencyBuff {
                amount: heirloom.efficiency_boost,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded skill threshold and efficiency boost. Simplistic 1:1 trait transfer.
- **Improvements**: Use an RNG roll to determine heirloom creation chance based on skill level. Expand `HeirloomTool` to hold a collection of traits and memories, not just a single string trait.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] High-skill Pop deaths have a chance to spawn an Heirloom Tool carrying their traits.
- [ ] Equipping an Heirloom Tool applies the stored traits and an efficiency buff to the new Pop.

## 7. Technical Guidance
- Integrate the Heirloom Tool as an actual inventory item that can be hauled and stored if the specific inventory architecture requires it.
- When transferring memories, be mindful of UI updates and ensure the chronicle attributes the original memory correctly.

## 8. Questions
*Builder: add questions here if spec is unclear.*

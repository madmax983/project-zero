# 1085 The Resource Curse

## 1. Overview
Finding the most valuable substance in the universe makes the colony a target. A colony discovering a hyper-valuable, unique resource generates massive wealth from mining, but the intense galactic attention automatically reduces the diplomatic standing with neighboring Layer 3 factions and significantly increases pirate raid frequency on Layer 2 and Layer 1.

## 2. Dependencies
- Base resource mining mechanics
- Diplomatic standing system (Layer 3)
- Pirate raid event system (Layer 1/2)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_hyper_valuable_resource_discovery_triggers_attention() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ResourceMinedEvent>();
        app.add_systems(Update, process_hyper_resources);

        app.init_resource::<DiplomaticRelations>();
        app.init_resource::<PirateThreatLevel>();

        let initial_standing = 100.0;
        app.world_mut().resource_mut::<DiplomaticRelations>().standing = initial_standing;
        app.world_mut().resource_mut::<PirateThreatLevel>().level = 0.0;

        // Act
        app.world_mut().resource_mut::<Events<ResourceMinedEvent>>().send(ResourceMinedEvent {
            resource_type: ResourceType::HyperValuable,
            amount: 10,
        });

        app.update();

        // Assert
        let final_standing = app.world().resource::<DiplomaticRelations>().standing;
        let final_threat = app.world().resource::<PirateThreatLevel>().level;

        assert!(final_standing < initial_standing, "Diplomatic standing should decrease");
        assert!(final_threat > 0.0, "Pirate threat level should increase");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

pub enum ResourceType {
    Common,
    HyperValuable,
}

#[derive(Event)]
pub struct ResourceMinedEvent {
    pub resource_type: ResourceType,
    pub amount: u32,
}

#[derive(Resource, Default)]
pub struct DiplomaticRelations {
    pub standing: f32,
}

#[derive(Resource, Default)]
pub struct PirateThreatLevel {
    pub level: f32,
}

pub fn process_hyper_resources(
    mut events: EventReader<ResourceMinedEvent>,
    mut diplomacy: ResMut<DiplomaticRelations>,
    mut pirates: ResMut<PirateThreatLevel>,
) {
    for event in events.read() {
        if let ResourceType::HyperValuable = event.resource_type {
            diplomacy.standing -= 5.0 * (event.amount as f32);
            pirates.level += 10.0 * (event.amount as f32);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded penalty values (5.0 and 10.0).
- **Improvements**: Extract these penalty constants into a configurable `ResourceCurseSettings` resource. Add a system that gradually cools down the pirate threat if mining stops.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Mining a hyper-valuable resource reduces diplomatic standing.
- [ ] Mining a hyper-valuable resource increases pirate threat level.

## 7. Technical Guidance
- Ensure `ResourceMinedEvent` is properly emitted by the actual gathering/mining systems when a hyper-valuable resource node is successfully tapped.
- The pirate threat level should likely tie into a director/spawner that schedules pirate raid events based on this threat level resource.

## 8. Questions
*Builder: add questions here if spec is unclear.*

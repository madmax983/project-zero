# 1087 Embezzlement Architecture

## 1. Overview
Corrupt planetary governors secretly use stolen materials to build hidden "Luxury Bunkers" or "Private Gardens" underneath or inside existing structures. These parasitic structures drain power and lower structural integrity of the host building.

## 2. Dependencies
- Building structural integrity system
- Governor corruption/traits system
- Building power consumption system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_embezzlement_creates_parasitic_structure() {
        let mut app = App::new();
        app.add_event::<EmbezzlementEvent>();
        app.add_systems(Update, process_embezzlement);

        let building_entity = app.world_mut().spawn((
            Building,
            StructuralIntegrity { max: 100.0, current: 100.0 },
            PowerConsumption { amount: 10.0 },
        )).id();

        app.world_mut().resource_mut::<Events<EmbezzlementEvent>>().send(EmbezzlementEvent {
            target_building: building_entity,
            embezzled_amount: 50.0,
        });

        app.update();

        // Check if parasitic structure component was added
        let parasite = app.world().get::<ParasiticStructure>(building_entity)
            .expect("Building should have a parasitic structure added");
        assert_eq!(parasite.power_drain, 5.0, "Parasite should drain power");

        let integrity = app.world().get::<StructuralIntegrity>(building_entity).unwrap();
        assert!(integrity.current < integrity.max, "Structural integrity should be lowered by the parasite");

        let power = app.world().get::<PowerConsumption>(building_entity).unwrap();
        assert!(power.amount > 10.0, "Power consumption should be increased by the parasite");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct StructuralIntegrity {
    pub max: f32,
    pub current: f32,
}

#[derive(Component)]
pub struct PowerConsumption {
    pub amount: f32,
}

#[derive(Event)]
pub struct EmbezzlementEvent {
    pub target_building: Entity,
    pub embezzled_amount: f32,
}

#[derive(Component)]
pub struct ParasiticStructure {
    pub power_drain: f32,
    pub integrity_penalty: f32,
}

pub fn process_embezzlement(
    mut commands: Commands,
    mut events: EventReader<EmbezzlementEvent>,
    mut query: Query<(&mut StructuralIntegrity, &mut PowerConsumption)>,
) {
    for event in events.read() {
        if let Ok((mut integrity, mut power)) = query.get_mut(event.target_building) {
            let parasite = ParasiticStructure {
                power_drain: event.embezzled_amount * 0.1,
                integrity_penalty: event.embezzled_amount * 0.2,
            };

            integrity.current -= parasite.integrity_penalty;
            power.amount += parasite.power_drain;

            commands.entity(event.target_building).insert(parasite);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded scaling factors for drain and penalty. Modifying current integrity and power consumption directly instead of using a modifier system.
- **Improvements**: Integrate with a proper stat modifier/buff system to calculate the final structural integrity and power draw so the penalty can be dynamically updated or removed (e.g., if the bunker is discovered and purged).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Embezzlement events spawn hidden parasitic structures inside buildings.
- [ ] Parasitic structures increase power consumption and lower structural integrity of the host building.

## 7. Technical Guidance
- The parasitic structure should probably not be visible to the player immediately. Consider adding a `Hidden` or `Undiscovered` component that requires an audit action to reveal.
- Ensure that if the host building is destroyed, the parasitic structure is also destroyed.

## 8. Questions
*Builder: add questions here if spec is unclear.*

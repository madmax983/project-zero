# 1043: Black Market Infrastructure

## 1. Overview
A parasitic, invisible economy operating right under your nose, linking desperate colonists to off-world smugglers. Smuggler fleets (Layer 2) occasionally visit the system and establish hidden "Drop Nodes" on the planet's surface (Layer 1). Pops with low wealth or high unrest will secretly transport stolen colony resources (like alloys) to these nodes in exchange for illegal, high-morale contraband. Shutting down the Drop Nodes returns stolen resources but instantly crashes the artificial morale.

## 2. Dependencies
- Layer 1 Resource Management (Alloys, etc.)
- Layer 1 Pops, Needs, and Morale (or `Unrest`)
- Integration with an event/spawning system for Drop Nodes

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer1::components::{Pop, Morale, Inventory};
    use scale::layer1::resources::{ColonyResources, ResourceType};

    #[test]
    fn test_smugglers_establish_drop_node() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SmugglerArrivalEvent>();
        app.add_systems(Update, handle_smuggler_arrival);

        // Act
        app.world_mut().send_event(SmugglerArrivalEvent { intensity: 1 });
        app.update();

        // Assert
        let mut query = app.world_mut().query::<&DropNode>();
        assert_eq!(query.iter(app.world()).count(), 1, "A Drop Node should be established upon smuggler arrival");
    }

    #[test]
    fn test_desperate_pop_exchanges_resources_for_contraband() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyResources::new());
        app.world_mut().resource_mut::<ColonyResources>().add(ResourceType::Alloys, 50.0);

        app.add_systems(Update, pop_smuggling_system);

        let drop_node_entity = app.world_mut().spawn(DropNode { stored_alloys: 0.0 }).id();
        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 20.0, modifiers: vec![] }, // Low morale
            Desperate, // Tag indicating they are likely to smuggle
        )).id();

        // Act
        app.update();

        // Assert
        let colony_resources = app.world().resource::<ColonyResources>();
        assert!(colony_resources.get(ResourceType::Alloys) < 50.0, "Colony alloys should have been stolen");

        let drop_node = app.world().get::<DropNode>(drop_node_entity).unwrap();
        assert!(drop_node.stored_alloys > 0.0, "Drop Node should have accumulated stolen alloys");

        let pop_morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(pop_morale.value > 20.0, "Pop should have received a morale boost from contraband");
    }

    #[test]
    fn test_shutting_down_drop_node_returns_resources_and_crashes_morale() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyResources::new());
        app.add_event::<ShutdownDropNodeEvent>();
        app.add_systems(Update, shutdown_drop_node_system);

        let drop_node_entity = app.world_mut().spawn(DropNode { stored_alloys: 100.0 }).id();
        let pop_entity = app.world_mut().spawn((
            Pop,
            Morale { value: 80.0, modifiers: vec![] }, // Artificially high
            ContrabandUser,
        )).id();

        // Act
        app.world_mut().send_event(ShutdownDropNodeEvent { node_entity: drop_node_entity });
        app.update();

        // Assert
        assert!(app.world().get::<DropNode>(drop_node_entity).is_none(), "Drop Node should be destroyed");

        let colony_resources = app.world().resource::<ColonyResources>();
        assert_eq!(colony_resources.get(ResourceType::Alloys), 100.0, "Stored alloys should be returned to the colony");

        let pop_morale = app.world().get::<Morale>(pop_entity).unwrap();
        assert!(pop_morale.value < 80.0, "Pop morale should crash when their contraband supply is cut off");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use scale::layer1::components::{Pop, Morale};
use scale::layer1::resources::{ColonyResources, ResourceType};

#[derive(Event)]
pub struct SmugglerArrivalEvent {
    pub intensity: u32,
}

#[derive(Component)]
pub struct DropNode {
    pub stored_alloys: f32,
}

#[derive(Component)]
pub struct Desperate;

#[derive(Component)]
pub struct ContrabandUser;

#[derive(Event)]
pub struct ShutdownDropNodeEvent {
    pub node_entity: Entity,
}

pub fn handle_smuggler_arrival(
    mut commands: Commands,
    mut events: EventReader<SmugglerArrivalEvent>,
) {
    for _ in events.read() {
        commands.spawn(DropNode { stored_alloys: 0.0 });
    }
}

pub fn pop_smuggling_system(
    mut colony_resources: ResMut<ColonyResources>,
    mut drop_nodes: Query<&mut DropNode>,
    mut desperate_pops: Query<(Entity, &mut Morale), With<Desperate>>,
    mut commands: Commands,
) {
    if let Ok(mut node) = drop_nodes.get_single_mut() {
        for (pop_entity, mut morale) in desperate_pops.iter_mut() {
            if colony_resources.get(ResourceType::Alloys) >= 5.0 {
                colony_resources.remove(ResourceType::Alloys, 5.0);
                node.stored_alloys += 5.0;
                morale.value += 20.0;
                commands.entity(pop_entity).insert(ContrabandUser);
            }
        }
    }
}

pub fn shutdown_drop_node_system(
    mut commands: Commands,
    mut events: EventReader<ShutdownDropNodeEvent>,
    mut nodes: Query<&DropNode>,
    mut colony_resources: ResMut<ColonyResources>,
    mut contraband_users: Query<&mut Morale, With<ContrabandUser>>,
) {
    for event in events.read() {
        if let Ok(node) = nodes.get(event.node_entity) {
            colony_resources.add(ResourceType::Alloys, node.stored_alloys);
            commands.entity(event.node_entity).despawn();

            for mut morale in contraband_users.iter_mut() {
                morale.value -= 50.0;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `SmugglerArrivalEvent` should be emitted by Layer 2 fleet logic when a smuggler fleet enters orbit.
- **Morale System**: Use proper modifier structures inside `Morale` instead of just raw values (e.g., `MoraleModifier::ContrabandWithdrawal`).
- **Resource Limits**: Ensure a cap on how many resources a node can hoard.
- **Detection**: Add mechanics where players need to expend security resources or wait for certain Intel to discover the hidden Drop Nodes before they can shut them down.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Smuggler arrival spawns a Drop Node.
- [ ] Desperate Pops steal colony resources and gain a morale boost.
- [ ] Shutting down the node returns the stolen goods but applies a harsh negative morale effect.

## 7. Technical Guidance
- Place the core logic in `src/layer1/economy/black_market.rs` or an equivalent module depending on the current directory structure.
- Consider utilizing the `Utility AI` architecture to make the Pops *decide* to visit the drop node instead of instantly teleporting resources in a system.
- Integrate the contraband withdrawal with the existing `Morale` component's `modifiers` list so it decays naturally over time instead of just permanently crashing the raw value.

## 8. Questions
*Builder: add questions here if spec is unclear.*

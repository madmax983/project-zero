# Specification: 1248 Pneumatic Tubes

## 1. Overview
The thwump of a canister arriving. Futurama-style logistics. Expensive piping that instantly moves small items (Food, Mail, Samples) between buildings using pressure. Can clog if overused or leak pressure if damaged.

## 2. Dependencies
- Building mechanics
- Resource logistics

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pneumatic_tube_transport() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(PneumaticTubePlugin);

    // Arrange
    let src = app.world_mut().spawn((PneumaticNode { connected_to: None }, Inventory { items: vec!["Food".to_string()] })).id();
    let dst = app.world_mut().spawn((PneumaticNode { connected_to: None }, Inventory { items: vec![] })).id();

    app.world_mut().entity_mut(src).get_mut::<PneumaticNode>().unwrap().connected_to = Some(dst);

    // Act
    app.update();

    // Assert
    assert!(app.world().get::<Inventory>(src).unwrap().items.is_empty());
    assert_eq!(app.world().get::<Inventory>(dst).unwrap().items.len(), 1);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct PneumaticNode {
    pub connected_to: Option<Entity>,
}

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<String>,
}

pub struct PneumaticTubePlugin;

impl Plugin for PneumaticTubePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, pneumatic_transport_system);
    }
}

fn pneumatic_transport_system(
    mut queries: ParamSet<(
        Query<(Entity, &PneumaticNode, &mut Inventory)>,
        Query<&mut Inventory>,
    )>
) {
    let mut transfers = Vec::new();
    for (src_entity, node, mut inv) in queries.p0().iter_mut() {
        if let Some(dst) = node.connected_to {
            if let Some(item) = inv.items.pop() {
                transfers.push((dst, item));
            }
        }
    }

    for (dst, item) in transfers {
        if let Ok(mut dst_inv) = queries.p1().get_mut(dst) {
            dst_inv.items.push(item);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Move string identifiers to real resource enums
- Add pressure handling and power requirements

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Beware of circular dependencies between pneumatic nodes

## 8. Questions
*Builder: add questions here if spec is unclear.*

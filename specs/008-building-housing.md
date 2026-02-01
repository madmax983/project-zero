# 008: Housing Building

## Overview

Housing provides shelter and rest for pops. A pop assigned to housing slowly restores their rest need. This is the first building with actual gameplay function.

## Dependencies

- `006` — Pop needs (rest need exists)
- `007` — Building placement (can place buildings)

## Requirements

### Must Have
- Housing building type in BuildingType enum
- Housing has capacity (max 2 pops)
- Housing has "resident" slots
- Pops in housing restore rest need each tick
- Visual distinction from other buildings (brown rectangle)

### Must NOT Have
- Automatic pop assignment (that's the job system spec)
- Housing construction time
- Housing upgrades

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct Housing {
    pub capacity: usize,
    pub residents: Vec<Entity>,
}

impl Default for Housing {
    fn default() -> Self {
        Self {
            capacity: 2,
            residents: Vec::new(),
        }
    }
}
```

### Rest Restoration

```rust
const REST_RESTORE_PER_TICK: f32 = 0.05;  // Full rest in ~20 ticks

fn restore_rest_in_housing(
    mut tick_events: EventReader<TickEvent>,
    housing_query: Query<&Housing>,
    mut pop_query: Query<&mut Needs, With<Pop>>,
) {
    for _ in tick_events.read() {
        for housing in &housing_query {
            for &resident in &housing.residents {
                if let Ok(mut needs) = pop_query.get_mut(resident) {
                    needs.rest = (needs.rest + REST_RESTORE_PER_TICK).min(1.0);
                }
            }
        }
    }
}
```

### Spawning Housing

When placed via building system:

```rust
fn spawn_housing(
    commands: &mut Commands,
    position: GridPosition,
) -> Entity {
    let world_pos = position.to_world();
    commands.spawn((
        Building { building_type: BuildingType::Housing },
        Housing::default(),
        position,
        Sprite {
            color: BuildingType::Housing.color(),
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
            ..default()
        },
        Transform::from_xyz(world_pos.x, world_pos.y, 0.5), // Above terrain, below pops
    )).id()
}
```

### Visual Feedback (Optional Enhancement)

Show occupancy somehow—darken sprite when occupied, or show small dots for residents.

## Acceptance Criteria

- [ ] Can place Housing via build mode
- [ ] Housing has `Housing` component with capacity 2
- [ ] Housing renders as brown rectangle
- [ ] If a pop is manually added to `residents`, their rest increases each tick
- [ ] Rest caps at 1.0
- [ ] Housing appears above terrain but below pops (z-ordering)
- [ ] `cargo check` passes

## Notes

Actual assignment of pops to housing will be handled by the job system (spec 010). For now, a builder could manually add a pop to `residents` in code to verify rest restoration works.

## Questions

*Builder: add questions here if spec is unclear.*

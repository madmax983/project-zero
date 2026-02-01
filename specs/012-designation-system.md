# 012: Designation System

## Overview

Allows the player to mark entities or terrain for jobs (Chopping trees, Mining rock). This is the "Order" interface common in colony sims.

## Dependencies

- `006` — Building Placement (reusing cursor/input patterns)
- `011` — Resources (target of designations)

## Requirements

### Must Have
- Designate mode toggle ('V' key)
- Designation types: `Chop` (for trees), `Mine` (for rock), `Cancel` (remove designation)
- Cursor movement (WASD/Arrows) in designate mode
- Place designation with Enter/Space
- Visual overlay showing designation (e.g., colored background or 'X' char)
- Persist designations as components on map

### Must NOT Have
- The actual job logic (Spec 013 handles the work)
- Drag-and-drop box selection (MVP is single tile click)

## Technical Guidance

### Components

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignationType {
    Chop,
    Mine,
}

#[derive(Component)]
pub struct Designation {
    pub designation_type: DesignationType,
}
```

### UI Resource

```rust
#[derive(Resource, Default)]
pub struct DesignateMode {
    pub active: bool,
    pub cursor: GridPosition,
    pub selected_type: DesignationType,
}
```

### Input Handling

Similar to Build Mode (Spec 006):
- 'V' toggles mode.
- 'Tab' cycles types (Chop -> Mine -> Cancel?).
- 'Enter' applies designation.

**Validation Logic:**
- `Chop` can only be applied to a tile with a `Tree`.
- `Mine` can only be applied to a tile with `TerrainType::Rock`.
- `Cancel` removes any `Designation` entity at cursor.

### Application System

```rust
fn apply_designation(world: &mut World) {
    let mode = world.resource::<DesignateMode>();
    let pos = mode.cursor;

    // Check if designation already exists
    let existing = world.query::<(&GridPosition, &Designation)>()
        .iter(world)
        .find(|(p, _)| p.x == pos.x && p.y == pos.y);

    if existing.is_some() {
        // Maybe remove old one first?
        // For MVP, just don't double place.
        return;
    }

    // Validate Target
    let valid = match mode.selected_type {
        DesignationType::Chop => {
            // Check for Tree entity
            world.query::<(&GridPosition, With<Tree>)>()
                .iter(world)
                .any(|(p, _)| p.x == pos.x && p.y == pos.y)
        },
        DesignationType::Mine => {
            // Check for Rock terrain
            let terrain = world.resource::<TerrainGrid>();
            if let Some(tile) = terrain.get(pos.x as usize, pos.y as usize) {
                tile == TerrainType::Rock
            } else {
                false
            }
        }
    };

    if valid {
        world.spawn((
            Designation { designation_type: mode.selected_type },
            GridPosition { x: pos.x, y: pos.y },
        ));
    }
}
```

### Rendering

Overlay designations on the map.
- `Chop`: Red 'X' or background? Maybe `C` (Cyan).
- `Mine`: `M` (Magenta).

In render loop:
```rust
if let Some((_, des)) = designations.iter().find(|(p, _)| p.x == world_x && p.y == world_y) {
    let (ch, col) = match des.designation_type {
        DesignationType::Chop => ('X', Color::Red),
        DesignationType::Mine => ('M', Color::Magenta),
    };
    // Render ON TOP of tree/terrain
    spans.push(Span::styled(ch.to_string(), Style::default().fg(col).bg(Color::Black)));
    continue;
}
```

### Status Bar

Update to show "DESIGNATE: Chop" etc.

## Acceptance Criteria

- [ ] Pressing 'V' enters Designate mode
- [ ] Cursor moves
- [ ] Can place 'Chop' on Trees (appears as Red X)
- [ ] Cannot place 'Chop' on empty ground
- [ ] Can place 'Mine' on Rock (appears as Magenta M)
- [ ] Cannot place 'Mine' on Grass
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*

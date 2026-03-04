# 005: Pop Needs (Hunger, Rest)

## Overview

Pops have needs that decay over time. Unmet needs eventually kill them. This creates the core survival pressure of the colony.

## Dependencies

- `003` — UI layout (SimulationTime for ticks)
- `004` — Pop entity exists

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/needs.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_needs_default() {
        let needs = Needs::default();
        assert_eq!(needs.hunger, 0.8);
        assert_eq!(needs.rest, 0.8);
    }

    #[test]
    fn test_needs_worst() {
        let needs1 = Needs { hunger: 0.5, rest: 0.7 };
        assert_eq!(needs1.worst(), 0.5);

        let needs2 = Needs { hunger: 0.9, rest: 0.3 };
        assert_eq!(needs2.worst(), 0.3);

        let needs3 = Needs { hunger: 0.5, rest: 0.5 };
        assert_eq!(needs3.worst(), 0.5);
    }

    #[test]
    fn test_needs_clamped_to_zero() {
        let mut needs = Needs { hunger: 0.01, rest: 0.01 };
        needs.hunger -= 0.02;
        needs.rest -= 0.02;

        assert!(needs.hunger >= 0.0);
        assert!(needs.rest >= 0.0);
    }

    #[test]
    fn test_needs_clamped_to_one() {
        let mut needs = Needs { hunger: 0.99, rest: 0.99 };
        needs.hunger += 0.1;
        needs.rest += 0.1;

        assert!(needs.hunger <= 1.0);
        assert!(needs.rest <= 1.0);
    }

    #[test]
    fn test_decay_needs_system() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger < 0.8, "Hunger should have decayed");
        assert!(needs.rest < 0.8, "Rest should have decayed");
        assert!(needs.hunger >= 0.0, "Hunger should not be negative");
        assert!(needs.rest >= 0.0, "Rest should not be negative");
    }

    #[test]
    fn test_decay_multiple_ticks() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        for _ in 0..10 {
            decay_needs_system(&mut world);
        }

        let needs = world.query::<&Needs>().single(&world);
        // After 10 ticks of decay
        assert!(needs.hunger < 0.6, "Hunger should decay significantly");
        assert!(needs.rest < 0.7, "Rest should decay");
    }

    #[test]
    fn test_kill_starving_pops_system() {
        let mut world = World::new();

        // Spawn healthy pop
        world.spawn((Pop, Needs { hunger: 0.5, rest: 0.5 }));

        // Spawn starving pop
        world.spawn((Pop, Needs { hunger: 0.0, rest: 0.5 }));

        kill_starving_pops_system(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Only healthy pop should survive");
    }

    #[test]
    fn test_kill_only_when_hunger_zero() {
        let mut world = World::new();

        // Pop with very low hunger but not zero
        world.spawn((Pop, Needs { hunger: 0.01, rest: 0.0 }));

        kill_starving_pops_system(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Pop with 0.01 hunger should survive");
    }

    #[test]
    fn test_pop_display_basic_healthy() {
        let needs = Needs { hunger: 0.8, rest: 0.8 };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☺');
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_pop_display_basic_warning() {
        let needs = Needs { hunger: 0.5, rest: 0.8 };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☻');
        assert_eq!(color, Color::Rgb(255, 165, 0)); // Orange
    }

    #[test]
    fn test_pop_display_basic_critical() {
        let needs = Needs { hunger: 0.2, rest: 0.8 };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☹');
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_pop_display_basic_uses_worst_need() {
        // Even if hunger is high, low rest should trigger warning
        let needs = Needs { hunger: 0.9, rest: 0.4 };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☻'); // Warning state
        assert_eq!(color, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_starve_from_full() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        // Run until pop dies
        let mut ticks = 0;
        while world.query::<&Pop>().iter(&world).count() > 0 && ticks < 100 {
            decay_needs_system(&mut world);
            kill_starving_pops_system(&mut world);
            ticks += 1;
        }

        assert!(ticks < 50, "Pop should die within ~50 ticks from full (0.8)");
        assert!(ticks > 30, "Pop should survive at least 30 ticks");
    }
}
```

**Test Coverage Requirements:**
- Needs: default values, worst() calculation
- Needs: clamping to [0.0, 1.0] range
- decay_needs_system: reduces both hunger and rest
- decay_needs_system: doesn't go negative
- kill_starving_pops_system: only kills when hunger == 0.0
- pop_display (basic version): characters and colors for health states
- pop_display (basic version): uses worst need for state determination
- Survival time: ~40 ticks from default (0.8) to death
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Needs Component

```rust
// src/layer1/needs.rs

use bevy_ecs::prelude::*;

/// Pop survival needs.
#[derive(Component, Clone, Copy, Debug)]
pub struct Needs {
    /// Hunger level: 0.0 = starving, 1.0 = full.
    pub hunger: f32,
    /// Rest level: 0.0 = exhausted, 1.0 = rested.
    pub rest: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.8,
            rest: 0.8,
        }
    }
}

impl Needs {
    /// Returns the worst (lowest) need value.
    #[must_use]
    pub fn worst(&self) -> f32 {
        self.hunger.min(self.rest)
    }
}
```

### Decay System

```rust
// src/layer1/needs.rs

use super::pop::Pop;

const HUNGER_DECAY_PER_TICK: f32 = 0.02; // ~50 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.01;   // ~100 ticks to exhaust

/// Decays needs for all pops each tick.
pub fn decay_needs_system(world: &mut World) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
    }
}
```

### Death System

```rust
// src/layer1/needs.rs

/// Despawns pops whose hunger has reached zero.
pub fn kill_starving_pops_system(world: &mut World) {
    // Collect entities to despawn (can't despawn while iterating)
    let to_despawn: Vec<Entity> = world
        .query_filtered::<(Entity, &Needs), With<Pop>>()
        .iter(world)
        .filter(|(_, needs)| needs.hunger <= 0.0)
        .map(|(entity, _)| entity)
        .collect();

    for entity in to_despawn {
        world.despawn(entity);
    }
}
```

### Visual Feedback (Basic Version)

```rust
// src/layer1/pop.rs - Replace pop_display function

use super::needs::Needs;
use ratatui::style::Color;

/// Returns the character and color for rendering a pop based on their needs.
///
/// **NOTE:** This is the basic version that only considers needs. Spec 009 will extend this
/// to also take PopState as a parameter to show different characters for working/resting pops.
/// The signature change from `pop_display(&Needs)` to `pop_display(&Needs, &PopState)` is
/// a forward-compatible extension.
#[must_use]
pub fn pop_display(needs: &Needs) -> (char, Color) {
    let health = needs.worst();
    if health > 0.6 {
        ('☺', Color::Yellow)       // Healthy
    } else if health > 0.3 {
        ('☻', Color::Rgb(255, 165, 0))  // Warning (orange)
    } else {
        ('☹', Color::Red)          // Critical
    }
}
```

### Update Pop Spawning

```rust
// src/layer1/pop.rs - Modify spawn_initial_pops

use super::needs::Needs;

pub fn spawn_initial_pops(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();
    let mut spawned = 0;

    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);

        if let Some(terrain_type) = terrain.get(x as usize, y as usize) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                world.spawn((
                    Pop,
                    GridPosition { x, y },
                    Needs::default(),  // ADD THIS
                ));
                spawned += 1;
            }
        }
    }
}
```

### Update Rendering

```rust
// src/layer1/terrain.rs - Modify render_terrain_and_pops signature

use super::pop::{GridPosition, pop_display};
use super::needs::Needs;

pub fn render_terrain_and_pops(
    frame: &mut Frame,
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
    pops_data: &[(GridPosition, (char, Color))],
) {
    let mut lines: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            // Check for pop first
            if let Some((_, (ch, color))) = pops_data.iter()
                .find(|(pos, _)| pos.x == world_x && pos.y == world_y)
            {
                line_spans.push(Span::styled(ch.to_string(), Style::default().fg(*color)));
                continue;
            }

            // Otherwise render terrain
            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black)
                }
            } else {
                (' ', Color::Black)
            };

            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(line_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

### Info Panel Update

```rust
// src/main.rs - Update render_info_panel

fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let pop_count = world.query::<&Pop>().iter(world).count();

    let text = format!(
        "Population: {}\n\n\
         Pops will starve\n\
         without food!",
        pop_count
    );
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}
```

### Main.rs Integration

```rust
// src/main.rs - Update render_map

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();

    // Collect pop data with visual state
    let pops_data: Vec<(GridPosition, (char, Color))> = world
        .query::<(&GridPosition, &Needs)>()
        .iter(world)
        .map(|(pos, needs)| (*pos, pop_display(needs)))
        .collect();

    render_terrain_and_pops(frame, inner, terrain, viewport, &pops_data);
}

// In main loop, after schedule.run():
if *world.resource::<GameState>() == GameState::Running {
    let speed = world.resource::<SimulationTime>().speed;
    if speed != SimSpeed::Paused {
        // ADD THESE SYSTEMS:
        decay_needs_system(&mut world);
        kill_starving_pops_system(&mut world);

        world.resource_mut::<SimulationTime>().tick += 1;
    }
}
```

### Module Integration

```rust
// src/layer1/mod.rs
pub mod terrain;
pub mod pop;
pub mod needs;

pub use terrain::{TerrainGrid, TerrainType, Viewport, generate_terrain};
pub use pop::{Pop, GridPosition, spawn_initial_pops, pop_display};
pub use needs::{Needs, decay_needs_system, kill_starving_pops_system};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Decay constants are magic numbers**: 0.02 and 0.01 hardcoded
   - Future: Extract to config file or resource
   - Current values tuned for ~40 tick survival, reasonable for testing

2. **Death only from hunger**: Rest doesn't kill pops
   - Design decision: Exhaustion slows productivity, starvation kills
   - Future spec could add "collapse from exhaustion" mechanic
   - Current: Keep death condition simple

3. **No need fulfillment**: Pops can't eat or sleep yet
   - Spec 007 (Housing) adds rest restoration
   - Spec 008 (Farm) adds food consumption
   - Current: Intentional - establishes doom timer first

4. **pop_display signature will change**: Forward compatibility note added
   - Spec 009 will extend to `pop_display(&Needs, &PopState)`
   - Current code documents this coming change
   - Tests will need updating when PopState added

5. **Health thresholds hardcoded**: 0.6 and 0.3 for visual states
   - Future: Extract to constants or config
   - Current: Values chosen for clear visual feedback

### Performance Considerations

- **Decay system is O(n)**: Iterates all pops once
- **Death system is O(n)**: Iterates all pops, despawns subset
- **Two-pass for death**: Collect first, then despawn (can't modify while iterating)
- **Linear search in rendering**: Finding pops by position is O(n*m) where n=pops, m=screen
  - Acceptable for <100 pops
  - Future: Spatial index if performance becomes issue

### API Design Notes

- `Needs` is Copy - cheap to pass, no lifetime issues
- `worst()` method - single source of truth for "health" calculation
- System functions take `&mut World` - allows querying any resources/components
- Constants at module level - easy to tune via find/replace

### Future Extensibility

When adding food consumption (Spec 008):
```rust
// New system in spec 008
fn consume_food_system(world: &mut World) {
    // Check hunger threshold
    // Consume food from stockpile
    // Restore hunger need
}
```

When adding rest restoration (Spec 007):
```rust
// New system in spec 007
fn restore_rest_in_housing_system(world: &mut World) {
    // Query housing residents
    // Restore rest need
}
```

When extending pop_display (Spec 009):
```rust
// Change signature
pub fn pop_display(needs: &Needs, state: &PopState) -> (char, Color) {
    let health = needs.worst();
    let (base_char, base_color) = /* health-based logic */;

    // Override character based on state
    let ch = match state {
        PopState::Idle => base_char,
        PopState::Working(_) => '⚒',
        PopState::Resting(_) => '☻',
    };

    (ch, base_color)
}
```

### System Execution Order

**Current (Spec 005):**
1. decay_needs_system
2. kill_starving_pops_system

**Future (with Spec 007-008):**
1. decay_needs_system
2. consume_food_system  (Spec 008)
3. restore_rest_in_housing_system  (Spec 007)
4. kill_starving_pops_system
5. cleanup_dead_references_system  (Spec 007-008)

Order matters: Decay first, then fulfillment, then death, then cleanup.

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer1/needs.rs
- [x] Pops have `Needs` component with hunger and rest
- [x] Needs decay each game tick when not paused
- [x] Pop character changes as health drops (☺ → ☻ → ☹)
- [x] Pop color shifts toward orange/red as needs drop
- [x] Pop despawns when hunger reaches 0.0
- [x] Starting pops survive ~40 ticks before dying
- [x] Info panel shows population count
- [x] Rest need exists but has no fulfillment yet (intentional)

## Technical Guidance

### Need Value Ranges

- **1.0 - 0.6**: Healthy (green/yellow indicators)
- **0.6 - 0.3**: Warning (orange indicators)
- **0.3 - 0.0**: Critical (red indicators)
- **0.0**: Death (hunger only)

### Decay Rates

With default starting value of 0.8:
- **Hunger**: 0.02 per tick → dies in ~40 ticks
- **Rest**: 0.01 per tick → exhausts in ~80 ticks (but doesn't kill)

These values create urgency without being punishing for new players.

### Visual Feedback States

| Worst Need | Character | Color | Meaning |
|------------|-----------|-------|---------|
| > 0.6 | ☺ | Yellow | Healthy |
| 0.3 - 0.6 | ☻ | Orange | Warning |
| < 0.3 | ☹ | Red | Critical |

**Design note:** Uses worst need, not average. A pop with hunger=1.0 and rest=0.2 shows as critical (rest is limiting factor).

### Common Pitfalls

1. **Forgetting max(0.0)**: Needs can decay negative without clamping
2. **Despawning while iterating**: Must collect entities first, despawn second
3. **Testing with GameState::Paused**: Systems don't run when paused
4. **Expecting pops to survive**: With no food/rest, all pops die - this is intentional!

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

## Forward Compatibility Note

**pop_display Signature Evolution:**

- **Spec 005 (this spec)**: `pop_display(needs: &Needs) -> (char, Color)`
- **Spec 009 (future)**: `pop_display(needs: &Needs, state: &PopState) -> (char, Color)`

The function signature will be extended (not replaced) to add PopState parameter. This is a backward-compatible change - all existing calls just need to add the new parameter. Tests in this spec use the basic version; Spec 009 tests will validate the extended version.
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.

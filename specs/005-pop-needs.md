# 005: Pop Needs (Hunger, Rest)

## Overview

Pops have needs that decay over time. Unmet needs eventually kill them. This creates the core survival pressure of the colony.

## Dependencies

- `003` — UI layout (SimulationTime for ticks)
- `004` — Pop entity exists

## Requirements

### Must Have
- Hunger need: 0.0 (starving) to 1.0 (full)
- Rest need: 0.0 (exhausted) to 1.0 (rested)
- Needs decay each simulation tick
- Pop dies (despawns) when hunger reaches 0.0
- Visual indicator: pop character changes based on health
- New pops spawn with needs at 0.8
- Info panel shows pop count

### Must NOT Have
- Need fulfillment (eating/sleeping)—requires buildings
- Morale or other needs
- Pop reproduction

## Technical Guidance

### Components

```rust
#[derive(Component)]
pub struct Needs {
    pub hunger: f32,  // 0.0 = starving, 1.0 = full
    pub rest: f32,    // 0.0 = exhausted, 1.0 = rested
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
    pub fn worst(&self) -> f32 {
        self.hunger.min(self.rest)
    }
}
```

### Update Pop Spawning

Modify spec 004's spawn to include Needs:

```rust
world.spawn((
    Pop,
    GridPosition { x, y },
    Needs::default(),
));
```

### Decay System

```rust
const HUNGER_DECAY_PER_TICK: f32 = 0.02;  // ~50 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.01;    // ~100 ticks to exhaust

fn decay_needs_system(world: &mut World) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
    }
}
```

### Death System

```rust
fn kill_starving_pops_system(world: &mut World) {
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

### Visual Feedback

Change pop character based on health:

```rust
fn pop_display(needs: &Needs) -> (char, Color) {
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

Update rendering to use this:

```rust
// In render, when collecting pop data:
let mut pops_data = Vec::new();
let mut query_state = world.query::<(&GridPosition, &Needs)>().with::<Pop>();
for (pos, needs) in query_state.iter(world) {
    pops_data.push((*pos, pop_display(needs)));
}

// Then in render loop:
if let Some((_, (ch, color))) = pops_data.iter().find(|(p, _)| p.x == world_x && p.y == world_y) {
    spans.push(Span::styled(ch.to_string(), Style::default().fg(*color)));
    continue;
}
```

### Info Panel Update

Show pop count in info panel:

```rust
fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
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

### System Execution

Add to main loop or schedule:

```rust
// Only run simulation systems when not paused
if sim_time.speed != SimSpeed::Paused {
    decay_needs_system(&mut world);
    kill_starving_pops_system(&mut world);
}
```

## Acceptance Criteria

- [ ] Pops have `Needs` component with hunger and rest
- [ ] Needs decay each game tick
- [ ] Pop character changes as health drops (☺ → ☻ → ☹)
- [ ] Pop color shifts toward red as needs drop
- [ ] Pop despawns when hunger reaches 0
- [ ] Starting pops don't immediately die (needs start at 0.8)
- [ ] With no intervention, all pops die within ~50 ticks
- [ ] Info panel shows population count
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*

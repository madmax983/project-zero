# 006: Pop Needs (Hunger, Rest)

## Overview

Pops have needs that decay over time. Unmet needs eventually kill them. This creates the core survival pressure of the colony.

## Dependencies

- `004` — Pop entity exists
- `005` — Time system (needs decay on tick)

## Requirements

### Must Have
- Hunger need: 0.0 (starving) to 1.0 (full)
- Rest need: 0.0 (exhausted) to 1.0 (rested)
- Needs decay each tick
- Pop dies when hunger reaches 0.0
- Visual indicator: pop color shifts based on worst need
- New pops spawn with needs at 0.8

### Must NOT Have
- Need fulfillment (eating/sleeping)—that requires buildings (future specs)
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

### Decay System

```rust
const HUNGER_DECAY_PER_TICK: f32 = 0.02;  // ~50 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.01;    // ~100 ticks to exhaust

fn decay_needs(
    mut tick_events: EventReader<TickEvent>,
    mut query: Query<&mut Needs, With<Pop>>,
) {
    for _ in tick_events.read() {
        for mut needs in &mut query {
            needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
            needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
        }
    }
}
```

### Death System

```rust
fn kill_starving_pops(
    mut commands: Commands,
    query: Query<(Entity, &Needs), With<Pop>>,
) {
    for (entity, needs) in &query {
        if needs.hunger <= 0.0 {
            commands.entity(entity).despawn();
            // Future: spawn grave, log death, etc.
        }
    }
}
```

### Visual Feedback

Color lerp based on worst need:

```rust
fn update_pop_visuals(
    mut query: Query<(&Needs, &mut Sprite), With<Pop>>,
) {
    for (needs, mut sprite) in &mut query {
        let health = needs.worst();
        // Lerp from red (dying) to normal flesh tone (healthy)
        let r = 0.9;
        let g = 0.3 + (health * 0.4); // 0.3 when dying, 0.7 when healthy
        let b = 0.3 + (health * 0.2);
        sprite.color = Color::srgb(r, g, b);
    }
}
```

## Acceptance Criteria

- [ ] Pops have `Needs` component with hunger and rest
- [ ] Needs decay each game tick
- [ ] Pop color shifts toward red as needs drop
- [ ] Pop despawns when hunger reaches 0
- [ ] Starting pops don't immediately die (needs start at 0.8)
- [ ] With no intervention, all pops die within ~50 ticks
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*

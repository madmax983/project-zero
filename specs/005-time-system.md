# 005: Time System (Tick, Pause, Speed)

## Overview

The colony simulation runs on discrete ticks. This system controls the game clock—pausing, resuming, and adjusting simulation speed.

## Dependencies

- `001` — Project scaffold (GameState enum)

## Requirements

### Must Have
- Discrete tick system (not continuous)
- Default: 1 tick per second (real-time)
- Speed controls: pause (0x), normal (1x), fast (3x), faster (5x)
- Spacebar toggles pause
- 1/2/3 keys set speed (normal/fast/faster)
- Visual indicator of current speed somewhere on screen
- Tick counter resource for other systems to read

### Must NOT Have
- Reverse time
- Frame-by-frame stepping (nice to have later)

## Technical Guidance

### Resources

```rust
#[derive(Resource)]
pub struct SimulationTime {
    pub tick: u64,
    pub speed: SimSpeed,
    pub accumulator: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SimSpeed {
    Paused,
    #[default]
    Normal,
    Fast,
    Faster,
}

impl SimSpeed {
    pub fn ticks_per_second(&self) -> f32 {
        match self {
            SimSpeed::Paused => 0.0,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast => 3.0,
            SimSpeed::Faster => 5.0,
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            SimSpeed::Paused => "⏸ PAUSED",
            SimSpeed::Normal => "▶ 1x",
            SimSpeed::Fast => "▶▶ 3x",
            SimSpeed::Faster => "▶▶▶ 5x",
        }
    }
}
```

### Tick System

```rust
fn advance_simulation_time(
    mut sim_time: ResMut<SimulationTime>,
    time: Res<Time>,
    mut tick_events: EventWriter<TickEvent>,
) {
    let tps = sim_time.speed.ticks_per_second();
    if tps == 0.0 {
        return;
    }
    
    sim_time.accumulator += time.delta_secs();
    let tick_duration = 1.0 / tps;
    
    while sim_time.accumulator >= tick_duration {
        sim_time.accumulator -= tick_duration;
        sim_time.tick += 1;
        tick_events.send(TickEvent(sim_time.tick));
    }
}

#[derive(Event)]
pub struct TickEvent(pub u64);
```

### Input Handling

```rust
fn handle_time_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sim_time: ResMut<SimulationTime>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        sim_time.speed = if sim_time.speed == SimSpeed::Paused {
            SimSpeed::Normal
        } else {
            SimSpeed::Paused
        };
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        sim_time.speed = SimSpeed::Normal;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        sim_time.speed = SimSpeed::Fast;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        sim_time.speed = SimSpeed::Faster;
    }
}
```

### UI Indicator

Simple text in corner. Use Bevy's built-in text rendering:

```rust
#[derive(Component)]
struct SpeedIndicator;

fn spawn_speed_indicator(mut commands: Commands) {
    commands.spawn((
        SpeedIndicator,
        Text::new("▶ 1x"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));
}

fn update_speed_indicator(
    sim_time: Res<SimulationTime>,
    mut query: Query<&mut Text, With<SpeedIndicator>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        **text = sim_time.speed.label().to_string();
    }
}
```

## Acceptance Criteria

- [ ] Game starts running at 1 tick/second
- [ ] Spacebar pauses and unpauses
- [ ] 1/2/3 keys change speed
- [ ] Speed indicator visible in corner
- [ ] Speed indicator updates when speed changes
- [ ] `TickEvent` fires each simulation tick
- [ ] `cargo check` passes

## Questions

*Builder: add questions here if spec is unclear.*

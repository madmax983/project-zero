# 1098: Psychoactive Weather

## Overview

A Layer 1 feature where rare weather events (e.g., Spore Storms, Neon Rain) apply mental status effects to Pops instead of physical damage. Pops might become Inspired, Paranoid, or Euphoric, drastically altering productivity, combat stats, or behavior, forcing players to choose between sheltering workers or embracing the mood buffs.

## Dependencies

- Layer 1 Weather system.
- Layer 1 Pop Status/Moodlet system.

## RED Phase: Tests First

```rust
#[test]
fn test_psychoactive_weather_applies_moodlets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, apply_weather_moodlets_system);

    // Arrange: A Pop outdoors during a Bliss Storm
    app.world_mut().insert_resource(CurrentWeather(WeatherType::BlissStorm));
    let pop = app.world_mut().spawn((Pop, Outdoors, Mood::default())).id();

    app.update();

    // Assert: Pop gains the Euphoric moodlet
    let mood = app.world().get::<Mood>(pop).unwrap();
    assert!(mood.has_moodlet(Moodlet::Euphoric));
}

#[test]
fn test_indoors_prevents_psychoactive_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, apply_weather_moodlets_system);

    // Arrange: A Pop indoors during a Spore Storm
    app.world_mut().insert_resource(CurrentWeather(WeatherType::SporeStorm));
    let pop = app.world_mut().spawn((Pop, Indoors, Mood::default())).id();

    app.update();

    // Assert: Pop does not gain the Paranoid moodlet
    let mood = app.world().get::<Mood>(pop).unwrap();
    assert!(!mood.has_moodlet(Moodlet::Paranoid));
}

#[test]
fn test_euphoria_halts_work_but_boosts_morale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, process_euphoria_effects_system);

    // Arrange: A Euphoric Pop assigned to a job
    let mut starting_mood = Mood::default();
    starting_mood.add(Moodlet::Euphoric);
    let pop = app.world_mut().spawn((
        Pop,
        starting_mood,
        Job { progress: 0.0, base_speed: 1.0 },
        Morale { value: 50 },
    )).id();

    app.update();

    // Assert: Work progress is 0 (halted), Morale is boosted
    assert_eq!(app.world().get::<Job>(pop).unwrap().progress, 0.0);
    assert!(app.world().get::<Morale>(pop).unwrap().value > 50);
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct CurrentWeather(pub WeatherType);

#[derive(PartialEq)]
pub enum WeatherType {
    Clear,
    BlissStorm,
    SporeStorm,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Outdoors;

#[derive(Component)]
pub struct Indoors;

#[derive(PartialEq, Clone, Copy)]
pub enum Moodlet {
    Euphoric,
    Paranoid,
}

#[derive(Component, Default)]
pub struct Mood {
    pub active_moodlets: Vec<Moodlet>,
}

impl Mood {
    pub fn has_moodlet(&self, moodlet: Moodlet) -> bool {
        self.active_moodlets.contains(&moodlet)
    }
    pub fn add(&mut self, moodlet: Moodlet) {
        if !self.has_moodlet(moodlet) {
            self.active_moodlets.push(moodlet);
        }
    }
}

pub fn apply_weather_moodlets_system(
    weather: Option<Res<CurrentWeather>>,
    mut outdoor_pops: Query<&mut Mood, (With<Pop>, With<Outdoors>, Without<Indoors>)>,
) {
    if let Some(w) = weather {
        let moodlet_to_apply = match w.0 {
            WeatherType::BlissStorm => Some(Moodlet::Euphoric),
            WeatherType::SporeStorm => Some(Moodlet::Paranoid),
            WeatherType::Clear => None,
        };

        if let Some(moodlet) = moodlet_to_apply {
            for mut mood in outdoor_pops.iter_mut() {
                mood.add(moodlet);
            }
        }
    }
}

#[derive(Component)]
pub struct Job {
    pub progress: f32,
    pub base_speed: f32,
}

#[derive(Component)]
pub struct Morale {
    pub value: i32,
}

pub fn process_euphoria_effects_system(
    mut pop_query: Query<(&Mood, &mut Job, &mut Morale), With<Pop>>,
) {
    for (mood, mut job, mut morale) in pop_query.iter_mut() {
        if mood.has_moodlet(Moodlet::Euphoric) {
            // Work halts
            job.progress += 0.0;
            // Morale boost (simplified, usually would be a gradual increase or a flat buff layer)
            morale.value += 10;
        } else {
            job.progress += job.base_speed;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Moodlet Expiration**: Moodlets currently persist forever. They need a duration/timer to wear off after the storm passes.
- **Gas Masks**: Hardcoding `Without<Indoors>` prevents the use of protective gear outdoors. Introduce an `AtmosphereSealed` or `HasGasMask` component that both Indoors and protective suits provide.
- **Morale Stacking**: The current implementation adds +10 Morale *every frame* for Euphoria, which will overflow instantly. Moodlets should act as static modifiers calculated on demand, not continuous additive ticks.

## Acceptance Criteria

- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for Psychoactive Weather logic.
- [ ] Pops outdoors during specific weather gain expected moodlets, altering their work/stats.

## Technical Guidance

- Utilize the existing `Modifier` pattern if one exists for stats/morale to prevent the per-tick accumulation bug.
- Ensure the Weather system can actually transition into these rare states during the game loop.

## Questions

*Builder: add questions here if spec is unclear.*

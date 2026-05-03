# Specification: 1250 Corrosive Atmosphere

## 1. Overview
The air hates you. Metal screams when it touches the wind.

## 2. Dependencies
- Environment/Atmosphere
- Buildings/Equipment degradation

## 3. RED Phase: Tests First
```rust
#[test]
fn test_corrosion_degrades_buildings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(CorrosionPlugin);

    // Arrange
    let building = app.world_mut().spawn((Building, Health { current: 100.0, max: 100.0 })).id();
    app.world_mut().insert_resource(Atmosphere { corrosive_level: 0.5 });

    // Act
    app.update();

    // Assert
    assert!(app.world().get::<Health>(building).unwrap().current < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Resource)]
pub struct Atmosphere {
    pub corrosive_level: f32,
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

pub struct CorrosionPlugin;

impl Plugin for CorrosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_corrosion_system);
    }
}

fn apply_corrosion_system(
    atmosphere: Option<Res<Atmosphere>>,
    mut query: Query<&mut Health, With<Building>>,
) {
    if let Some(atm) = atmosphere {
        if atm.corrosive_level > 0.0 {
            for mut health in query.iter_mut() {
                health.current -= atm.corrosive_level;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate corrosion based on exposure
- Use Time resource for degradation over time

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Handle edge cases around zero or negative corrosive levels

## 8. Questions
*Builder: add questions here if spec is unclear.*

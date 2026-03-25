# The Subterranean Smog Layer

**1. Overview**
Building downwards traps your colony in its own pollution, forcing desperate ventilation measures. Heavy industrial buildings (smelters, refineries) produce a physical "Smog" entity that obeys gravity and sinks into lower Z-levels. If these levels aren't actively vented to the surface, the smog displaces oxygen, slowly suffocating Pops and corrupting crops.

**2. Dependencies**
- `layer1::map::TerrainGrid`
- `layer1::needs::Needs`
- `layer1::buildings::HeavyIndustry`

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_smog_generation() {
        let mut app = App::new();
        app.add_systems(Update, generate_smog_system);

        let industry = app.world_mut().spawn((
            HeavyIndustry { active: true },
            Position { x: 5, y: 5, z: -1 },
        )).id();

        app.update();

        // Ensure a smog entity was generated at or below the industry
        let mut smog_query = app.world_mut().query::<(&Smog, &Position)>();
        let mut found_smog = false;
        for (_, pos) in smog_query.iter(app.world()) {
            if pos.z <= -1 {
                found_smog = true;
            }
        }
        assert!(found_smog, "Smog should be generated at or below active heavy industry");
    }

    #[test]
    fn test_smog_suffocates_pops() {
        let mut app = App::new();
        app.add_systems(Update, apply_smog_effects_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { oxygen: 100.0, ..default() },
            Position { x: 5, y: 5, z: -2 },
        )).id();

        app.world_mut().spawn((
            Smog { density: 1.0 },
            Position { x: 5, y: 5, z: -2 },
        ));

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.oxygen < 100.0, "Smog should decrease oxygen for Pops in the same position");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct HeavyIndustry {
    pub active: bool,
}

#[derive(Component, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct Smog {
    pub density: f32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component, Default)]
pub struct Needs {
    pub oxygen: f32,
}

pub fn generate_smog_system(
    mut commands: Commands,
    query: Query<(&HeavyIndustry, &Position)>,
) {
    for (industry, pos) in query.iter() {
        if industry.active {
            commands.spawn((
                Smog { density: 0.1 },
                Position { x: pos.x, y: pos.y, z: pos.z - 1 }, // Simplistic: just spawn below
            ));
        }
    }
}

pub fn apply_smog_effects_system(
    mut pops: Query<(&mut Needs, &Position), With<Pop>>,
    smogs: Query<(&Smog, &Position)>,
) {
    for (mut needs, pop_pos) in pops.iter_mut() {
        for (smog, smog_pos) in smogs.iter() {
            if pop_pos.x == smog_pos.x && pop_pos.y == smog_pos.y && pop_pos.z == smog_pos.z {
                needs.oxygen -= smog.density;
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Implement a proper fluid dynamics grid to allow smog to diffuse and settle realistically rather than just spawning static entities.
- Add vents as a building type that actively removes `Smog` components or changes their diffusion path.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Heavy industries generate `Smog` components on lower Z-levels.
- [ ] `Smog` reduces the oxygen `Needs` of overlapping Pops.

**7. Technical Guidance**
- Utilize the existing `Grid` or `Map` systems if available to handle the spatial queries efficiently instead of O(N^2) iterations.

**8. Questions**
*Builder: add questions here if spec is unclear.*

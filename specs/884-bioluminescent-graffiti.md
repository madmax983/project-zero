# 884: The Bioluminescent Graffiti

## 1. Overview
The lower classes use genetically modified flora to express their discontent in the dark. Pops with high unrest but low power plant a slow-growing, glowing fungus in unpatrolled corridors. The fungus forms anti-establishment symbols that boost the morale of the lower class but anger the enforcers. Eradicating the fungus requires toxic chemicals that lower air quality, but leaving it empowers a passive rebellion.

## 2. Dependencies
- `050-civil-unrest.md` (Unrest mechanic)
- `053-lighting-system.md` (Light levels)
- `144-graffiti-and-signage.md` (Graffiti foundation)
- `113-social-stratification.md` (Social classes)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bioluminescent_graffiti_spawn() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Add required components/systems

        let tile = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            LightLevel(0.0), // Dark unpatrolled corridor
        )).id();

        let pop = app.world_mut().spawn((
            Pop,
            SocialClass::Labor,
            Unrest(80.0), // High unrest
            GridPosition { x: 0, y: 0 },
        )).id();

        // Act
        app.update();

        // Assert
        let graffiti_query = app.world_mut().query::<&BioluminescentGraffiti>();
        assert_eq!(graffiti_query.iter(app.world()).count(), 1, "Graffiti should spawn in dark tiles by unhappy lower class");
    }

    #[test]
    fn test_bioluminescent_graffiti_morale_effect() {
        // Arrange
        let mut app = App::new();
        // Setup world with graffiti
        app.world_mut().spawn((
            BioluminescentGraffiti,
            GridPosition { x: 0, y: 0 },
        ));

        let labor_pop = app.world_mut().spawn((
            Pop,
            SocialClass::Labor,
            Morale(50.0),
            GridPosition { x: 1, y: 0 },
        )).id();

        let elite_pop = app.world_mut().spawn((
            Pop,
            SocialClass::Elite, // Enforcer/Elite
            Morale(50.0),
            GridPosition { x: 1, y: 0 },
        )).id();

        // Act
        app.update(); // Apply aura effects

        // Assert
        let labor_morale = app.world().get::<Morale>(labor_pop).unwrap().0;
        let elite_morale = app.world().get::<Morale>(elite_pop).unwrap().0;
        assert!(labor_morale > 50.0, "Lower class should gain morale");
        assert!(elite_morale < 50.0, "Upper class/Enforcers should lose morale");
    }

    #[test]
    fn test_graffiti_eradication_air_quality() {
        // Arrange
        let mut app = App::new();
        let graffiti = app.world_mut().spawn((
            BioluminescentGraffiti,
            GridPosition { x: 0, y: 0 },
        )).id();

        let tile = app.world_mut().spawn((
            GridPosition { x: 0, y: 0 },
            AirQuality(100.0),
        )).id();

        // Act
        // Trigger eradication action (e.g. chemical spray)
        app.world_mut().send_event(EradicateGraffitiEvent { target: graffiti });
        app.update();

        // Assert
        assert!(app.world().get_entity(graffiti).is_err(), "Graffiti should be destroyed");
        let air_quality = app.world().get::<AirQuality>(tile).unwrap().0;
        assert!(air_quality < 100.0, "Air quality should drop after eradication");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct BioluminescentGraffiti;

#[derive(Event)]
pub struct EradicateGraffitiEvent {
    pub target: Entity,
}

pub fn spawn_bioluminescent_graffiti_system(
    mut commands: Commands,
    pop_query: Query<(&SocialClass, &Unrest, &GridPosition), With<Pop>>,
    tile_query: Query<(&LightLevel, &GridPosition)>,
) {
    // Check for high unrest labor pops in dark tiles, spawn graffiti
}

pub fn graffiti_morale_aura_system(
    graffiti_query: Query<&GridPosition, With<BioluminescentGraffiti>>,
    mut pop_query: Query<(&SocialClass, &mut Morale, &GridPosition), With<Pop>>,
) {
    // Apply morale buff to nearby Labor pops and debuff to Elite pops
}

pub fn eradicate_graffiti_system(
    mut commands: Commands,
    mut events: EventReader<EradicateGraffitiEvent>,
    mut tile_query: Query<(&GridPosition, &mut AirQuality)>,
    graffiti_query: Query<&GridPosition, With<BioluminescentGraffiti>>,
) {
    // Despawn graffiti and reduce air quality on the tile
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with existing `Graffiti` and `Aura` components instead of creating entirely new parallel systems if possible.
- Ensure the eradication uses the existing `ActionType::Clean` or similar utility AI tasks rather than just an event.
- Consider making the "glow" actually emit a small amount of `LightLevel` so it serves a dual purpose.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Graffiti spawns appropriately based on class and light level
- [ ] Morale is correctly affected based on class
- [ ] Eradication lowers air quality

## 7. Technical Guidance
- Look at `src/layer1/graffiti.rs` (if it exists from 144) to extend it with the bioluminescent specific logic.
- Tie the `EradicateGraffitiEvent` into the Utility AI so enforcers automatically try to clean it if they have the tools.

## 8. Questions
*Builder: add questions here if spec is unclear.*

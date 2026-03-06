# Specification: Architectural Eras

## 1. Overview
**Layer:** 1 (Colony)
**Fantasy:** Your city tells its age through its skyline.
**Mechanic:** Buildings are stamped with the "Era" they were built in. Old buildings retain old stats (worse efficiency) but gain "Heritage" or "Charm". New buildings are efficient but "Soulless".
**Emergence:** The player refuses to demolish the inefficient "First Hut" because it provides a massive morale aura as a historical site.
**Tension:** Demolish the past for efficiency, or preserve it for culture?

This feature tracks the era a structure was built in. As time progresses, older buildings lose efficiency but begin to emit a positive `MoraleAura` based on their `Heritage` level.

## 2. Dependencies
- `Structure` (Base building data)
- `Efficiency` component
- `Chronicle` or Global Time (to track eras)
- `MoraleAura` component (from morale/culture specs)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_heritage_increases_over_time() {
        let mut app = App::new();
        app.add_systems(Update, calculate_heritage_system);

        // Spawn a building built in Era 0
        let entity = app.world.spawn((
            Structure { id: 1 },
            ArchitecturalEra { built_era: 0, heritage_score: 0.0 }
        )).id();

        // Advance global era to 2
        app.world.insert_resource(GlobalEra { current_era: 2 });
        app.update();

        let era = app.world.get::<ArchitecturalEra>(entity).unwrap();
        assert!(era.heritage_score > 0.0, "Heritage score should increase as eras pass");
    }

    #[test]
    fn test_heritage_grants_morale_aura_but_lowers_efficiency() {
        let mut app = App::new();
        app.add_systems(Update, apply_heritage_effects_system);

        let entity = app.world.spawn((
            Structure { id: 1 },
            ArchitecturalEra { built_era: 0, heritage_score: 100.0 },
            Efficiency { base: 1.0, current: 1.0 },
            MoraleAura { radius: 0, boost: 0 }
        )).id();

        app.update();

        let eff = app.world.get::<Efficiency>(entity).unwrap();
        let aura = app.world.get::<MoraleAura>(entity).unwrap();

        assert!(eff.current < 1.0, "Old buildings should lose efficiency");
        assert!(aura.radius > 0 && aura.boost > 0, "Old buildings should gain morale aura");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct GlobalEra {
    pub current_era: u32,
}

#[derive(Component)]
pub struct ArchitecturalEra {
    pub built_era: u32,
    pub heritage_score: f32,
}

#[derive(Component)]
pub struct Efficiency {
    pub base: f32,
    pub current: f32,
}

#[derive(Component, Default)]
pub struct MoraleAura {
    pub radius: i32,
    pub boost: i32,
}

pub fn calculate_heritage_system(
    global_era: Res<GlobalEra>,
    mut query: Query<&mut ArchitecturalEra>,
) {
    for mut era in query.iter_mut() {
        let age = global_era.current_era.saturating_sub(era.built_era);
        // Simple scaling: 10 heritage per era of age
        era.heritage_score = (age * 10) as f32;
    }
}

pub fn apply_heritage_effects_system(
    mut query: Query<(&ArchitecturalEra, &mut Efficiency, &mut MoraleAura)>,
) {
    for (era, mut eff, mut aura) in query.iter_mut() {
        if era.heritage_score > 50.0 {
            // Very old building
            eff.current = eff.base * 0.8; // 20% penalty
            aura.radius = 5;
            aura.boost = 2; // +2 Morale
        } else if era.heritage_score > 0.0 {
            // Slightly old
            eff.current = eff.base * 0.9;
            aura.radius = 2;
            aura.boost = 1;
        } else {
            // Modern building
            eff.current = eff.base;
            aura.radius = 0;
            aura.boost = 0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Era Transitions:** Add an event `EraAdvancedEvent` to trigger heritage recalculation only when eras change instead of every frame.
- **Dynamic Stats:** Hook the efficiency drop into the existing `modifier` stack for `Efficiency` instead of overriding `current` directly.
- **Building Types:** Some buildings (like monuments) shouldn't lose efficiency, only gain heritage. Add a tag `HistoricalExempt` if needed.

## 6. Acceptance Criteria
- [ ] `ArchitecturalEra` components record `built_era` correctly.
- [ ] `calculate_heritage_system` updates heritage scores based on `GlobalEra`.
- [ ] Heritage reduces building efficiency.
- [ ] Heritage provides a `MoraleAura` buff.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **Global Era Tracking:** Increment `GlobalEra` either based on total game ticks (e.g., every 100 days) or through technology unlocks.
- **Aura Stacking:** Ensure that overlapping `MoraleAura` from multiple heritage sites don't break the morale balance. Use `max` instead of `sum` in the morale system if necessary.

## 8. Questions
*Builder: Add any questions here.*

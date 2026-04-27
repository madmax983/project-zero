# The Vanguard Ram

## 1. Overview
**Layer:** 2
**Fantasy:** A suicidal, heavily armored spearhead designed not to shoot, but to physically break enemy formations.
**Mechanic:** A ship with no conventional weapons but incredible frontal armor and a massive engine. Its only attack is a direct kinetic strike. Upon impact, it deals devastating damage to capital ships or orbital stations, but the ship itself is almost always destroyed, and the impact creates a lethal, localized debris cloud.

## 2. Dependencies
- Layer 2 Fleet Combat System
- Debris Cascade Mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_vanguard_ram_kinetic_strike() {
        // Arrange
        let mut app = App::new();
        app.add_event::<KineticStrikeEvent>();

        let target = app.world_mut().spawn(Ship { health: 1000.0 }).id();
        let ram = app.world_mut().spawn(VanguardRam).id();

        // Act
        app.world_mut().send_event(KineticStrikeEvent { attacker: ram, target });
        app.add_systems(Update, resolve_kinetic_strike_system);
        app.update();

        // Assert
        let target_ship = app.world().get::<Ship>(target).unwrap();
        assert!(target_ship.health < 1000.0, "Target should take massive damage");
        assert!(app.world().get::<VanguardRam>(ram).is_none(), "Vanguard Ram should be destroyed on impact");
    }

    #[test]
    fn test_ram_impact_creates_debris() {
        // Arrange
        let mut app = App::new();
        app.add_event::<KineticStrikeEvent>();
        app.init_resource::<OrbitalDebrisField>();

        let target = app.world_mut().spawn(Ship { health: 1000.0 }).id();
        let ram = app.world_mut().spawn(VanguardRam).id();

        app.world_mut().send_event(KineticStrikeEvent { attacker: ram, target });

        // Act
        app.add_systems(Update, resolve_kinetic_strike_system);
        app.update();

        // Assert
        let debris = app.world().resource::<OrbitalDebrisField>();
        assert!(debris.density > 0.0, "Impact should generate localized debris");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct VanguardRam;

#[derive(Component)]
pub struct Ship {
    pub health: f32,
}

#[derive(Event)]
pub struct KineticStrikeEvent {
    pub attacker: Entity,
    pub target: Entity,
}

#[derive(Resource, Default)]
pub struct OrbitalDebrisField {
    pub density: f32,
}

pub fn resolve_kinetic_strike_system(
    mut events: EventReader<KineticStrikeEvent>,
    mut commands: Commands,
    mut ship_query: Query<&mut Ship>,
    mut debris_field: ResMut<OrbitalDebrisField>,
) {
    for ev in events.read() {
        // Deal massive damage to target
        if let Ok(mut target_ship) = ship_query.get_mut(ev.target) {
            target_ship.health -= 800.0; // Arbitrary high damage value
        }

        // Destroy the ram
        commands.entity(ev.attacker).despawn();

        // Generate debris
        debris_field.density += 25.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Damage dealt should be proportional to the speed/mass of the Ram and the armor of the target.
- Debris generation should spawn at the specific physical coordinates of the impact, not just a global resource.
- Add hit-chance calculations, as small nimble ships might evade the ram.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Kinetic Strike destroys the attacking Ram entity.
- [ ] Target takes massive damage.
- [ ] Impact generates orbital debris.

## 7. Technical Guidance
- **Movement:** You may need to create a special behavior in the combat AI to prioritize rushing the biggest target rather than holding formation.
- **Debris:** Hook into the existing `OrbitalDebrisField` or `DebrisCascade` systems to ensure the aftermath is fully integrated.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

# The Orbital Mirror Misalignment

**1. Overview**
Playing with stellar fire and accidentally burning the house down. You construct massive "Orbital Mirrors" in Layer 2 to reflect extra sunlight onto Layer 1, boosting agricultural output and solar power. However, if the mirrors suffer micro-meteor impacts or sabotage, they can misalign. A misaligned mirror concentrates the beam into a devastating "Sun-Laser" that sweeps across the colony, instantly incinerating buildings and Pops in its path. The tension lies in balancing the immense, free energy and agricultural boom of orbital manipulation vs. the terrifying, ever-present risk of literally microwaving your own colony.

**2. Dependencies**
- `layer2::orbit::OrbitalStructures`
- `layer1::power::SolarGrid`
- `layer1::agriculture::FarmOutput`
- `layer1::disasters::EnvironmentalHazard`

**3. RED Phase: Tests First**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_mirror_boosts_output() {
        let mut app = App::new();
        app.add_systems(Update, orbital_mirror_buff_system);

        // Arrange
        app.world_mut().spawn((OrbitalMirror { aligned: true }));
        let farm = app.world_mut().spawn((Farm { base_output: 10, current_output: 10 })).id();

        // Act
        app.update();

        // Assert
        let farm_state = app.world().get::<Farm>(farm).unwrap();
        assert!(farm_state.current_output > farm_state.base_output, "Aligned orbital mirror should boost farm output");
    }

    #[test]
    fn test_mirror_misalignment_event() {
        let mut app = App::new();
        app.add_event::<MeteorImpactEvent>();
        app.add_systems(Update, mirror_impact_system);

        // Arrange
        let mirror = app.world_mut().spawn((OrbitalMirror { aligned: true })).id();

        // Act
        app.world_mut().resource_mut::<Events<MeteorImpactEvent>>().send(MeteorImpactEvent { target: mirror });
        app.update();

        // Assert
        let mirror_state = app.world().get::<OrbitalMirror>(mirror).unwrap();
        assert!(!mirror_state.aligned, "Meteor impact should misalign the mirror");
    }

    #[test]
    fn test_misaligned_mirror_causes_damage() {
        let mut app = App::new();
        app.add_event::<SunLaserDamageEvent>();
        app.add_systems(Update, sun_laser_damage_system);

        // Arrange
        app.world_mut().spawn((OrbitalMirror { aligned: false }));
        let building = app.world_mut().spawn((Building { health: 100 })).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<SunLaserDamageEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "Misaligned mirror should emit SunLaserDamageEvent");
    }
}
```

**4. GREEN Phase: Minimal Implementation**

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalMirror {
    pub aligned: bool,
}

#[derive(Component)]
pub struct Farm {
    pub base_output: u32,
    pub current_output: u32,
}

#[derive(Component)]
pub struct Building {
    pub health: u32,
}

#[derive(Event)]
pub struct MeteorImpactEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct SunLaserDamageEvent {
    pub target: Entity,
    pub damage: u32,
}

pub fn orbital_mirror_buff_system(
    mirrors: Query<&OrbitalMirror>,
    mut farms: Query<&mut Farm>,
) {
    let active_mirrors = mirrors.iter().filter(|m| m.aligned).count() as u32;
    for mut farm in farms.iter_mut() {
        farm.current_output = farm.base_output + (active_mirrors * 5);
    }
}

pub fn mirror_impact_system(
    mut events: EventReader<MeteorImpactEvent>,
    mut mirrors: Query<&mut OrbitalMirror>,
) {
    for event in events.read() {
        if let Ok(mut mirror) = mirrors.get_mut(event.target) {
            mirror.aligned = false;
        }
    }
}

pub fn sun_laser_damage_system(
    mirrors: Query<&OrbitalMirror>,
    buildings: Query<Entity, With<Building>>,
    mut damage_events: EventWriter<SunLaserDamageEvent>,
) {
    let misaligned_count = mirrors.iter().filter(|m| !m.aligned).count();
    if misaligned_count > 0 {
        // Just pick the first building to damage for minimal implementation
        if let Some(target) = buildings.iter().next() {
            damage_events.send(SunLaserDamageEvent {
                target,
                damage: 50 * misaligned_count as u32,
            });
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Calculate a trajectory or path for the "Sun-Laser" so it sweeps across the map rather than randomly targeting a building.
- Introduce repair mechanics (`RealignmentTask`) for engineers to fix misaligned mirrors.
- Create visual/UI indicators when a mirror misaligns, giving the player time to react.
- Integrate the agricultural buff with the `SolarGrid` power system in a similar systemic fashion.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Aligned mirrors increase farm output
- [ ] Meteor impacts misalign mirrors
- [ ] Misaligned mirrors generate destructive events targeting buildings

**7. Technical Guidance**
- When integrating with `layer2`, ensure cross-layer communication for `MeteorImpactEvent` works correctly via Bevy events.
- To simulate the sweeping laser, consider storing a spatial target coordinate on the `OrbitalMirror` that updates every frame.

**8. Questions**
*Builder: add questions here if spec is unclear.*

# 1287: The Cargo Cult Diplomat

## 1. Overview
**Layer:** Cross-layer (1 -> 3)

**Fantasy:** A primitive colony misunderstanding high-tech diplomacy and worshipping an automated probe.

**Mechanic:** If a Layer 3 civilization sends an automated scout probe to a low-tech Layer 1 colony and the probe crashes or malfunctions, the colony might mistake it for a divine ambassador. The colony will dedicate massive resources to appeasing the dead machine.

## 2. Dependencies
- Cross-Layer Systems (Layer 3 Probe to Layer 1)
- Colony Tech Level tracking
- Resource Tribute System
- Diplomacy / Casus Belli System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            detect_crashed_probe_system,
            generate_cult_tribute_system,
            process_disrespect_casus_belli_system,
        ));
        app
    }

    #[test]
    fn test_low_tech_colony_worships_crashed_probe() {
        let mut app = setup_app();

        // Spawn a low tech colony
        let colony = app.world_mut().spawn((
            Colony { tech_level: 1 }, // Low tech
            ResourcePool { tribute_accumulated: 0 },
        )).id();

        // Spawn a crashed probe on that colony
        app.world_mut().spawn((
            Probe { is_crashed: true, owner_empire: Entity::from_raw(999) },
            Location { colony_entity: colony },
        ));

        app.update();

        // The colony should now have a DivineAmbassador component pointing to the probe
        let mut cult_query = app.world_mut().query::<&DivineAmbassador>();
        let ambassador = cult_query.get(app.world(), colony);
        assert!(ambassador.is_ok(), "Low tech colony should worship crashed probe");
    }

    #[test]
    fn test_cult_generates_tribute() {
        let mut app = setup_app();

        let probe = app.world_mut().spawn(Probe { is_crashed: true, owner_empire: Entity::from_raw(999) }).id();

        let colony = app.world_mut().spawn((
            Colony { tech_level: 1 },
            ResourcePool { tribute_accumulated: 0 },
            DivineAmbassador { probe_entity: probe },
        )).id();

        app.update();

        let resources = app.world().get::<ResourcePool>(colony).unwrap();
        assert!(resources.tribute_accumulated > 0, "Cult should generate tribute resources for the 'god'");
    }

    #[test]
    fn test_disrespecting_god_causes_war() {
        let mut app = setup_app();

        let empire_entity = app.world_mut().spawn(Empire).id();
        let probe = app.world_mut().spawn(Probe { is_crashed: true, owner_empire: empire_entity }).id();

        let colony = app.world_mut().spawn((
            Colony { tech_level: 1 },
            DivineAmbassador { probe_entity: probe },
            DisrespectedGodFlag, // Simulated trigger where empire tries to correct them
        )).id();

        app.update();

        // Check for war declaration event
        let mut casus_belli_query = app.world_mut().query::<&CasusBelli>();
        assert!(casus_belli_query.iter(app.world()).any(|cb| cb.target == empire_entity), "Disrespecting the cult should generate a Casus Belli against the probe's owner");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony {
    pub tech_level: u32,
}

#[derive(Component)]
pub struct ResourcePool {
    pub tribute_accumulated: u32,
}

#[derive(Component)]
pub struct Probe {
    pub is_crashed: bool,
    pub owner_empire: Entity,
}

#[derive(Component)]
pub struct Location {
    pub colony_entity: Entity,
}

#[derive(Component)]
pub struct DivineAmbassador {
    pub probe_entity: Entity,
}

#[derive(Component)]
pub struct DisrespectedGodFlag;

#[derive(Component)]
pub struct Empire;

#[derive(Component)]
pub struct CasusBelli {
    pub source: Entity,
    pub target: Entity,
}

pub fn detect_crashed_probe_system(
    mut commands: Commands,
    colonies: Query<(Entity, &Colony), Without<DivineAmbassador>>,
    probes: Query<(Entity, &Probe, &Location)>,
) {
    for (probe_entity, probe, location) in probes.iter() {
        if probe.is_crashed {
            if let Ok((colony_entity, colony)) = colonies.get(location.colony_entity) {
                // Low tech threshold
                if colony.tech_level <= 2 {
                    commands.entity(colony_entity).insert(DivineAmbassador { probe_entity });
                }
            }
        }
    }
}

pub fn generate_cult_tribute_system(
    mut colonies: Query<(&DivineAmbassador, &mut ResourcePool)>,
) {
    for (_, mut resources) in colonies.iter_mut() {
        resources.tribute_accumulated += 10; // Generate tribute
    }
}

pub fn process_disrespect_casus_belli_system(
    mut commands: Commands,
    colonies: Query<(Entity, &DivineAmbassador), With<DisrespectedGodFlag>>,
    probes: Query<&Probe>,
) {
    for (colony_entity, ambassador) in colonies.iter() {
        if let Ok(probe) = probes.get(ambassador.probe_entity) {
            commands.spawn(CasusBelli {
                source: colony_entity,
                target: probe.owner_empire,
            });
            // Remove flag after processing
            commands.entity(colony_entity).remove::<DisrespectedGodFlag>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Driven Actions**: `DisrespectedGodFlag` is a component, but handling diplomatic actions like correcting beliefs would be better suited as a standard Bevy `Event`.
- **Tribute Extraction**: The generated `tribute_accumulated` sits on the colony. There needs to be a mechanism for the `owner_empire` to actually collect this tribute via Layer 3 mechanics (e.g., a smuggling ship or periodic transfer).
- **Tech Level Constants**: Replace magic numbers like `tech_level <= 2` with named constants or enums representing civilization eras (e.g., `TechEra::BronzeAge`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Cross-Layer Entities**: Ensure `owner_empire` Entity ID references are valid across Layer 1 and Layer 3 contexts. If they exist in different Bevy Worlds, you'll need cross-world ID mapping.
- **Probe Crashing**: Integrate with whatever system handles Layer 3 objects entering Layer 1 airspace.

## 8. Questions
*Builder: add questions here if spec is unclear.*

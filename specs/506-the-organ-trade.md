# 506 - The Organ Trade

## 1. Overview
Turning the dead (or the living) into the most valuable export in the sector. You can enact a "Mandatory Organ Harvesting" edict. Dead Pops, or even live Prisoners, can be processed into "Vital Organs." These trade for astronomical prices on the Layer 3 market, instantly solving any economic crisis. However, enacting the edict creates a permanent, massive "Horror" debuff to all non-psychopathic Pops, and Layer 3 pacifist empires will embargo you.

## 2. Dependencies
- `054` Colony Edicts
- `039` Trade System
- `072` Justice System
- `034` Pop Health and Damage

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;
    // Mock imports
    // use crate::layer1::edicts::{Edict, EdictType};
    // use crate::layer1::pop::{Pop, Status};
    // use crate::layer1::inventory::Inventory;
    // use crate::layer1::morale::MoraleModifier;

    #[derive(Component)]
    struct Pop {
        pub status: String, // "Dead", "Alive"
        pub has_psychopath_trait: bool,
    }

    #[derive(Component)]
    struct Inventory {
        pub vital_organs: u32,
    }

    #[derive(Resource)]
    struct ActiveEdicts {
        pub organ_harvesting: bool,
    }

    #[derive(Component)]
    struct MoraleModifier {
        pub value: i32,
        pub reason: String,
    }

    fn harvest_organs_system(
        mut commands: Commands,
        edicts: Res<ActiveEdicts>,
        mut inventory: Query<&mut Inventory>,
        pops: Query<(Entity, &Pop)>,
    ) {
        // Implementation omitted for RED phase. Should fail tests.
    }

    #[test]
    fn test_organ_harvesting_edict_generates_organs_and_horror() {
        let mut app = App::new();

        app.insert_resource(ActiveEdicts { organ_harvesting: true });

        // Spawn Inventory
        app.world_mut().spawn(Inventory { vital_organs: 0 });

        // Spawn a dead pop
        let dead_pop = app.world_mut().spawn(Pop { status: "Dead".to_string(), has_psychopath_trait: false }).id();

        // Spawn a living, non-psychopathic pop
        let living_pop = app.world_mut().spawn(Pop { status: "Alive".to_string(), has_psychopath_trait: false }).id();

        app.add_systems(Update, harvest_organs_system);
        app.update();

        // Check Inventory
        let mut inv_count = 0;
        for inv in app.world().query::<&Inventory>().iter(app.world()) {
            inv_count = inv.vital_organs;
        }
        assert_eq!(inv_count, 1, "Should harvest 1 organ from the dead pop");

        // Check dead pop is despawned
        assert!(app.world().get_entity(dead_pop).is_none(), "Dead pop should be consumed");

        // Check living pop got horror debuff
        let modifier = app.world().get::<MoraleModifier>(living_pop);
        assert!(modifier.is_some(), "Living pop should get a morale modifier");
        assert_eq!(modifier.unwrap().value, -20, "Living pop should get a -20 Horror debuff");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {
    pub status: String,
    pub has_psychopath_trait: bool,
}

#[derive(Component)]
pub struct Inventory {
    pub vital_organs: u32,
}

#[derive(Resource)]
pub struct ActiveEdicts {
    pub organ_harvesting: bool,
}

#[derive(Component, Clone)]
pub struct MoraleModifier {
    pub value: i32,
    pub reason: String,
}

pub fn harvest_organs_system(
    mut commands: Commands,
    edicts: Res<ActiveEdicts>,
    mut inventory: Query<&mut Inventory>,
    pops: Query<(Entity, &Pop)>,
) {
    if edicts.organ_harvesting {
        let mut organs_harvested = 0;
        let mut entities_to_despawn = Vec::new();

        for (entity, pop) in pops.iter() {
            if pop.status == "Dead" {
                organs_harvested += 1;
                entities_to_despawn.push(entity);
            } else if !pop.has_psychopath_trait {
                commands.entity(entity).insert(MoraleModifier { value: -20, reason: "Horror".to_string() });
            }
        }

        if organs_harvested > 0 {
            for mut inv in inventory.iter_mut() {
                inv.vital_organs += organs_harvested;
            }
        }

        for e in entities_to_despawn {
            commands.entity(e).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event-Driven**: Listen to `PopDeathEvent` instead of scanning all entities every tick.
- **Layer 3 Diplomacy**: `organ_harvesting` needs to broadcast a signal to the Layer 3 diplomacy simulation to apply the "Embargo" modifier from Pacifist Empires.
- **Morale Debuff Cleanup**: Ensure that if the Edict is repealed, the `MoraleModifier` is gracefully removed rather than accumulating.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Dead Pops produce `VitalOrgans`.
- [ ] Living, non-psychopathic Pops receive a Horror debuff while the edict is active.

## 7. Technical Guidance
- Integrate with `034 Pop Health and Damage` to properly track death and corpse entities (which might be separate entities from the Pop itself).
- Ensure the trade value of `VitalOrgans` is set extremely high in the trade tables.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

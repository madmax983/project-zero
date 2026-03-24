# 570 The Bio-Acoustic Miasma

## 1. Overview
This feature introduces a creeping environmental hazard: "The Bio-Acoustic Miasma". A dense, living fog envelops the colony. While not physically toxic, it actively records the social interactions, complaints, and subversive plots of Pops it touches, and then loudly broadcasts them globally as ambient noise. This turns private stress and factional grievances into immediate public knowledge, causing localized paranoia, spontaneous arrests, and massive morale hits.

## 2. Dependencies
- Layer 1 environmental/weather grid (Miasma coverage)
- Layer 1 Pop psychology, stress, and `SocialInteraction` systems
- `Chronicle` system for recording the social fallout

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scale::layer1::weather::{MiasmaCloud, MiasmaRecordedSecret};
    use scale::layer1::psychology::{StressTracker, ParanoiaTracker};

    #[test]
    fn test_miasma_records_interaction() {
        let mut world = setup_test_world();
        let pos = GridPosition { x: 5, y: 5 };
        world.spawn(MiasmaCloud { position: pos });
        let gossiper = world.spawn((Pop, pos, StressTracker { level: 80 })).id();

        record_miasma_secret(&mut world, gossiper);

        let secrets = world.get_resource::<MiasmaRecordedSecret>().unwrap();
        assert!(secrets.contains("high_stress_complaint"));
    }

    #[test]
    fn test_miasma_broadcasts_secret() {
        let mut world = setup_test_world();
        world.insert_resource(MiasmaRecordedSecret { secrets: vec!["plot_strike".to_string()] });
        let listener = world.spawn((Pop, GridPosition { x: 10, y: 10 }, ParanoiaTracker { level: 0 })).id();

        broadcast_miasma_secrets(&mut world);

        let paranoia = world.get::<ParanoiaTracker>(listener).unwrap().level;
        assert!(paranoia > 0);
        assert!(get_chronicle_events(&world).contains("secret_broadcast"));
    }

    #[test]
    fn test_miasma_dissipates() {
        let mut world = setup_test_world();
        let pos = GridPosition { x: 5, y: 5 };
        let cloud = world.spawn(MiasmaCloud { position: pos, lifetime: 1 }).id();

        update_miasma_clouds(&mut world);

        assert!(world.get::<MiasmaCloud>(cloud).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct MiasmaCloud {
    pub position: GridPosition,
    pub lifetime: u32,
}

#[derive(Resource, Default)]
pub struct MiasmaRecordedSecret {
    pub secrets: Vec<String>,
}

#[derive(Component)]
pub struct ParanoiaTracker {
    pub level: u32,
}

pub fn record_miasma_secret(world: &mut World, entity: Entity) {
    if let Some(pos) = world.get::<GridPosition>(entity) {
        if world.query::<&MiasmaCloud>().iter().any(|cloud| cloud.position == *pos) {
            let stress = world.get::<StressTracker>(entity).unwrap().level;
            if stress > 50 {
                let mut secrets = world.get_resource_mut::<MiasmaRecordedSecret>().unwrap();
                secrets.secrets.push("high_stress_complaint".to_string());
            }
        }
    }
}

pub fn broadcast_miasma_secrets(world: &mut World) {
    let mut secrets = world.get_resource_mut::<MiasmaRecordedSecret>().unwrap();
    if !secrets.secrets.is_empty() {
        for mut paranoia in world.query::<&mut ParanoiaTracker>().iter_mut() {
            paranoia.level += 10;
        }
        secrets.secrets.clear();
        let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
        chronicle.add_event("secret_broadcast".to_string());
    }
}

pub fn update_miasma_clouds(world: &mut World) {
    let mut to_despawn = Vec::new();
    for (entity, mut cloud) in world.query::<(Entity, &mut MiasmaCloud)>().iter_mut() {
        if cloud.lifetime == 1 {
            to_despawn.push(entity);
        } else {
            cloud.lifetime -= 1;
        }
    }
    for e in to_despawn {
        world.despawn(e);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Replace the string-based `secrets` vector with an `Enum` or struct representing specific grievance types (e.g., Faction Plot, Workplace Complaint) for nuanced paranoia tracking.
- **Code Smells:** `record_miasma_secret` does an O(N) cloud lookup per pop; utilize a spatial grid or `OccupancyMap` to check if a Pop's tile contains Miasma.
- **Performance:** Ensure broadcasting does not negatively impact frame rate. Use `EventWriter` to batch broadcasts and `EventReader` to apply paranoia over time.
- **API Improvements:** Create an `AmbientAudioEvent` to notify the UI/Audio layer to play garbled whispering sound effects when secrets are broadcast.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Miasma clouds record high-stress Pop states and convert them into global paranoia events.

## 7. Technical Guidance
- **Code Structure:** Add the cloud logic to `src/layer1/weather.rs` and the psychological effects to `src/layer1/psychology.rs`.
- **Integration Points:** Link `ParanoiaTracker` increases directly to the existing mental break threshold logic.
- **Gotchas:** Make sure the Miasma dissipates correctly over time so it does not permanently lock the colony into a cycle of maximum paranoia.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*

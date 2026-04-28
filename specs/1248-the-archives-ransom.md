# 1248: The Archive's Ransom

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** A precursor artifact holds the cure to your plagues, but it demands your culture in return.
**Mechanic:** You discover a massive Precursor Archive. It offers instant tech unlocks or cures for ongoing disasters, but the "currency" it demands is the permanent deletion of specific Colony Memories or Art.
**Emergence:** To cure a deadly epidemic, you trade the Archive the memory of your founding heroes. The disease is cured, but the entire colony instantly forgets why they are there, leading to a massive ideological schism and the collapse of your political system.
**Tension:** Sacrifice your history and identity for immediate survival, or hold onto your culture even if it means dying?

## 2. Dependencies
- Layer 1 Pop Memories / Cultural Artifacts
- Layer 3 Tech Tree / Ongoing Disasters
- Precursor Archive Entity/Event System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_trading_memory_cures_disaster() {
        let mut app = App::new();
        app.add_systems(Update, process_archive_trade_system);

        // Add a colony memory
        let memory_id = app.world.spawn(ColonyMemory {
            id: "founding_heroes".to_string(),
            cultural_value: 100.0
        }).id();

        // Add an ongoing disaster
        let disaster_id = app.world.spawn(OngoingDisaster {
            type_: DisasterType::Epidemic,
            severity: 50.0
        }).id();

        // Spawn Precursor Archive
        let archive_id = app.world.spawn(PrecursorArchive).id();

        // Send a trade event
        app.world.insert_resource(Events::<ArchiveTradeEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<ArchiveTradeEvent>>().unwrap();
        events.send(ArchiveTradeEvent {
            archive: archive_id,
            sacrificed_memory: memory_id,
            target_disaster: Some(disaster_id),
            target_tech: None,
        });

        app.update();

        // The disaster should be cured (despawned or severity zeroed)
        assert!(app.world.get::<OngoingDisaster>(disaster_id).is_none(), "Disaster must be removed after successful trade");

        // The memory should be deleted
        assert!(app.world.get::<ColonyMemory>(memory_id).is_none(), "Sacrificed memory must be permanently deleted");
    }

    #[test]
    fn test_memory_deletion_causes_ideological_schism() {
        let mut app = App::new();
        app.add_systems(Update, (process_archive_trade_system, apply_schism_system));

        let memory_id = app.world.spawn(ColonyMemory {
            id: "founding_heroes".to_string(),
            cultural_value: 100.0
        }).id();

        // Add a Pop that holds this memory dear
        let pop_id = app.world.spawn((Pop, Unrest(0.0))).id();

        // Add the archive and a trade event
        let archive_id = app.world.spawn(PrecursorArchive).id();
        app.world.insert_resource(Events::<ArchiveTradeEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<ArchiveTradeEvent>>().unwrap();
        events.send(ArchiveTradeEvent {
            archive: archive_id,
            sacrificed_memory: memory_id,
            target_disaster: None,
            target_tech: Some("advanced_lasers".to_string()),
        });

        app.update();

        // The loss of a high-value memory should cause massive unrest/schism in the pops
        let unrest = app.world.get::<Unrest>(pop_id).unwrap();
        assert!(unrest.0 > 0.0, "Deleting a memory must increase unrest and trigger an ideological schism");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ColonyMemory {
    pub id: String,
    pub cultural_value: f32,
}

#[derive(Component)]
pub struct OngoingDisaster {
    pub type_: DisasterType,
    pub severity: f32,
}

#[derive(PartialEq)]
pub enum DisasterType {
    Epidemic,
    Famine,
}

#[derive(Component)]
pub struct PrecursorArchive;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Unrest(pub f32);

#[derive(Event)]
pub struct ArchiveTradeEvent {
    pub archive: Entity,
    pub sacrificed_memory: Entity,
    pub target_disaster: Option<Entity>,
    pub target_tech: Option<String>,
}

pub fn process_archive_trade_system(
    mut trade_events: EventReader<ArchiveTradeEvent>,
    mut commands: Commands,
    archives: Query<&PrecursorArchive>,
    memories: Query<&ColonyMemory>,
) {
    for event in trade_events.read() {
        if archives.get(event.archive).is_ok() {
            if let Ok(memory) = memories.get(event.sacrificed_memory) {
                let val = memory.cultural_value;
                // Delete the memory
                commands.entity(event.sacrificed_memory).despawn();

                // Cure the disaster
                if let Some(disaster) = event.target_disaster {
                    commands.entity(disaster).despawn();
                }

                // (Tech unlock logic would go here)
            }
        }
    }
}

pub fn apply_schism_system(
    // In a real implementation, this would listen to a MemoryDeletedEvent to know *which* memory was lost
    // For MVP, we can just bump unrest if we detect a trade event happened
    mut trade_events: EventReader<ArchiveTradeEvent>,
    mut pops: Query<&mut Unrest, With<Pop>>,
) {
    let mut schism = false;
    for _ in trade_events.read() {
        schism = true;
    }

    if schism {
        for mut unrest in pops.iter_mut() {
            unrest.0 += 50.0; // Massive unrest spike
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Driven:** Instead of `apply_schism_system` directly reading `ArchiveTradeEvent`, the trade system should emit a `MemoryErasedEvent(String)`. The schism system should listen to that, applying unrest only to pops that cared about that specific memory.
- **Tech Unlocks:** Integrate the `target_tech` branch with the existing Layer 3 technology manager.
- **UI:** Expose a dedicated "Precursor Archive Interface" where the player selects a memory and sees the "cost" (expected unrest) before confirming the trade.

## 6. Acceptance Criteria
- [ ] `ArchiveTradeEvent` deletes a `ColonyMemory`.
- [ ] Processing the trade cures an `OngoingDisaster` or unlocks a tech.
- [ ] Memory deletion triggers an unrest/schism mechanic across Pops.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- Be careful with `Commands::despawn()`. Ensure no dangling Entity references remain in other components when a Memory is erased.
- The Precursor Archive should probably be spawned via a Layer 2 exploration event (e.g., surveying an anomaly).

## 8. Questions
*Builder: Add any questions here.*

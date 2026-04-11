# The Derelict Lottery (Spec 723)

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Every wreck is a treasure chest that might bite.
**Mechanic:** You find "Derelict Hulks" in orbit. Scanning gives vague info ("High Energy", "Biologicals"). You must tow them to Layer 1 to open them. They contain massive loot (Ancient Tech, Raw Resources) or massive threats (Xenomorphs, Plagues) that spill out immediately upon landing.
**Emergence:** You tow a "Medical Frigate" hoping for medicine. It was a quarantine ship. You just landed a zombie plague in your city center.
**Tension:** Safe scrap (orbit) vs. The Mystery Box (ground).

## 2. Dependencies
- `layer2::station::Station` or generic orbital Point of Interest system.
- `layer2::fleet::Fleet` towing mechanics.
- `layer1::events` for spawning ground threats.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::Fleet;
    use crate::layer1::economy::{ColonyResources, ResourceType};
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<TowDerelictEvent>();
        app.add_event::<OpenDerelictEvent>();
        app.add_event::<BiologicalThreatEvent>();
        app.add_systems(Update, (process_derelict_towing, process_derelict_opening));
        app.init_resource::<ColonyResources>();
        app
    }

    #[test]
    fn test_derelict_can_be_towed_to_colony() {
        let mut app = setup_app();

        let derelict = app.world_mut().spawn(DerelictHulk {
            scan_hint: "High Energy".to_string(),
            contents: DerelictContents::AncientTech(50.0),
            is_towed: false,
        }).id();

        let fleet = app.world_mut().spawn(Fleet::default()).id();

        // Towing event
        app.world_mut().send_event(TowDerelictEvent {
            fleet,
            derelict,
            destination: OrbitSlot::Colony,
        });

        app.update();

        let hulk = app.world().get::<DerelictHulk>(derelict).unwrap();
        assert!(hulk.is_towed);
        // Verify destination somehow, e.g., location component update
    }

    #[test]
    fn test_opening_tech_derelict_grants_resources() {
        let mut app = setup_app();

        let derelict = app.world_mut().spawn(DerelictHulk {
            scan_hint: "High Energy".to_string(),
            contents: DerelictContents::AncientTech(50.0),
            is_towed: true,
        }).id();

        app.world_mut().send_event(OpenDerelictEvent { derelict });

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::Research), 50.0);

        // Ensure derelict is despawned
        assert!(app.world().get::<DerelictHulk>(derelict).is_none());
    }

    #[test]
    fn test_opening_biological_derelict_spawns_threats() {
        let mut app = setup_app();

        let derelict = app.world_mut().spawn(DerelictHulk {
            scan_hint: "Biologicals".to_string(),
            contents: DerelictContents::Plague,
            is_towed: true,
        }).id();

        // Spawn a target pop to catch the plague
        let pop = app.world_mut().spawn(Pop::default()).id();

        app.world_mut().send_event(OpenDerelictEvent { derelict });

        app.update();

        let events = app.world().resource::<Events<BiologicalThreatEvent>>();
        let mut reader = events.get_reader();
        let evs: Vec<_> = reader.read(events).collect();

        assert_eq!(evs.len(), 1);
        assert!(app.world().get::<DerelictHulk>(derelict).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::economy::{ColonyResources, ResourceType};
use crate::layer2::fleet::Fleet;

#[derive(Component)]
pub struct DerelictHulk {
    pub scan_hint: String,
    pub contents: DerelictContents,
    pub is_towed: bool,
}

pub enum DerelictContents {
    AncientTech(f32),
    RawResources(ResourceType, f32),
    Plague,
    Xenomorphs,
}

#[derive(Event)]
pub struct TowDerelictEvent {
    pub fleet: Entity,
    pub derelict: Entity,
    pub destination: OrbitSlot,
}

pub enum OrbitSlot {
    Colony,
    DeepSpace,
}

#[derive(Event)]
pub struct OpenDerelictEvent {
    pub derelict: Entity,
}

#[derive(Event)]
pub struct BiologicalThreatEvent;

pub fn process_derelict_towing(
    mut events: EventReader<TowDerelictEvent>,
    mut derelicts: Query<&mut DerelictHulk>,
) {
    for event in events.read() {
        if let Ok(mut hulk) = derelicts.get_mut(event.derelict) {
            hulk.is_towed = true;
            // Additional logic to attach to fleet transform or orbit slot
        }
    }
}

pub fn process_derelict_opening(
    mut commands: Commands,
    mut events: EventReader<OpenDerelictEvent>,
    derelicts: Query<&DerelictHulk>,
    mut resources: ResMut<ColonyResources>,
    mut bio_threats: EventWriter<BiologicalThreatEvent>,
) {
    for event in events.read() {
        if let Ok(hulk) = derelicts.get(event.derelict) {
            if !hulk.is_towed {
                continue; // Must be towed to open
            }

            match &hulk.contents {
                DerelictContents::AncientTech(amount) => resources.add(ResourceType::Research, *amount),
                DerelictContents::RawResources(res_type, amount) => resources.add(*res_type, *amount),
                DerelictContents::Plague => bio_threats.send(BiologicalThreatEvent),
                DerelictContents::Xenomorphs => { /* Spawn combat event on colony surface */ },
            }

            commands.entity(event.derelict).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Towing Mechanic**: Need to ensure ships lose speed or capacity while towing a heavy derelict. `layer2::fleet::FleetMovementEvent` should check for attached hulks.
- **Threat Events**: `BiologicalThreatEvent` should integrate directly with `layer1::sickness` or morale decay. Xenomorphs should trigger `layer1::combat` or vermin systems.
- **Scanning UI**: The "scan hint" string must be surfaced to the UI so the player can gamble based on "High Energy" vs "Faint Biosignatures".

## 6. Acceptance Criteria
- [ ] `DerelictHulk` component spawns in Layer 2 space.
- [ ] Fleets can tow hulks to the colony.
- [ ] Opening hulks yields the correct `DerelictContents` (loot vs threat).
- [ ] Unknown threats generate corresponding events on Layer 1.
- [ ] Test coverage >85%.

## 7. Technical Guidance
- Integrate into a new file `src/layer2/derelict.rs`.
- Buffer and register all events in `src/setup.rs` and `src/layer1/systems/cleanup.rs`.
- Use Bevy events (`AddChronicleEvent`) to record the outcome of opening the "mystery box".

## 8. Questions
- *Builder: Can players scrap the derelict in orbit for a minor, safe yield instead of towing it down?*
  *Architect:* Yes, but the yield is significantly lower (10% of total) and removes the chance for rare tech.
- *Builder: What determines the scan accuracy? Can high-tech sensors reveal exactly what is inside?*
  *Architect:* Sensor level determines the detail of the tooltip, from vague "Energy Spike" to specific "Active Bio-Weapon".

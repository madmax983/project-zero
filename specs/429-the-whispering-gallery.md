# Spec 429: The Whispering Gallery

## 1. Overview
A beautifully constructed "Grand Dome" provides massive Morale but functions as a perfect acoustic reflector. All Rumors (from the Rumor Web) spoken within it are broadcast to the entire colony, instantly accelerating the spread of both positive news and terrifying false panics.

## 2. Dependencies
- `055-rumor-web` (Rumor Web System)
- `064-room-quality` (Room Quality/Morale System)
- `343-the-rumor-web` (Rumor Web Expansion)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_grand_dome_amplifies_rumors() {
        let mut app = App::new();
        app.add_event::<RumorSpokenEvent>();
        app.add_event::<BroadcastRumorEvent>();
        app.add_systems(Update, whispering_gallery_system);

        let dome_entity = app.world_mut().spawn((
            Room,
            AcousticReflector { amplification: 10.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        app.world_mut().send_event(RumorSpokenEvent {
            rumor_id: "food_shortage".to_string(),
            speaker: Entity::PLACEHOLDER,
            location: Vec3::ZERO,
            room_entity: Some(dome_entity),
        });

        app.update();

        let broadcast_events = app.world().resource::<Events<BroadcastRumorEvent>>();
        let mut reader = broadcast_events.get_reader();
        let events: Vec<_> = reader.read(broadcast_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].rumor_id, "food_shortage");
        assert_eq!(events[0].colony_wide, true);
    }

    #[test]
    fn test_rumor_outside_dome_not_amplified() {
        let mut app = App::new();
        app.add_event::<RumorSpokenEvent>();
        app.add_event::<BroadcastRumorEvent>();
        app.add_systems(Update, whispering_gallery_system);

        let standard_room = app.world_mut().spawn((
            Room,
            Transform::from_xyz(10.0, 0.0, 0.0),
        )).id();

        app.world_mut().send_event(RumorSpokenEvent {
            rumor_id: "mutiny".to_string(),
            speaker: Entity::PLACEHOLDER,
            location: Vec3::new(10.0, 0.0, 0.0),
            room_entity: Some(standard_room),
        });

        app.update();

        let broadcast_events = app.world().resource::<Events<BroadcastRumorEvent>>();
        let mut reader = broadcast_events.get_reader();
        let events: Vec<_> = reader.read(broadcast_events).collect();

        assert_eq!(events.len(), 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct AcousticReflector {
    pub amplification: f32,
}

#[derive(Event)]
pub struct RumorSpokenEvent {
    pub rumor_id: String,
    pub speaker: Entity,
    pub location: Vec3,
    pub room_entity: Option<Entity>,
}

#[derive(Event)]
pub struct BroadcastRumorEvent {
    pub rumor_id: String,
    pub colony_wide: bool,
}

pub fn whispering_gallery_system(
    mut rumor_spoken_events: EventReader<RumorSpokenEvent>,
    mut broadcast_rumor_events: EventWriter<BroadcastRumorEvent>,
    reflectors: Query<&AcousticReflector>,
) {
    for event in rumor_spoken_events.read() {
        if let Some(room_entity) = event.room_entity {
            if reflectors.get(room_entity).is_ok() {
                broadcast_rumor_events.send(BroadcastRumorEvent {
                    rumor_id: event.rumor_id.clone(),
                    colony_wide: true,
                });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Consider calculating propagation based on actual acoustic paths rather than instantly blanketing the colony.
- Add an `AcousticDampener` component to counter the effects of a reflector.
- Integrate with morale to ensure `Grand Dome` provides baseline mood buffs even without rumors.

## 6. Acceptance Criteria
- [ ] `AcousticReflector` component exists and can be attached to rooms.
- [ ] Rumors spoken in a room with an `AcousticReflector` trigger a colony-wide broadcast.
- [ ] Rumors spoken elsewhere do not trigger the broadcast.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Ensure `AcousticReflector` only broadcasts once per unique rumor within a tick to prevent infinite echo loops.
- Hook into the existing Rumor System to ensure `BroadcastRumorEvent` actually affects Pop moods.

## 8. Questions
- Should the Grand Dome amplify positive rumors more than negative ones, or is it strictly neutral?
- *Architect:* The Grand Dome is strictly neutral for MVP. It simply increases the propagation radius of any rumor generated within it, regardless of sentiment.

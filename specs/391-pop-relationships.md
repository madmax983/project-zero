# 391 - Pop Relationships

## 1. Overview
This feature introduces social relationships between Pops in the Colony Layer. Pops track their affinity with other Pops they interact with. High affinity provides mood buffs when near, while low affinity causes debuffs and potential conflicts. This adds social fabric, allowing "power couples" to work better together and rivals to sabotage each other.

## 2. Dependencies
- `003-population-basics.md` (Pop entity and Needs)
- `016-utility-ai-system.md` (Utility AI to handle interactions and mood changes)
- `031-pop-morale.md` (Mood system)
- `047-pop-relationships.md` (If existing, this will extend or define it. We assume 047 was partial or related, but this spec fully defines the affinity tracking and aura effects)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{Pop, Mood};

    #[test]
    fn test_affinity_tracking_creation() {
        let mut app = App::new();
        app.add_systems(Update, process_social_interactions);

        let pop1 = app.world_mut().spawn((Pop, Needs::default(), Mood::default())).id();
        let pop2 = app.world_mut().spawn((Pop, Needs::default(), Mood::default())).id();

        // Simulate interaction adding affinity
        app.world_mut().resource_mut::<Events<SocialInteractionEvent>>().send(SocialInteractionEvent {
            pop_a: pop1,
            pop_b: pop2,
            affinity_change: 5.0,
        });

        app.update();

        // pop1 should have affinity with pop2
        let relationships1 = app.world().get::<PopRelationships>(pop1).unwrap();
        assert_eq!(relationships1.get_affinity(pop2), 5.0);

        // Relationship is reciprocal
        let relationships2 = app.world().get::<PopRelationships>(pop2).unwrap();
        assert_eq!(relationships2.get_affinity(pop1), 5.0);
    }

    #[test]
    fn test_affinity_aura_mood_buff() {
        let mut app = App::new();
        app.add_systems(Update, apply_relationship_auras);

        let pop1 = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Mood { current: 50.0, ..default() },
        )).id();

        let pop2 = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(1.0, 0.0, 0.0), // Close enough
            Mood { current: 50.0, ..default() },
        )).id();

        // Set high affinity
        let mut rel1 = PopRelationships::default();
        rel1.set_affinity(pop2, 50.0);
        app.world_mut().entity_mut(pop1).insert(rel1);

        app.update();

        // pop1 gets mood buff from pop2
        let mood1 = app.world().get::<Mood>(pop1).unwrap();
        assert!(mood1.current > 50.0, "High affinity should buff mood when nearby");
    }

    #[test]
    fn test_affinity_aura_mood_debuff() {
        let mut app = App::new();
        app.add_systems(Update, apply_relationship_auras);

        let pop1 = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Mood { current: 50.0, ..default() },
        )).id();

        let pop2 = app.world_mut().spawn((
            Pop,
            Transform::from_xyz(1.0, 0.0, 0.0), // Close enough
            Mood { current: 50.0, ..default() },
        )).id();

        // Set low affinity (rival)
        let mut rel1 = PopRelationships::default();
        rel1.set_affinity(pop2, -50.0);
        app.world_mut().entity_mut(pop1).insert(rel1);

        app.update();

        // pop1 gets mood debuff from pop2
        let mood1 = app.world().get::<Mood>(pop1).unwrap();
        assert!(mood1.current < 50.0, "Low affinity should debuff mood when nearby");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::layer1::pop::Mood;

#[derive(Component, Default)]
pub struct PopRelationships {
    pub affinities: HashMap<Entity, f32>,
}

impl PopRelationships {
    pub fn get_affinity(&self, other: Entity) -> f32 {
        self.affinities.get(&other).copied().unwrap_or(0.0)
    }

    pub fn set_affinity(&mut self, other: Entity, value: f32) {
        self.affinities.insert(other, value);
    }
}

#[derive(Event)]
pub struct SocialInteractionEvent {
    pub pop_a: Entity,
    pub pop_b: Entity,
    pub affinity_change: f32,
}

pub fn process_social_interactions(
    mut events: EventReader<SocialInteractionEvent>,
    mut query: Query<&mut PopRelationships>,
) {
    for event in events.read() {
        // Update pop_a
        if let Ok(mut rel) = query.get_mut(event.pop_a) {
            let current = rel.get_affinity(event.pop_b);
            rel.set_affinity(event.pop_b, current + event.affinity_change);
        }
        // Update pop_b
        if let Ok(mut rel) = query.get_mut(event.pop_b) {
            let current = rel.get_affinity(event.pop_a);
            rel.set_affinity(event.pop_a, current + event.affinity_change);
        }
    }
}

pub fn apply_relationship_auras(
    mut query: Query<(Entity, &Transform, &PopRelationships, &mut Mood)>,
    all_pops: Query<(Entity, &Transform), With<Mood>>,
) {
    let aura_radius = 5.0;

    let mut mood_changes: Vec<(Entity, f32)> = Vec::new();

    for (entity1, transform1, rel, _mood) in query.iter() {
        let mut mood_mod = 0.0;
        for (entity2, transform2) in all_pops.iter() {
            if entity1 == entity2 { continue; }

            let dist = transform1.translation.distance(transform2.translation);
            if dist <= aura_radius {
                let affinity = rel.get_affinity(entity2);
                if affinity > 20.0 {
                    mood_mod += 1.0; // Buff
                } else if affinity < -20.0 {
                    mood_mod -= 1.0; // Debuff
                }
            }
        }
        mood_changes.push((entity1, mood_mod));
    }

    for (entity, change) in mood_changes {
        if let Ok((_, _, _, mut mood)) = query.get_mut(entity) {
            mood.current = (mood.current + change).clamp(0.0, 100.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Hash / Grid Optimization**: `apply_relationship_auras` uses $O(N^2)$ distance checks. This should be optimized using a spatial grid or the existing `TerrainGrid` to only check adjacent/nearby tiles.
- **Affinity Decay**: Relationships should slowly decay towards 0.0 over time if there's no interaction.
- **Event Handling**: Expand `SocialInteractionEvent` to include the *type* of interaction (Chat, Fight, Work Together) which could dictate the affinity change.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥ 85% for the `layer1/social.rs` or `layer1/relationships.rs`
- [ ] Pops correctly gain/lose mood based on proximity to high/low affinity Pops.

## 7. Technical Guidance
- `PopRelationships` should be added to the `PopBundle`.
- Add `apply_relationship_auras` to the `Observation` or `Needs` system sets, running periodically (e.g., once per in-game hour or day).
- Hook up `SocialInteractionEvent` emissions in the `utility_ai_system` when Pops perform social actions like "Chat".

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 705 - The Scapegoat Protocol

## 1. Overview
The darkest aspect of crowd psychology. When things go wrong, the mob demands blood, not solutions. During periods of severe, prolonged Unrest (starvation, repeated raids), the colony's Pops will spontaneously generate a "Scapegoat" narrative, blaming a specific, innocent Pop (often a minority faction or someone with a strange trait) for the crisis. If the player publicly exiles or executes the Scapegoat, the Unrest instantly vanishes for a time, regardless of the actual underlying problem.

## 2. Dependencies
- `050-civil-unrest` (Unrest mechanic)
- `031-pop-morale` (Morale states)
- `010-chronicle-system` (Chronicle/History logging)
- `691-the-exile` (for Exiling pops)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::unrest::Unrest;
    use crate::layer1::chronicle::AddChronicleEvent;

    #[test]
    fn test_scapegoat_narrative_generates_under_high_unrest() {
        let mut world = World::new();
        world.insert_resource(Unrest { level: 90.0 }); // High unrest

        let pop_a = world.spawn((Pop, Trait::Xenophile)).id();
        let pop_b = world.spawn((Pop, Trait::Standard)).id();
        let pop_c = world.spawn((Pop, Trait::Standard)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(evaluate_scapegoat_emergence);

        // Run several ticks
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        // Check if a Scapegoat component was added to one of the pops
        let mut scapegoat_count = 0;
        let mut scapegoat_entity = None;
        for (entity, _) in world.query::<(Entity, &Scapegoat)>().iter(&world) {
            scapegoat_count += 1;
            scapegoat_entity = Some(entity);
        }

        assert_eq!(scapegoat_count, 1, "Exactly one scapegoat should emerge");
        // Pop A with the weird trait should ideally be the one chosen, but testing exactly which one depends on the trait scoring logic.
    }

    #[test]
    fn test_executing_scapegoat_clears_unrest_and_adds_memory() {
        let mut world = World::new();
        world.insert_resource(Unrest { level: 95.0 });
        world.init_resource::<Events<AddChronicleEvent>>();

        let scapegoat_entity = world.spawn((Pop, Scapegoat)).id();

        // The player action to execute/exile
        let mut events = world.resource_mut::<Events<ExecuteScapegoatEvent>>();
        events.send(ExecuteScapegoatEvent { target: scapegoat_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_scapegoat_executions);

        schedule.run(&mut world);

        // The scapegoat should be despawned (executed)
        assert!(world.get_entity(scapegoat_entity).is_none(), "Scapegoat should be despawned");

        // Unrest should be artificially crushed
        let unrest = world.resource::<Unrest>();
        assert_eq!(unrest.level, 0.0, "Unrest should be instantly reset to 0");

        // A chronicle event for "Blood on our Hands" should be generated
        let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_reader();
        let mut found_event = false;
        for ev in reader.read(chronicle_events) {
            if ev.template_id == "BLOOD_ON_OUR_HANDS" {
                found_event = true;
                break;
            }
        }
        assert!(found_event, "Execution should add a chronicle event recording the dark deed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/social/scapegoat.rs

use bevy::prelude::*;
use crate::layer1::social::unrest::Unrest;
use crate::layer1::chronicle::AddChronicleEvent;
use crate::layer1::pop::Pop;

#[derive(Component, Debug, Clone)]
pub struct Scapegoat;

#[derive(Event, Debug)]
pub struct ExecuteScapegoatEvent {
    pub target: Entity,
}

pub fn evaluate_scapegoat_emergence(
    mut commands: Commands,
    unrest: Res<Unrest>,
    pop_query: Query<Entity, (With<Pop>, Without<Scapegoat>)>,
    scapegoat_query: Query<Entity, With<Scapegoat>>,
) {
    if unrest.level > 80.0 {
        // If no scapegoat exists, create one
        if scapegoat_query.is_empty() {
            if let Some(target) = pop_query.iter().next() { // Just grab the first one for minimal impl
                commands.entity(target).insert(Scapegoat);
            }
        }
    }
}

pub fn process_scapegoat_executions(
    mut commands: Commands,
    mut events: EventReader<ExecuteScapegoatEvent>,
    mut unrest: ResMut<Unrest>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.target).despawn_recursive();

        // Crush unrest
        unrest.level = 0.0;

        // Add history
        chronicle_events.send(AddChronicleEvent {
            template_id: "BLOOD_ON_OUR_HANDS".to_string(),
            entities: vec![ev.target.to_bits().to_string()],
            ..Default::default()
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Selection Logic:** The minimal implementation just grabs the first available Pop. This should be refactored to score Pops based on minority traits, low social standing, or specific factions to fulfill the "mob targets the outsider" fantasy.
- **Unrest Suppression Duration:** Simply setting Unrest to 0 is a bit simplistic; the underlying causes (e.g., starvation) will cause it to rise again immediately. Add a temporary "Catharsis" or "Mob Satiated" resource/modifier that prevents Unrest from growing for a set duration (e.g., 10 days).
- **Exile Alternative:** Add an `ExileScapegoatEvent` that hooks into the existing exile mechanics (`691-the-exile`) instead of despawning, giving the same short-term unrest relief but a different chronicle outcome.

## 6. Acceptance Criteria
- [ ] Sustained high unrest (e.g., >80.0) causes a `Scapegoat` component to be added to a Pop.
- [ ] Firing an `ExecuteScapegoatEvent` despawns the target.
- [ ] Executing a scapegoat immediately sets colony Unrest to 0.
- [ ] Executing a scapegoat triggers an `AddChronicleEvent` with the template "BLOOD_ON_OUR_HANDS".
- [ ] `cargo test` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage for `scapegoat.rs` is ≥85%.

## 7. Technical Guidance
- Add to `src/layer1/social/scapegoat.rs`.
- Register the systems in `src/layer1/systems/social.rs` or `observation.rs`.
- Ensure you register the `ExecuteScapegoatEvent` in `src/setup.rs` and `src/layer1/systems/cleanup.rs`.
- The chronicle template "BLOOD_ON_OUR_HANDS" will need to be added to `lore/TEMPLATES.md` by the Lore Master later, but the code can use the string ID now.

## 8. Questions
*Builder: Add questions here if spec is unclear.*

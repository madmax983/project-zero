# 783 The Sub-Lithic Cult

## 1. Overview
**Layer:** 1
**Fantasy:** A society that worships the deep underground and fears the open sky.
**Mechanic:** Pops assigned to deep mining for multiple generations begin to develop "Agoraphobia." They refuse to work on the surface and form a cult that actively sabotages spaceports to prevent people from "leaving the embrace of the stone."
**Emergence:** Your primary spaceport is destroyed just as a colony ship is about to launch. When you send enforcers into the deep mines to arrest the saboteurs, you discover they've built a massive, thriving subterranean city entirely independent of your rule.
**Tension:** The incredible mining efficiency of the deep-dwellers vs. the slow, inevitable loss of control over your own subterranean infrastructure.

## 2. Dependencies
- `004-pop-entity`
- `068-pop-factions`
- `084-pop-traits`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_agoraphobia_development() {
        // Arrange: Setup world with a pop assigned to deep mining
        let mut app = App::new();
        app.add_systems(Update, process_deep_mining_exposure);

        let pop = app.world_mut().spawn((
            Pop,
            JobAssignment::DeepMining,
            DeepMiningExposure(0.0),
        )).id();

        // Act: Run simulation ticks to increase exposure
        for _ in 0..10 {
            app.update();
        }

        // Assert: Verify pop developed Agoraphobia trait
        let exposure = app.world().get::<DeepMiningExposure>(pop).unwrap();
        assert!(exposure.0 > 1.0); // Threshold reached

        let has_agoraphobia = app.world().get::<TraitAgoraphobia>(pop).is_some();
        assert!(has_agoraphobia, "Pop should develop Agoraphobia after prolonged deep mining");
    }

    #[test]
    fn test_sub_lithic_cult_formation() {
        // Arrange: Multiple pops with Agoraphobia
        let mut app = App::new();
        app.add_systems(Update, evaluate_cult_formation);

        for _ in 0..5 {
            app.world_mut().spawn((Pop, TraitAgoraphobia));
        }

        // Act: Run cult formation system
        app.update();

        // Assert: A new Faction with Sub-Lithic Ideology should be created
        let mut query = app.world_mut().query::<&Faction>();
        let cult_exists = query.iter(app.world()).any(|f| f.ideology == Ideology::SubLithic);
        assert!(cult_exists, "Sub-Lithic Cult faction should form when enough pops have Agoraphobia");
    }

    #[test]
    fn test_spaceport_sabotage() {
        // Arrange: A cult member and a spaceport building
        let mut app = App::new();
        app.add_event::<SabotageEvent>();
        app.add_systems(Update, process_cult_sabotage);

        let spaceport = app.world_mut().spawn((Building::Spaceport, Health(100.0))).id();
        app.world_mut().spawn((
            Pop,
            TraitAgoraphobia,
            FactionMember(FactionId::SubLithic),
            Position::near(spaceport)
        ));

        let mut events = app.world_mut().resource_mut::<Events<SabotageEvent>>();
        events.clear();

        // Act: Run sabotage logic
        app.update();

        // Assert: Spaceport is sabotaged
        let sabotage_events = app.world().resource::<Events<SabotageEvent>>();
        assert!(sabotage_events.len() > 0, "Cult member should trigger a sabotage event on the spaceport");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Trait indicating fear of surface and preference for deep layers
#[derive(Component, Debug, Clone)]
pub struct TraitAgoraphobia;

#[derive(Component, Default)]
pub struct DeepMiningExposure(pub f32);

// Process exposure to deep mining environments
pub fn process_deep_mining_exposure(
    mut query: Query<(Entity, &JobAssignment, &mut DeepMiningExposure)>,
    mut commands: Commands,
) {
    for (entity, job, mut exposure) in query.iter_mut() {
        if *job == JobAssignment::DeepMining {
            exposure.0 += 0.2; // Arbitrary increment
            if exposure.0 > 1.0 {
                commands.entity(entity).insert(TraitAgoraphobia);
            }
        }
    }
}

// Logic to group agoraphobic pops into the Sub-Lithic Cult faction
pub fn evaluate_cult_formation(
    query: Query<(), With<TraitAgoraphobia>>,
    mut commands: Commands,
) {
    let count = query.iter().count();
    if count >= 5 {
        // Minimal logic: spawn faction if threshold met
        // In real implementation, check if it already exists
        commands.spawn(Faction {
            ideology: Ideology::SubLithic,
            // ... other fields
        });
    }
}

// Logic for cult members to target surface structures
#[derive(Event)]
pub struct SabotageEvent {
    pub target: Entity,
}

pub fn process_cult_sabotage(
    cult_query: Query<(&FactionMember, &Position), With<TraitAgoraphobia>>,
    target_query: Query<(Entity, &Building), With<Health>>,
    mut sabotage_events: EventWriter<SabotageEvent>,
) {
    for (member, _pos) in cult_query.iter() {
        if member.0 == FactionId::SubLithic {
            for (target_entity, building) in target_query.iter() {
                if *building == Building::Spaceport {
                    sabotage_events.send(SabotageEvent { target: target_entity });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:**
  - Ensure `evaluate_cult_formation` does not continuously spawn factions; it needs a check or should operate as an event responder.
  - Sabotage logic should factor in actual pathfinding and timing, rather than immediate proximity triggers.
  - Deep mining exposure should probably consider generational inheritance (as specified in "multiple generations").
- **Code Smells:** Arbitrary floating-point increments for exposure; should tie into `SimulationTime` or actual work cycles.
- **Performance:** `process_cult_sabotage` has an $O(N \times M)$ loop where N is cult members and M is targets. This should be optimized using spatial queries or caching `Spaceport` locations.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops develop Agoraphobia correctly after sustained deep mining.
- [ ] Sub-Lithic Cult forms when sufficient Agoraphobic pops exist.
- [ ] Cult members correctly issue `SabotageEvent`s targeting spaceports.

## 7. Technical Guidance
- **Generational aspect:** Tie `TraitAgoraphobia` not just to individual exposure but allow it to be passed down if parents have it or if born in deep layers.
- **Integration Points:** Link `SabotageEvent` to the existing justice or disaster systems so the player is notified.
- **Gotchas:** Make sure `TraitAgoraphobia` applies penalties when pops are forced to work on the surface (e.g., massive stress or efficiency drops).

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 1181: Biological Copyright Infringement

## 1. Overview
As you genetically modify your Pops (or if they naturally mutate), there is a risk their genome matches a sequence patented by an ancient, hyper-capitalist Fallen Empire. If triggered, the Empire issues a "Cease and Desist" order on your Pops' very existence. You must either pay exorbitant royalties for every Pop born with that trait, or "recall" (purge) the offending biological units.

## 2. Dependencies
- Layer 1/3 Cross-layer interaction.
- `Pop` traits/genetics system.
- Diplomacy or Event system to handle the "Cease and Desist" from the Fallen Empire.
- Economy/Resource system for paying royalties.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_patented_trait_triggers_audit() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, check_patented_traits_system);
    app.add_event::<CopyrightInfringementEvent>();

    // Register a patented trait
    let mut patent_registry = PatentedGenomes::default();
    patent_registry.register(TraitId::FrostHardy, "MegaCorp Mega-Empire");
    app.world_mut().insert_resource(patent_registry);

    // Spawn a pop with the patented trait
    app.world_mut().spawn((Pop, Genome { traits: vec![TraitId::FrostHardy] }));

    // Act
    app.update();

    // Assert: An event should be fired regarding the infringement
    let events = app.world().resource::<Events<CopyrightInfringementEvent>>();
    let mut reader = events.get_reader();
    assert!(reader.read(events).next().is_some(), "Spawning a pop with a patented trait should trigger an infringement event");
}

#[test]
fn test_royalty_payments_drained_per_tick() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, process_royalty_payments_system);

    app.world_mut().insert_resource(ColonyResources { credits: 1000 });

    // We have an active Cease and Desist order that we chose to pay royalties for
    app.world_mut().insert_resource(ActiveRoyalties { cost_per_tick: 50 });

    // Act
    app.update();

    // Assert: Credits should be drained
    let resources = app.world().resource::<ColonyResources>();
    assert_eq!(resources.credits, 950, "Royalties should drain credits per tick");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component)]
pub struct Pop;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraitId {
    FrostHardy,
    NightVision,
    // ...
}

#[derive(Component)]
pub struct Genome {
    pub traits: Vec<TraitId>,
}

#[derive(Resource, Default)]
pub struct PatentedGenomes {
    patents: HashMap<TraitId, String>,
}

impl PatentedGenomes {
    pub fn register(&mut self, trait_id: TraitId, owner: &str) {
        self.patents.insert(trait_id, owner.to_string());
    }
}

#[derive(Event)]
pub struct CopyrightInfringementEvent {
    pub trait_id: TraitId,
    pub owner: String,
}

#[derive(Resource)]
pub struct ActiveRoyalties {
    pub cost_per_tick: u32,
}

#[derive(Resource)]
pub struct ColonyResources {
    pub credits: u32,
}

pub fn check_patented_traits_system(
    mut events: EventWriter<CopyrightInfringementEvent>,
    patents: Res<PatentedGenomes>,
    pops: Query<&Genome, Added<Pop>>, // Check only newly spawned/mutated pops for simplicity
) {
    for genome in pops.iter() {
        for t in &genome.traits {
            if let Some(owner) = patents.patents.get(t) {
                events.send(CopyrightInfringementEvent {
                    trait_id: *t,
                    owner: owner.clone(),
                });
            }
        }
    }
}

pub fn process_royalty_payments_system(
    mut resources: ResMut<ColonyResources>,
    royalties: Option<Res<ActiveRoyalties>>,
) {
    if let Some(active) = royalties {
        if resources.credits >= active.cost_per_tick {
            resources.credits -= active.cost_per_tick;
        } else {
            // Out of money - trigger invasion or purge
            // Handled in a separate system
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Throttling**: The infringement event should probably only fire once per trait, or aggregate into a colony-wide debriefing, rather than firing for every single pop spawned.
- **Dynamic Pricing**: `ActiveRoyalties` should calculate `cost_per_tick` dynamically based on a query counting exactly how many Pops currently possess the offending trait.
- **Consequences**: Add the system that triggers a hostile Layer 3 fleet invasion if `ColonyResources.credits` cannot cover the royalty payment.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Spawning a pop with a patented trait correctly triggers the infringement system, and paying royalties dynamically drains colony resources.

## 7. Technical Guidance
- Integrate with Layer 3 (Galaxy map). The `PatentedGenomes` resource should be populated based on the generation of Fallen Empires during galaxy creation.
- Ensure the player is given a UI prompt via the diplomacy screen to decide whether to pay royalties or accept war.

## 8. Questions
*Builder: add questions here if spec is unclear.*

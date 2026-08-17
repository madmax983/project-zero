# 1376: The Artifact Market

## 1. Overview
**Layer:** Cross-layer (1 -> 2)

**Fantasy:** One colony's trash is an empire's treasure.

**Mechanic:** Everyday items from early colony days (e.g., the first pickaxe, a handwritten diary) gain "Historical Artifact" status over time. They can be sold on the inter-colony trade network for exorbitant prices to wealthy collectors on core worlds.

**Emergence:** You are starving and need credits to buy food shipments. You realize the only thing of value is the founder's original spacesuit, currently displayed in the town square. Selling it saves the colony but permanently tanks local morale and erases a piece of your history.

**Tension:** Preserving cultural heritage vs. immediate survival.

## 2. Dependencies
- Base ECS system
- Item/Inventory tracking
- Colony Morale system
- Trade/Economy system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<SellArtifactEvent>();
        app.add_systems(Update, process_artifact_sale_system);
        app.insert_resource(Economy { credits: 0.0 });
        app.insert_resource(ColonyMorale { value: 100.0 });
        app
    }

    #[test]
    fn test_selling_artifact_grants_credits_but_tanks_morale() {
        let mut app = setup_app();

        // Spawn a historical artifact
        let artifact = app.world_mut().spawn((
            Item,
            HistoricalArtifact { value: 5000.0, morale_penalty: 20.0 },
        )).id();

        app.world_mut().send_event(SellArtifactEvent { artifact_entity: artifact });
        app.update();

        let economy = app.world().resource::<Economy>();
        let morale = app.world().resource::<ColonyMorale>();

        assert_eq!(economy.credits, 5000.0, "Should gain immense credits");
        assert_eq!(morale.value, 80.0, "Should suffer severe morale penalty");

        // Artifact should be removed from world
        assert!(app.world().get::<Item>(artifact).is_none(), "Artifact should be despawned");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct HistoricalArtifact {
    pub value: f32,
    pub morale_penalty: f32,
}

#[derive(Resource)]
pub struct Economy {
    pub credits: f32,
}

#[derive(Resource)]
pub struct ColonyMorale {
    pub value: f32,
}

#[derive(Event)]
pub struct SellArtifactEvent {
    pub artifact_entity: Entity,
}

pub fn process_artifact_sale_system(
    mut commands: Commands,
    mut events: EventReader<SellArtifactEvent>,
    query: Query<&HistoricalArtifact>,
    mut economy: ResMut<Economy>,
    mut morale: ResMut<ColonyMorale>,
) {
    for ev in events.read() {
        if let Ok(artifact) = query.get(ev.artifact_entity) {
            economy.credits += artifact.value;
            morale.value -= artifact.morale_penalty;

            // Remove the artifact from the game (sold)
            commands.entity(ev.artifact_entity).despawn();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Aging Items**: Create a system that periodically scans old items and upgrades them to `HistoricalArtifact` if they exceed a certain age threshold.
- **Museums**: Allow placing artifacts in a `Museum` building to provide a passive passive morale aura, making the choice to sell them harder.
- **Chronicle Hook**: Selling an artifact should trigger an `AddChronicleEvent` documenting the sacrifice.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Selling an artifact increases `Economy.credits`.
- [ ] Selling an artifact decreases `ColonyMorale.value`.
- [ ] The sold artifact entity is despawned.

## 7. Technical Guidance
- The sale logic should integrate into the main Trade UI, likely via a special "Black Market" or "Core World Collector" tab.

## 8. Questions
*Builder: add questions here if spec is unclear.*

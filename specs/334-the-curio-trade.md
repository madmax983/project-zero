# Specification: The Curio Trade

## 1. Overview
**Layer:** Cross-layer (Colony -> Trade/Galaxy)
**Fantasy:** Your colony is a curiosity to the bored elites of the Core Worlds.
**Mechanic:** Unique local items (alien wood carvings, glowing rocks) can be sold for massive credits as "Art", but removing them upsets the local ecosystem or pop culture.
**Emergence:** You strip-mine the sacred singing crystals to pay for a shield generator, causing a spiritual depression in the colony.
**Tension:** Cultural heritage vs. Economic survival.

This feature allows specific gathered or crafted resources to be tagged as `Curio`. Exporting these grants high `Credit` values but triggers a colony-wide `CulturalLossEvent` lowering morale.

## 2. Dependencies
- `TradeDeal` system (Layer 3)
- `ResourceItem` or `Cargo` system
- `MoraleModifier` (Layer 1)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_curio_export_grants_high_credits() {
        let mut app = App::new();
        app.add_systems(Update, process_curio_export_system);

        app.world.insert_resource(ColonyResources { credits: 0 });
        let mut events = Events::<TradeCompletedEvent>::default();
        events.send(TradeCompletedEvent {
            item_type: ResourceType::SingingCrystal,
            is_curio: true,
            quantity: 1,
        });
        app.world.insert_resource(events);

        app.update();

        let resources = app.world.get_resource::<ColonyResources>().unwrap();
        assert!(resources.credits >= 1000, "Curios should sell for a massive premium");
    }

    #[test]
    fn test_curio_export_causes_cultural_loss() {
        let mut app = App::new();
        app.add_systems(Update, apply_cultural_loss_system);

        let entity = app.world.spawn((
            Pop { id: 1 },
            Morale { current: 100.0 }
        )).id();

        let mut events = Events::<CulturalLossEvent>::default();
        events.send(CulturalLossEvent { magnitude: 20.0 });
        app.world.insert_resource(events);

        app.update();

        let morale = app.world.get::<Morale>(entity).unwrap();
        assert!(morale.current < 100.0, "Pops should lose morale when cultural items are exported");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct ColonyResources {
    pub credits: i32,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    Iron,
    SingingCrystal,
}

#[derive(Event)]
pub struct TradeCompletedEvent {
    pub item_type: ResourceType,
    pub is_curio: bool,
    pub quantity: i32,
}

#[derive(Event)]
pub struct CulturalLossEvent {
    pub magnitude: f32,
}

#[derive(Component)]
pub struct Pop {
    pub id: u32,
}

#[derive(Component)]
pub struct Morale {
    pub current: f32,
}

pub fn process_curio_export_system(
    mut events: EventReader<TradeCompletedEvent>,
    mut resources: ResMut<ColonyResources>,
    mut loss_events: EventWriter<CulturalLossEvent>,
) {
    for event in events.read() {
        if event.is_curio {
            resources.credits += 1000 * event.quantity; // Massive payout
            loss_events.send(CulturalLossEvent { magnitude: 20.0 * event.quantity as f32 });
        } else {
            resources.credits += 10 * event.quantity; // Normal payout
        }
    }
}

pub fn apply_cultural_loss_system(
    mut events: EventReader<CulturalLossEvent>,
    mut query: Query<&mut Morale, With<Pop>>,
) {
    for event in events.read() {
        for mut morale in query.iter_mut() {
            morale.current -= event.magnitude;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Flow:** The `CulturalLossEvent` should probably apply a timed `StressModifier(CulturalLoss, duration)` to Pops rather than an immediate flat deduction to `current` morale.
- **Dynamic Pricing:** The `1000` credit payout should scale down if the market is flooded with identical curios (supply/demand).
- **Curio Definition:** Add a `Curio` component to items on the map so they emit Beauty while local, reinforcing the loss when they leave.

## 6. Acceptance Criteria
- [ ] Selling a `Curio` tagged item grants significantly higher credits than base items.
- [ ] Exporting a Curio triggers a `CulturalLossEvent`.
- [ ] `apply_cultural_loss_system` correctly reduces pop morale.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- **Tagging:** Use a marker component `#[derive(Component)] pub struct Curio;` on specific resource entities before they are packed into cargo.
- **Lore Integration:** Connect the `CulturalLossEvent` to the chronicle system so players can see *why* their colony is depressed.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.

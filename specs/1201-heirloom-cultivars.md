# 1201: Heirloom Cultivars

## 1. Overview
**Layer:** 1
**Fantasy:** The emotional attachment to the food of the old homeworld clashing with the brutal efficiency needed to survive on a new frontier.
**Mechanic:** Colonists occasionally arrive with secretly smuggled "Heirloom Seeds" from their ancestral worlds. These crops are highly inefficient to grow in the new alien soil, requiring massive amounts of water and specialized care, but yield "Nostalgic Produce." Consuming this produce completely clears a Pop's "Homesickness" debuff and massively boosts their morale, but only for the specific sub-culture of Pops it belongs to.
**Emergence:** You try to mandate the planting of ultra-efficient, bio-engineered nutrient paste algae to survive a harsh winter. A faction of colonists refuses, secretly turning their life-support hydroponics bays into resource-intensive Heirloom gardens. They are happy and well-fed, but the rest of the colony is starving because the water reserves are being drained to grow sentimental tomatoes.
**Tension:** Do you violently uproot the Heirloom crops to enforce raw survival efficiency, devastating the morale of your founding colonists, or do you dedicate precious survival resources to maintaining a literal taste of home?

## 2. Dependencies
- Layer 1 Farming / Hydroponics systems
- Water grid / Resource consumption
- Pop Needs (Homesickness, Morale)
- Pop Cultural Traits

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_heirloom_crops_consume_excess_water() {
        let mut app = App::new();
        app.add_systems(Update, consume_water_system);

        // Standard crop
        let standard_id = app.world.spawn(Crop { water_requirement: 10.0, is_heirloom: false }).id();
        // Heirloom crop
        let heirloom_id = app.world.spawn(Crop { water_requirement: 10.0, is_heirloom: true }).id();

        app.world.insert_resource(WaterSupply { amount: 100.0 });

        app.update();

        // The water supply should be reduced, but the heirloom should have consumed significantly more (e.g., 5x)
        let water = app.world.get_resource::<WaterSupply>().unwrap();
        // Assuming 10 for standard + 50 (10*5) for heirloom = 60 consumed, 40 left.
        assert_eq!(water.amount, 40.0, "Heirloom crops must consume drastically more water");
    }

    #[test]
    fn test_nostalgic_produce_cures_homesickness() {
        let mut app = App::new();
        app.add_systems(Update, consume_food_system);

        // Spawn a pop with severe homesickness
        let pop_id = app.world.spawn((Pop, Homesickness(100.0), SubCulture::FoundingEra)).id();

        // Spawn nostalgic produce matching their culture
        let produce_id = app.world.spawn(NostalgicProduce { target_culture: SubCulture::FoundingEra }).id();

        // Add a consume event
        app.world.insert_resource(Events::<ConsumeEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<ConsumeEvent>>().unwrap();
        events.send(ConsumeEvent { pop: pop_id, item: produce_id });

        app.update();

        // Homesickness should be entirely cleared
        let homesickness = app.world.get::<Homesickness>(pop_id).unwrap();
        assert_eq!(homesickness.0, 0.0, "Nostalgic Produce must completely cure Homesickness for matching cultures");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Crop {
    pub water_requirement: f32,
    pub is_heirloom: bool,
}

#[derive(Resource)]
pub struct WaterSupply {
    pub amount: f32,
}

#[derive(PartialEq, Eq, Clone, Copy, Component)]
pub enum SubCulture {
    FoundingEra,
    Spacer,
    CoreWorlder,
}

#[derive(Component)]
pub struct NostalgicProduce {
    pub target_culture: SubCulture,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Homesickness(pub f32);

#[derive(Event)]
pub struct ConsumeEvent {
    pub pop: Entity,
    pub item: Entity,
}

pub fn consume_water_system(
    mut water: ResMut<WaterSupply>,
    crops: Query<&Crop>,
) {
    for crop in crops.iter() {
        let multiplier = if crop.is_heirloom { 5.0 } else { 1.0 };
        water.amount -= crop.water_requirement * multiplier;
    }
}

pub fn consume_food_system(
    mut consume_events: EventReader<ConsumeEvent>,
    mut pop_query: Query<(&mut Homesickness, &SubCulture), With<Pop>>,
    item_query: Query<&NostalgicProduce>,
) {
    for event in consume_events.read() {
        if let Ok((mut homesickness, culture)) = pop_query.get_mut(event.pop) {
            if let Ok(produce) = item_query.get(event.item) {
                if produce.target_culture == *culture {
                    homesickness.0 = 0.0;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently `consume_water_system` blindly subtracts water. It should handle shortages gracefully, where standard crops survive longer on low water than finicky heirloom crops.
- Pops should actively convert standard hydroponics bays to Heirloom bays if their `Homesickness` gets too high, representing the "secret smuggling" mechanic. This requires adding a system that mutates standard `Crop` entities into heirloom variants.
- The `SubCulture` enum will likely need to be a String or ID reference in a full implementation to support dynamically generated cultures.

## 6. Acceptance Criteria
- [ ] Heirloom crops consume significantly more water/resources than standard crops.
- [ ] Consuming `NostalgicProduce` reduces or clears `Homesickness` for pops of matching culture.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.
- [ ] 0 clippy warnings.

## 7. Technical Guidance
- Integration: This spec provides the base entities, but ensure they are integrated with the existing Utility AI so pops will actually seek out and eat the `NostalgicProduce`.
- Use Bevy's `EventReader` correctly for `ConsumeEvent` processing.

## 8. Questions
*Builder: Add any questions here.*

# Spec 434: The Parasitic Habitation Block

## 1. Overview
If housing quality drops too low while population density is high, a "Phantom Block" forms. It's a physical slum that doesn't appear on your official registry. Pops living there consume no colony resources but produce nothing and slowly accumulate extreme radical ideologies.

## 2. Dependencies
- `007-housing`
- `064-room-quality`
- `197-civic-ideology`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_phantom_block_formation() {
        let mut app = App::new();
        app.insert_resource(HousingStats { quality_avg: 10.0, pop_density: 150.0 });
        app.add_systems(Update, evaluate_housing_conditions_system);

        app.update();

        let blocks = app.world().query::<&PhantomBlock>().iter(app.world()).count();
        assert_eq!(blocks, 1);
    }

    #[test]
    fn test_phantom_block_radicals() {
        let mut app = App::new();
        app.add_systems(Update, phantom_block_radicalization_system);

        let entity = app.world_mut().spawn((
            ResidentInPhantomBlock,
            IdeologyRadicalization { level: 0.0 },
        )).id();

        app.update();

        let ideology = app.world().get::<IdeologyRadicalization>(entity).unwrap();
        assert_eq!(ideology.level, 5.0); // +5 per tick in block
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct HousingStats {
    pub quality_avg: f32,
    pub pop_density: f32,
}

#[derive(Component)]
pub struct PhantomBlock;

#[derive(Component)]
pub struct ResidentInPhantomBlock;

#[derive(Component)]
pub struct IdeologyRadicalization {
    pub level: f32,
}

pub fn evaluate_housing_conditions_system(
    mut commands: Commands,
    stats: Res<HousingStats>,
) {
    if stats.quality_avg < 20.0 && stats.pop_density > 100.0 {
        commands.spawn(PhantomBlock);
    }
}

pub fn phantom_block_radicalization_system(
    mut q_residents: Query<&mut IdeologyRadicalization, With<ResidentInPhantomBlock>>,
) {
    for mut ideology in q_residents.iter_mut() {
        ideology.level += 5.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate `HousingStats` dynamically by iterating over all housing entities and calculating moving averages for density and quality.
- Connect `PhantomBlock` creation to actually reserving empty space or unused rooms, removing them from the standard registry UI.
- Ensure Pops living in the `PhantomBlock` are excluded from the `ColonyResources` consumption calculations but still exist in the physics simulation.

## 6. Acceptance Criteria
- [ ] If average housing quality is low and density is high, a `PhantomBlock` is spawned.
- [ ] Pops with `ResidentInPhantomBlock` steadily increase in `IdeologyRadicalization`.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Integration with the Civic Ideology and Unrest systems is key; extreme radicalization should eventually trigger a rebellion event.
- Pops moving into the `PhantomBlock` should probably be those who are homeless or have the lowest current housing quality.

## 8. Questions
- Can the player actively demolish a `PhantomBlock`, or does doing so instantly trigger the radicalized pops to riot?
*Architect:* Yes, you can demolish a `PhantomBlock` by deconstructing the underlying terrain or rooms, but doing so instantly triggers a `RadicalRiotEvent` spawning hostile pops.

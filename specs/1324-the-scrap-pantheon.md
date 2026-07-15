# 1324: The Scrap Pantheon

## 1. Overview
The cast-offs of an industrial civilization become objects of worship for those left behind. On highly polluted or abandoned industrial sectors, low-status Pops can begin scavenging broken machinery and piecing together massive, non-functional "Idols" from the scrap. These idols generate a localized religious fervor, replacing traditional needs with a desire to "feed" the idol more high-tech components.

## 2. Dependencies
- `044` Horticulture & Beauty (for base Beauty mechanics, which pollution interacts with)
- `354` Industrial Byproducts (for waste/slag items)
- `147` Secret Societies (for handling the localized cult faction)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::scrap_pantheon::{ScrapCult, ScrapIdol, spawn_scrap_idol_system};
    use crate::layer1::map::BeautyGrid;
    use crate::layer1::resources::Inventory;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_idol_spawns_in_polluted_areas() {
        let mut world = World::new();

        let mut beauty = BeautyGrid::new(10, 10);
        beauty.set(5, 5, -50.0); // Extremely polluted
        world.insert_resource(beauty);

        // Add some pops nearby
        world.spawn((Pop::new(), Transform::from_xyz(5.0, 5.0, 0.0)));
        world.spawn((Pop::new(), Transform::from_xyz(5.0, 6.0, 0.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_scrap_idol_system);
        schedule.run(&mut world);

        // Verify idol is spawned
        let mut idol_query = world.query::<&ScrapIdol>();
        assert_eq!(idol_query.iter(&world).count(), 1, "An idol should spawn in the highly polluted area");
    }

    #[test]
    fn test_pops_join_cult_near_idol() {
        let mut world = World::new();

        // Spawn idol
        let idol_entity = world.spawn((ScrapIdol, Transform::from_xyz(5.0, 5.0, 0.0))).id();

        // Spawn pop nearby
        let pop_entity = world.spawn((Pop::new(), Transform::from_xyz(5.0, 6.0, 0.0))).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(recruit_cultists_system);
        schedule.run(&mut world);

        // Verify pop joined cult
        assert!(world.get::<ScrapCult>(pop_entity).is_some(), "Pop near idol should join the cult");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::BeautyGrid;
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct ScrapIdol;

#[derive(Component)]
pub struct ScrapCult;

pub fn spawn_scrap_idol_system(
    mut commands: Commands,
    beauty: Option<Res<BeautyGrid>>,
    pops: Query<(Entity, &Transform), With<Pop>>,
) {
    let beauty = match beauty {
        Some(b) => b,
        None => return,
    };

    // Check for high pollution (negative beauty)
    for x in 0..beauty.width() {
        for y in 0..beauty.height() {
            if beauty.get(x as i32, y as i32) <= -50.0 {
                // If polluted, check if there are pops nearby to build it
                let mut pops_nearby = 0;
                for (_, transform) in pops.iter() {
                    let dist = ((transform.translation.x - x as f32).powi(2) +
                               (transform.translation.y - y as f32).powi(2)).sqrt();
                    if dist < 5.0 {
                        pops_nearby += 1;
                    }
                }

                // Spawn idol if enough pops are around
                if pops_nearby >= 2 {
                    commands.spawn((
                        ScrapIdol,
                        Transform::from_xyz(x as f32, y as f32, 0.0)
                    ));
                    return; // Just spawn one for now
                }
            }
        }
    }
}

pub fn recruit_cultists_system(
    mut commands: Commands,
    idols: Query<&Transform, With<ScrapIdol>>,
    pops: Query<(Entity, &Transform), (With<Pop>, Without<ScrapCult>)>,
) {
    for idol_transform in idols.iter() {
        for (pop_entity, pop_transform) in pops.iter() {
            let dist = ((idol_transform.translation.x - pop_transform.translation.x).powi(2) +
                       (idol_transform.translation.y - pop_transform.translation.y).powi(2)).sqrt();

            if dist < 10.0 {
                commands.entity(pop_entity).insert(ScrapCult);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `IdolNeeds` component that functions as a "hunger" for the idol, causing cultists to prioritize gathering scrap resources (Slag, Metal) and depositing them at the idol's location.
- Cultists should ignore regular leisure/social needs while near the idol, getting their fulfillment purely from "feeding" it.
- Tie the size/influence radius of the idol to the amount of scrap it has consumed.

## 6. Acceptance Criteria
- [ ] Tests pass
- [ ] Scrap Idols spawn in highly polluted areas (beauty <= -50.0).
- [ ] Pops near an idol join the `ScrapCult`.
- [ ] Test coverage >85%.

## 7. Technical Guidance
- Ensure the `BeautyGrid` is properly mocked/setup in the tests.
- Cult recruitment should likely be probabilistic in the future, rather than an instant 100% conversion based on radius.

## 8. Questions
*Builder: add questions here if spec is unclear.*

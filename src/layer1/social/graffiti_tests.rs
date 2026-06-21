#[cfg(test)]
mod tests {

    use crate::layer1::architecture::Building;
    use crate::layer1::architecture::BuildingType;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::psychology::stress::StressTracker;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer1::social::graffiti::*;
    use crate::layer1::social::morale::Morale;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_creative_pop_creates_graffiti_under_high_stress() {
        let mut world = World::new();

        // Setup building
        let building_id = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Setup creative pop with high stress
        let mut traits = Traits::default();
        traits.add(Trait::Artistic); // Using Artistic
        world.spawn((
            Pop,
            traits,
            StressTracker {
                accumulated_stress: 90.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run graffiti generation system
        propaganda_graffiti_system(&mut world);

        // Check if graffiti component was added to the building
        assert!(
            world.get::<RebelliousGraffiti>(building_id).is_some(),
            "Highly stressed creative pop should tag the building"
        );
    }

    #[test]
    fn test_graffiti_aura_effects() {
        let mut world = World::new();

        // Setup building with graffiti
        let _building_id = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                RebelliousGraffiti { intensity: 1.0 },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Setup a worker pop nearby
        let pop_id = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Morale::default(),
                GraffitiEfficiency { current: 1.0 },
            ))
            .id();

        // Run aura effect system
        graffiti_aura_system(&mut world);

        // Check effects
        let eff = world.get::<GraffitiEfficiency>(pop_id).unwrap();
        let morale = world.get::<Morale>(pop_id).unwrap();

        assert!(eff.current < 1.0, "Graffiti should lower efficiency");
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.value > 0.0 && m.label == "Venting: Rebellious Graffiti"),
            "Graffiti should boost morale"
        );
    }
}

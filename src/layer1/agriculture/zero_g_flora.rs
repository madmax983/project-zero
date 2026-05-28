use crate::layer1::economy::resources::ResourceType;
use crate::layer2::system::GravityLevel;
use bevy::prelude::*;

#[derive(Component)]
pub struct Depressurized;

#[derive(Component)]
pub struct ZeroGCropYield {
    pub amount: f32,
}

pub struct ZeroGResourceYield {
    pub resource_type: ResourceType,
    pub amount: f32,
}

#[derive(Default, Clone)]
pub struct ZeroGGrowthConditions {
    pub requires_microgravity: bool,
}

pub struct ZeroGCrop {
    pub name: String,
    pub conditions: ZeroGGrowthConditions,
    pub yield_resource: ResourceType,
    pub base_growth_time: f32,
}

impl ZeroGCrop {
    pub fn can_grow(&self, world: &World, entity: Entity) -> bool {
        if self.conditions.requires_microgravity {
            if let Some(gravity) = world.get::<GravityLevel>(entity) {
                *gravity == GravityLevel::MicroGravity || *gravity == GravityLevel::ZeroG
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn harvest(&self) -> ZeroGResourceYield {
        ZeroGResourceYield {
            resource_type: match self.yield_resource {
                ResourceType::HyperValuable => ResourceType::HyperValuable,
                _ => ResourceType::Food,
            },
            amount: 10.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct DepressurizationEvent(pub Entity);

pub fn handle_depressurization(
    mut commands: Commands,
    query: Query<(Entity, &Depressurized), Added<Depressurized>>,
    mut event_writer: EventWriter<DepressurizationEvent>,
) {
    for (entity, _) in query.iter() {
        commands.entity(entity).remove::<ZeroGCropYield>();
        event_writer.send(DepressurizationEvent(entity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::station::Station;
use crate::layer2::fleet::StationType;

    #[test]
    fn test_zero_g_flora_growth_requires_microgravity() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let growth_conditions = ZeroGGrowthConditions {
            requires_microgravity: true,
        };

        let crop = ZeroGCrop {
            name: "Void-Orchid".to_string(),
            conditions: growth_conditions,
            yield_resource: ResourceType::HyperValuable,
            base_growth_time: 100.0,
        };

        // Act & Assert
        // Test planetary growth (no microgravity component)
        let planetary_farm = app.world_mut().spawn_empty().id();
        let result = crop.can_grow(app.world(), planetary_farm);
        assert!(
            !result,
            "Zero-G Flora should not grow on a planetary surface."
        );

        // Test orbital growth (with microgravity component)
        let orbital_farm = app.world_mut().spawn(GravityLevel::MicroGravity).id();
        let result2 = crop.can_grow(app.world(), orbital_farm);
        assert!(
            result2,
            "Zero-G Flora should grow in a microgravity environment."
        );
    }

    #[test]
    fn test_zero_g_flora_yields_high_value_resource() {
        // Arrange
        let crop = ZeroGCrop {
            name: "Stellar-Vine".to_string(),
            conditions: ZeroGGrowthConditions {
                requires_microgravity: true,
            },
            yield_resource: ResourceType::HyperValuable,
            base_growth_time: 200.0,
        };

        // Act
        let resource_yield = crop.harvest();

        // Assert
        assert!(matches!(
            resource_yield.resource_type,
            ResourceType::HyperValuable
        ));
        assert_eq!(resource_yield.amount, 10.0);
    }

    #[test]
    fn test_micro_meteorite_strike_destroys_crop() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DepressurizationEvent>();
        app.add_systems(Update, handle_depressurization);

        // Setup an orbital hydroponics bay with crops
        let farm = app
            .world_mut()
            .spawn((
                GravityLevel::MicroGravity,
                ZeroGCropYield { amount: 50.0 },
                Station {
                    station_type: StationType::Hydroponics,
                },
            ))
            .id();

        // Act
        // Simulate depressurization event (e.g. from meteorite)
        app.world_mut().entity_mut(farm).insert(Depressurized);
        app.update();

        // Assert
        let crop_yield = app.world().get::<ZeroGCropYield>(farm);
        assert!(
            crop_yield.is_none() || crop_yield.unwrap().amount == 0.0,
            "Crops must be destroyed upon depressurization."
        );

        let events = app.world().resource::<Events<DepressurizationEvent>>();
        assert!(
            !events.is_empty(),
            "DepressurizationEvent should be emitted"
        );
    }
}

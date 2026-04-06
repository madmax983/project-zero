use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::building::Building;

/// Tracks famine stress (e.g. going without food).
#[derive(Component)]
pub struct FamineStress(pub u32);

/// Represents a Pop's charisma.
#[derive(Component)]
pub struct Charisma(pub u32);

/// Identifies a pop that has become a symbiotic cultist.
#[derive(Component)]
pub struct SymbioticCultist;

/// Cultists derive energy from sunlight. This component acts as their 'sunlight' need.
#[derive(Component)]
pub struct CultSunlightNeed(pub u32);

/// Designates a building as a workplace that may or may not have sunlight access.
#[derive(Component)]
pub struct Workplace {
    pub sunlight_access: bool,
}

/// Event emitted when the cult demands an infrastructure change.
#[derive(Event)]
pub struct CultDemandEvent {
    pub building: Entity,
    pub demand_type: DemandType,
}

#[derive(PartialEq)]
pub enum DemandType {
    SunlightAccess,
}

/// Query type for Symbiotic Cult formation.
pub type CultFormationQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static FamineStress, &'static Charisma),
    (With<Pop>, Without<SymbioticCultist>),
>;

/// A highly stressed, charismatic pop can form a symbiotic cult.
pub fn spawn_symbiotic_cult_system(mut commands: Commands, query: CultFormationQuery) {
    for (entity, stress, charisma) in query.iter() {
        if stress.0 >= 100 && charisma.0 >= 80 {
            commands.entity(entity).insert(SymbioticCultist);
        }
    }
}

/// Mutates the cultist's biology: continuously removes hunger need (clamping to 1.0)
/// and ensures they have a sunlight need component.
pub fn apply_cult_biology_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Needs, Option<&CultSunlightNeed>), With<SymbioticCultist>>,
) {
    for (entity, mut needs, sunlight_opt) in query.iter_mut() {
        needs.hunger = 1.0;
        if sunlight_opt.is_none() {
            commands.entity(entity).insert(CultSunlightNeed(100));
        }
    }
}

/// Generates demands for workplaces without sunlight access.
/// Uses a `Local` counter to prevent spamming events every frame.
pub fn cult_infrastructure_demand_system(
    cultists: Query<&SymbioticCultist>,
    workplaces: Query<(Entity, &Workplace), With<Building>>,
    mut events: EventWriter<CultDemandEvent>,
    mut cooldown: Local<u32>,
) {
    if *cooldown > 0 {
        *cooldown -= 1;
        return;
    }

    if cultists.iter().count() >= 3 {
        for (entity, workplace) in workplaces.iter() {
            if !workplace.sunlight_access {
                events.send(CultDemandEvent {
                    building: entity,
                    demand_type: DemandType::SunlightAccess,
                });
            }
        }
        *cooldown = 100; // Wait 100 ticks before demanding again.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_cult_formation() {
        let mut app = App::new();
        app.add_systems(Update, spawn_symbiotic_cult_system);

        // Arrange
        let pop = app.world_mut().spawn((
            Pop,
            FamineStress(100),
            Charisma(80),
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().get::<SymbioticCultist>(pop).is_some(), "Highly stressed, charismatic pop should found a cult during famine");
    }

    #[test]
    fn test_cultist_needs_altered() {
        let mut app = App::new();
        app.add_systems(Update, apply_cult_biology_system);

        // Arrange
        let pop = app.world_mut().spawn((
            Pop,
            SymbioticCultist,
            Needs {
                hunger: 0.5,
                ..Default::default()
            }
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        // Since the real Needs has 'hunger', satisfying it (1.0) equates to 0 need for food.
        assert_eq!(needs.hunger, 1.0, "Cultist should not need normal food");

        let sunlight = app.world().get::<CultSunlightNeed>(pop).unwrap();
        assert!(sunlight.0 > 0, "Cultist should need sunlight");
    }

    #[test]
    fn test_infrastructure_demand() {
        let mut app = App::new();
        app.add_event::<CultDemandEvent>();
        app.add_systems(Update, cult_infrastructure_demand_system);

        // Arrange
        app.world_mut().spawn((Pop, SymbioticCultist));
        app.world_mut().spawn((Pop, SymbioticCultist));
        app.world_mut().spawn((Pop, SymbioticCultist)); // High cult population

        let building = app.world_mut().spawn((
            Building { building_type: BuildingType::Housing },
            Workplace { sunlight_access: false }
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<CultDemandEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        assert!(reader.read(events).any(|e| e.building == building && e.demand_type == DemandType::SunlightAccess), "Cult should demand sunlight access in workplaces");
    }
}

# The Symbiotic Cult

**1. Overview**
A new religion that demands physical sacrifice, turning colonists into something more, or less, than human. A charismatic Pop founds a "Symbiotic Cult" focused on a specific native flora or fauna. Cult members intentionally infect themselves, gaining unique buffs (e.g., photosynthesis, natural armor) but altering their Needs (requiring raw sunlight or specific alien meat instead of normal food). As the cult grows, they demand the colony's infrastructure be altered to suit their new biology. The tension lies in embracing a bizarre, post-human evolution that solves immediate resource crises vs. maintaining a baseline human civilization and risking a holy war against your own adapted citizens.

**2. Dependencies**
- `layer1::needs::Needs`
- `layer1::factions::Faction`
- `layer1::events::SimulationEvent`
- `layer1::biology::Traits`

**3. RED Phase: Tests First**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

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
                food: 50,
                water: 50,
                sunlight: 0,
            }
        )).id();

        // Act
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(pop).unwrap();
        assert_eq!(needs.food, 0, "Cultist should not need normal food");
        assert!(needs.sunlight > 0, "Cultist should need sunlight");
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
            Building,
            Workplace { sunlight_access: false }
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<CultDemandEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).any(|e| e.building == building && e.demand_type == DemandType::SunlightAccess), "Cult should demand sunlight access in workplaces");
    }
}
```

**4. GREEN Phase: Minimal Implementation**

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct FamineStress(pub u32);

#[derive(Component)]
pub struct Charisma(pub u32);

#[derive(Component)]
pub struct SymbioticCultist;

#[derive(Component)]
pub struct Needs {
    pub food: u32,
    pub water: u32,
    pub sunlight: u32,
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Workplace {
    pub sunlight_access: bool,
}

#[derive(Event)]
pub struct CultDemandEvent {
    pub building: Entity,
    pub demand_type: DemandType,
}

#[derive(PartialEq)]
pub enum DemandType {
    SunlightAccess,
}

pub fn spawn_symbiotic_cult_system(mut commands: Commands, query: Query<(Entity, &FamineStress, &Charisma), Without<SymbioticCultist>>) {
    for (entity, stress, charisma) in query.iter() {
        if stress.0 >= 100 && charisma.0 >= 80 {
            commands.entity(entity).insert(SymbioticCultist);
        }
    }
}

pub fn apply_cult_biology_system(mut query: Query<&mut Needs, Added<SymbioticCultist>>) {
    for mut needs in query.iter_mut() {
        needs.food = 0;
        needs.sunlight = 100; // Arbitrary high value
    }
}

pub fn cult_infrastructure_demand_system(
    cultists: Query<&SymbioticCultist>,
    workplaces: Query<(Entity, &Workplace)>,
    mut events: EventWriter<CultDemandEvent>,
) {
    if cultists.iter().count() >= 3 {
        for (entity, workplace) in workplaces.iter() {
            if !workplace.sunlight_access {
                events.send(CultDemandEvent {
                    building: entity,
                    demand_type: DemandType::SunlightAccess,
                });
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Consider creating a more generalized system for mutating `Needs` based on various traits or buffs.
- Replace magic numbers (e.g., `100` for famine stress, `3` for cult population threshold) with constants or configurable resource parameters.
- Handle different types of cults (e.g., meat-eaters vs photosynthesizers) by creating an enum `CultType` on the `SymbioticCultist` component.
- Ensure the `CultDemandEvent` feeds into a broader faction/unrest system.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] A highly stressed, charismatic pop can form a cult
- [ ] Cultists have their food needs replaced by sunlight needs
- [ ] A high population of cultists generates infrastructure demands

**7. Technical Guidance**
- Make sure to add `CultDemandEvent` to the app's event registry (`app.add_event::<CultDemandEvent>()`).
- Integrating this with existing need-fulfillment systems might require refactoring how `Needs` are consumed (e.g., pops consuming sunlight passively vs actively seeking out food).
- Use `Added<SymbioticCultist>` to ensure biology is only mutated once, preventing overwrites if other systems also touch `Needs`.

**8. Questions**
*Builder: add questions here if spec is unclear.*

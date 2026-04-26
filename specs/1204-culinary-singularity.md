# 1204: The Culinary Singularity

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** A society obsessed with achieving the perfect dish inadvertently creates a galaxy-spanning crisis.
**Mechanic:** As your empire advances, a faction called "The Gastronomers" emerges. They demand increasingly exotic and dangerous ingredients from across the galaxy to achieve the "Culinary Singularity"—a meal so perfect it induces a state of permanent enlightenment. Fulfilling their requests requires diverting military fleets to hunt Leviathans or mining unstable stars for rare isotopes.
**Emergence:** The Gastronomers demand the core of a specific, volatile gas giant to use as a cooking heat source. You comply, but the extraction process destabilizes the planet, causing it to go supernova. The resulting dish is incredible, granting permanent maximum morale to your capital, but you just destroyed an entire inhabited star system to make a really good soup.
**Tension:** The immense, empire-wide buffs provided by the Culinary Singularity vs. the apocalyptic lengths you must go to in order to procure the ingredients.

## 2. Dependencies
- Layer 3 Faction demands / Quests
- Layer 3 Fleet missions (Hunting, Extraction)
- Layer 1 Morale / Buffs system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_gastronomer_faction_generates_exotic_demand() {
        let mut app = App::new();
        app.add_systems(Update, generate_faction_demands_system);

        // Spawn Gastronomer faction at high influence
        app.world.spawn((Faction::Gastronomers, Influence(80.0)));
        app.world.insert_resource(Events::<FactionDemandEvent>::default());

        app.update();

        let demand_events = app.world.get_resource::<Events<FactionDemandEvent>>().unwrap();
        let mut reader = demand_events.get_cursor();
        let events: Vec<_> = reader.read(demand_events).collect();

        assert_eq!(events.len(), 1, "High influence Gastronomers should generate a demand");
        assert!(matches!(events[0].demand_type, DemandType::LeviathanMeat | DemandType::StarCore), "Demand must be exotic/dangerous");
    }

    #[test]
    fn test_culinary_singularity_grants_permanent_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_culinary_singularity_buff);

        // Spawn a pop with normal morale
        let pop_id = app.world.spawn((Pop, Morale(50.0), MaxMorale(100.0))).id();

        app.world.insert_resource(Events::<CulinarySingularityAchievedEvent>::default());
        let mut events = app.world.get_resource_mut::<Events<CulinarySingularityAchievedEvent>>().unwrap();
        events.send(CulinarySingularityAchievedEvent);

        app.update();

        let morale = app.world.get::<Morale>(pop_id).unwrap();
        let max_morale = app.world.get::<MaxMorale>(pop_id).unwrap();

        // Permanent max morale buff
        assert_eq!(max_morale.0, 200.0, "Singularity should permanently increase max morale");
        assert_eq!(morale.0, 200.0, "Singularity should maximize current morale");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub enum Faction {
    Gastronomers,
    Military,
}

#[derive(Component)]
pub struct Influence(pub f32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DemandType {
    LeviathanMeat,
    StarCore,
    StandardFood,
}

#[derive(Event)]
pub struct FactionDemandEvent {
    pub faction: Entity,
    pub demand_type: DemandType,
}

#[derive(Event)]
pub struct CulinarySingularityAchievedEvent;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Morale(pub f32);

#[derive(Component)]
pub struct MaxMorale(pub f32);

pub fn generate_faction_demands_system(
    factions: Query<(Entity, &Faction, &Influence)>,
    mut demand_events: EventWriter<FactionDemandEvent>,
) {
    for (entity, faction, influence) in factions.iter() {
        if matches!(faction, Faction::Gastronomers) && influence.0 > 75.0 {
            // Simplified logic: trigger exotic demand if influence is high
            demand_events.send(FactionDemandEvent {
                faction: entity,
                demand_type: DemandType::StarCore,
            });
        }
    }
}

pub fn apply_culinary_singularity_buff(
    mut singularity_events: EventReader<CulinarySingularityAchievedEvent>,
    mut pops: Query<(&mut Morale, &mut MaxMorale), With<Pop>>,
) {
    for _ in singularity_events.read() {
        for (mut morale, mut max_morale) in pops.iter_mut() {
            max_morale.0 = 200.0; // Permanent cap increase
            morale.0 = 200.0;     // Instant fill
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Demands shouldn't trigger every frame. Implement a cooldown or track active quests in a `ActiveFactionDemands` resource.
- Introduce actual negative side effects for harvesting `StarCore` (e.g. firing a `SystemDestabilizedEvent` that Builders will handle).
- Ensure the morale buff applies to future Pops that spawn, perhaps by writing it to a `GlobalEmpireBuffs` resource.

## 6. Acceptance Criteria
- [ ] Gastronomer faction demands exotic resources like Leviathan Meat or Star Cores.
- [ ] Fulfilling the final demand triggers `CulinarySingularityAchievedEvent`.
- [ ] `CulinarySingularityAchievedEvent` permanently raises and maxes out Pop morale.
- [ ] All RED phase tests pass.
- [ ] Coverage >= 85%.

## 7. Technical Guidance
- The scope is mostly around the demand generation and the buff application. The actual combat with Leviathans or extracting Star Cores should be assumed as separate Layer 3 mechanics.

## 8. Questions
*Builder: Add any questions here.*

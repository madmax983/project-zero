# 635: The Empathy Cascade

## 1. Overview

A psychological contagion that paralyzes a planet not with fear, but with an overwhelming, shared sorrow. If a colony experiences a massive tragedy (e.g., thousands of Pops dying in an orbital bombardment), the intense collective trauma spawns an "Empathy Cascade" event. This operates like a psychic disease. Pops who interact with trauma survivors inherit a portion of their depression. If the Cascade spreads to a spaceport, it can infect departing trade fleets, spreading the depression penalty to other planets in the system.

## 2. Dependencies

- `031` Pop Morale
- `047` Pop Relationships

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_empathy_cascade_trigger() {
        // Arrange
        let mut app = App::new();
        app.add_event::<MassDeathEvent>()
           .add_event::<EmpathyCascadeEvent>()
           .add_systems(Update, trigger_cascade);

        app.world_mut().insert_resource(DeathCount(0));

        // Act
        app.world_mut().send_event(MassDeathEvent { count: 1000 });
        app.update();

        // Assert
        let events = app.world().resource::<Events<EmpathyCascadeEvent>>();
        assert_eq!(events.iter().count(), 1, "MassDeathEvent should trigger EmpathyCascadeEvent");
    }

    #[test]
    fn test_cascade_infection_through_interaction() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopInteractionEvent>()
           .add_systems(Update, spread_cascade);

        let survivor = app.world_mut().spawn((
            Pop { morale: 10.0 },
            EmpathyCascade { intensity: 50.0 },
        )).id();

        let healthy_pop = app.world_mut().spawn((
            Pop { morale: 80.0 },
        )).id();

        // Act
        app.world_mut().send_event(PopInteractionEvent {
            pop_a: survivor,
            pop_b: healthy_pop,
        });
        app.update();

        // Assert
        let new_infection = app.world().get::<EmpathyCascade>(healthy_pop).unwrap();
        assert!(new_infection.intensity > 0.0, "Healthy pop should catch depression from interaction");

        let healthy_morale = app.world().get::<Pop>(healthy_pop).unwrap().morale;
        assert!(healthy_morale < 80.0, "Healthy pop morale should drop after infection");
    }

    #[test]
    fn test_fleet_infection() {
        // Arrange
        let mut app = App::new();
        app.add_event::<FleetDockEvent>()
           .add_systems(Update, infect_fleet);

        let spaceport = app.world_mut().spawn((
            Spaceport { cascade_level: 20.0 },
        )).id();

        let fleet = app.world_mut().spawn((
            Fleet { infected: false },
        )).id();

        // Act
        app.world_mut().send_event(FleetDockEvent { spaceport, fleet });
        app.update();

        // Assert
        let infected_fleet = app.world().get::<Fleet>(fleet).unwrap();
        assert!(infected_fleet.infected, "Fleet docking at infected spaceport should become infected");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct EmpathyCascade {
    pub intensity: f32,
}

#[derive(Event)]
pub struct MassDeathEvent {
    pub count: u32,
}

#[derive(Event)]
pub struct EmpathyCascadeEvent;

#[derive(Event)]
pub struct PopInteractionEvent {
    pub pop_a: Entity,
    pub pop_b: Entity,
}

#[derive(Event)]
pub struct FleetDockEvent {
    pub spaceport: Entity,
    pub fleet: Entity,
}

#[derive(Resource)]
pub struct DeathCount(pub u32);

#[derive(Component)]
pub struct Pop {
    pub morale: f32,
}

#[derive(Component)]
pub struct Spaceport {
    pub cascade_level: f32,
}

#[derive(Component)]
pub struct Fleet {
    pub infected: bool,
}

pub fn trigger_cascade(
    mut events: EventReader<MassDeathEvent>,
    mut cascade_events: EventWriter<EmpathyCascadeEvent>,
    mut death_count: ResMut<DeathCount>,
) {
    for event in events.read() {
        death_count.0 += event.count;
        if death_count.0 >= 1000 {
            cascade_events.send(EmpathyCascadeEvent);
        }
    }
}

pub fn spread_cascade(
    mut events: EventReader<PopInteractionEvent>,
    mut commands: Commands,
    mut pops: Query<(Entity, &mut Pop, Option<&EmpathyCascade>)>,
) {
    for event in events.read() {
        let mut infection_intensity = 0.0;

        if let Ok((_, _, Some(cascade))) = pops.get(event.pop_a) {
            infection_intensity = cascade.intensity;
        } else if let Ok((_, _, Some(cascade))) = pops.get(event.pop_b) {
            infection_intensity = cascade.intensity;
        }

        if infection_intensity > 0.0 {
            let target = if pops.get(event.pop_a).unwrap().2.is_none() { event.pop_a } else { event.pop_b };
            if let Ok((ent, mut pop, None)) = pops.get_mut(target) {
                let transmitted_intensity = infection_intensity * 0.5;
                commands.entity(ent).insert(EmpathyCascade { intensity: transmitted_intensity });
                pop.morale -= transmitted_intensity;
            }
        }
    }
}

pub fn infect_fleet(
    mut events: EventReader<FleetDockEvent>,
    spaceports: Query<&Spaceport>,
    mut fleets: Query<&mut Fleet>,
) {
    for event in events.read() {
        if let Ok(spaceport) = spaceports.get(event.spaceport) {
            if spaceport.cascade_level > 10.0 {
                if let Ok(mut fleet) = fleets.get_mut(event.fleet) {
                    fleet.infected = true;
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Decay Mechanism:** The `EmpathyCascade` intensity should naturally decay over time to prevent a permanent soft-lock on morale.
- **Configurable Thresholds:** The `1000` death count and `10.0` spaceport cascade level should be stored in a tunable configuration resource.
- **Bi-directional Spread:** Pop interaction currently assumes one infected and one uninfected. Consider the case where both are infected; their intensities might average out or stack with diminishing returns.
- **Layer 3 Integration:** A docked, infected fleet needs a system to transmit the infection to the destination spaceport when it docks elsewhere.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A mass death event reliably triggers the empathy cascade.
- [ ] Pops infect each other during interaction, suffering morale drops.
- [ ] Fleets docking at infected spaceports catch the infection.

## 7. Technical Guidance

- Utilize the existing `PopInteractionEvent` from `047-pop-relationships.md`.
- Ensure spaceport infection levels correlate with the number of highly infected pops in the vicinity or working at the spaceport.
- Implement a medical or social mitigation strategy (e.g., therapy centers or temporary social isolation) to combat the spread.

## 8. Questions

*Builder: add questions here if spec is unclear.*

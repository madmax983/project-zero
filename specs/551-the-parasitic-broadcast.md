# Spec 551: The Parasitic Broadcast

## 1. Overview
A catchy pop song from a dead empire is actually a hostile, self-replicating memetic virus. The colony's comms array picks it up, giving Pops a massive "Entertained" buff. However, the song is a memetic parasite. Infected Pops spend their work hours humming the tune, drastically reducing their productivity, and subconsciously re-wiring colony machinery to broadcast the signal back into space, drawing the attention of automated Layer 3 exterminator fleets.

## 2. Dependencies
- `016` Utility AI System (needs)
- `378` Comms Relay (to receive the signal)
- Faction/Threat event bridge from Layer 3.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_ai::WorkSpeed;

    #[test]
    fn test_parasitic_broadcast_gives_massive_leisure_buff_but_reduces_work_speed() {
        let mut app = App::new();
        app.add_systems(Update, process_parasitic_infection_system);

        let pop = app.world_mut().spawn((
            Pop,
            Needs { leisure: 50.0, ..Default::default() },
            WorkSpeed { multiplier: 1.0 },
            ParasiticInfection { active: true },
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        let speed = app.world().get::<WorkSpeed>(pop).unwrap();

        // Leisure should be maxed, but work speed severely reduced
        assert_eq!(needs.leisure, 100.0);
        assert!(speed.multiplier < 1.0);
    }

    #[test]
    fn test_infected_pops_generate_exterminator_threat() {
        let mut app = App::new();
        app.add_event::<ExterminatorThreatEvent>();
        app.add_systems(Update, accumulate_parasitic_threat_system);
        app.insert_resource(ExterminatorThreat { level: 0.0 });
        app.insert_resource(Time::default());

        app.world_mut().spawn((Pop, ParasiticInfection { active: true }));

        // Advance time
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1.0));
        app.update();

        let threat = app.world().resource::<ExterminatorThreat>();
        assert!(threat.level > 0.0, "Infected pops must accumulate threat level");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::WorkSpeed;

#[derive(Component)]
pub struct ParasiticInfection {
    pub active: bool,
}

#[derive(Resource)]
pub struct ExterminatorThreat {
    pub level: f32,
}

#[derive(Event)]
pub struct ExterminatorThreatEvent;

pub fn process_parasitic_infection_system(
    mut query: Query<(&mut Needs, &mut WorkSpeed, &ParasiticInfection)>,
) {
    for (mut needs, mut speed, infection) in query.iter_mut() {
        if infection.active {
            needs.leisure = 100.0; // Infinite entertainment
            speed.multiplier = 0.5; // 50% work speed reduction
        }
    }
}

pub fn accumulate_parasitic_threat_system(
    time: Res<Time>,
    query: Query<&ParasiticInfection>,
    mut threat: ResMut<ExterminatorThreat>,
    mut events: EventWriter<ExterminatorThreatEvent>,
) {
    let dt = time.delta_secs();
    let infected_count = query.iter().filter(|i| i.active).count();

    if infected_count > 0 {
        threat.level += (infected_count as f32) * dt * 0.1;
    }

    if threat.level >= 100.0 {
        events.send(ExterminatorThreatEvent);
        threat.level = 0.0; // Reset after sending event
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `InfectionConfig` resource to manage the work speed penalty, threat accumulation rate, and the threshold for the Layer 3 event.
- The `ExterminatorThreatEvent` should spawn a hostile Layer 3 fleet that targets the Layer 1 colony via orbital bombardment or invasion.
- Implement a system to 'cure' the infection, perhaps by building a "Counter-Frequency Emitter" or initiating a colony-wide communications blackout (referencing Spec 126 Blackout Protocol).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Pops with `ParasiticInfection` have their `Leisure` need maximized.
- [ ] Pops with `ParasiticInfection` have their `WorkSpeed` reduced.
- [ ] Active infections accumulate `ExterminatorThreat` over time.
- [ ] Crossing the threat threshold emits an `ExterminatorThreatEvent`.

## 7. Technical Guidance
- The initial infection vector should be a random event originating from a `CommsRelay` or `CommandCenter`.
- The infection should spread between Pops when they interact socially (e.g., in a `Tavern` or adjacent tiles), turning them into "carriers" humming the tune.

## 8. Questions
*Builder: add questions here if spec is unclear.*

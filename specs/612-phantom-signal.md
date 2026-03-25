# The Phantom Signal

**1. Overview**
A deep-space listening post (Layer 2) detects a tantalizing signal promising incredible technology from an unknown sector (Layer 3). Following the signal requires massive resource investment and specialized research on Layer 1 to decipher. However, the signal is actually a complex, ancient memetic virus designed to waste resources.

**2. Dependencies**
- `layer2::sensors::ListeningPost`
- `layer1::research::ResearchProject`
- `layer1::production::Resource`

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_phantom_signal_consumes_research() {
        let mut app = App::new();
        app.add_systems(Update, phantom_signal_research_system);

        let project = app.world_mut().spawn((
            PhantomSignalProject { progress: 0.0, required: 1000.0, decoded: false },
        )).id();

        app.insert_resource(ResearchPoints(500.0));

        app.update();

        let state = app.world().get::<PhantomSignalProject>(project).unwrap();
        assert_eq!(state.progress, 500.0, "Research points should be consumed to progress the signal");
        assert_eq!(app.world().resource::<ResearchPoints>().0, 0.0, "Global research points should be depleted");
    }

    #[test]
    fn test_phantom_signal_viral_payload() {
        let mut app = App::new();
        app.add_systems(Update, phantom_signal_payload_system);

        app.world_mut().spawn((
            PhantomSignalProject { progress: 1000.0, required: 1000.0, decoded: false },
        ));

        let drill = app.world_mut().spawn((
            MiningDrill { efficiency: 1.0 },
        )).id();

        app.update();

        let modified_drill = app.world().get::<MiningDrill>(drill).unwrap();
        assert!(modified_drill.efficiency < 0.0, "Once decoded, the memetic virus should sabotage equipment");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PhantomSignalProject {
    pub progress: f32,
    pub required: f32,
    pub decoded: bool,
}

#[derive(Resource)]
pub struct ResearchPoints(pub f32);

#[derive(Component)]
pub struct MiningDrill {
    pub efficiency: f32,
}

pub fn phantom_signal_research_system(
    mut points: Option<ResMut<ResearchPoints>>,
    mut projects: Query<&mut PhantomSignalProject>,
) {
    if let Some(mut points) = points {
        for mut project in projects.iter_mut() {
            if !project.decoded && points.0 > 0.0 {
                let amount = points.0.min(project.required - project.progress);
                project.progress += amount;
                points.0 -= amount;
            }
        }
    }
}

pub fn phantom_signal_payload_system(
    mut projects: Query<&mut PhantomSignalProject>,
    mut drills: Query<&mut MiningDrill>,
) {
    for mut project in projects.iter_mut() {
        if project.progress >= project.required && !project.decoded {
            project.decoded = true;
            for mut drill in drills.iter_mut() {
                drill.efficiency = -1.0; // Sabotaged
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Emit a `MemeticVirusDecodedEvent` so multiple systems can subscribe and be sabotaged instead of tightly coupling to `MiningDrill`.
- Make the amount of research requested scale dynamically over time to trap the player deeper into the sunk cost.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Phantom signal project consumes `ResearchPoints`.
- [ ] Completing the phantom signal project applies a negative effect to infrastructure.

**7. Technical Guidance**
- Use the central event bus for the negative effects to maintain decoupled systems.

**8. Questions**
*Builder: add questions here if spec is unclear.*

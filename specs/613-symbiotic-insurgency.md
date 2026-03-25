# The Symbiotic Insurgency

**1. Overview**
Harvesting native "Xeno-Flora" yields high-value medicinal compounds but angers a hidden, planet-wide fungal intelligence. Instead of sending monsters, the intelligence infects the harvested products. Pops who consume the medicine are slowly "converted," becoming unwilling sleeper agents. When a critical mass is reached, they simultaneously sabotage key infrastructure to allow the jungle to reclaim the colony.

**2. Dependencies**
- `layer1::biology::Flora`
- `layer1::needs::Medical`
- `layer1::population::Pop`

**3. RED Phase: Tests First**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_medicine_infection() {
        let mut app = App::new();
        app.add_systems(Update, apply_medicine_system);

        let pop = app.world_mut().spawn((
            Pop,
            SymbiontInfection { level: 0.0 },
        )).id();

        app.world_mut().spawn((
            Medicine { infected: true },
            TargetPop(pop),
        ));

        app.update();

        let infection = app.world().get::<SymbiontInfection>(pop).unwrap();
        assert!(infection.level > 0.0, "Consuming infected medicine should increase the infection level");
    }

    #[test]
    fn test_critical_mass_sabotage() {
        let mut app = App::new();
        app.add_systems(Update, sleeper_agent_sabotage_system);

        app.insert_resource(GlobalInfectionState { total_infected: 100, critical_mass: 50 });

        let infrastructure = app.world_mut().spawn((
            Infrastructure { functional: true },
        )).id();

        app.update();

        let building = app.world().get::<Infrastructure>(infrastructure).unwrap();
        assert!(!building.functional, "When global infection exceeds critical mass, infrastructure should be sabotaged");
    }
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct SymbiontInfection {
    pub level: f32,
}

#[derive(Component)]
pub struct Medicine {
    pub infected: bool,
}

#[derive(Component)]
pub struct TargetPop(pub Entity);

#[derive(Component)]
pub struct Infrastructure {
    pub functional: bool,
}

#[derive(Resource)]
pub struct GlobalInfectionState {
    pub total_infected: u32,
    pub critical_mass: u32,
}

pub fn apply_medicine_system(
    mut commands: Commands,
    mut pops: Query<&mut SymbiontInfection, With<Pop>>,
    medicines: Query<(Entity, &Medicine, &TargetPop)>,
) {
    for (med_entity, medicine, target) in medicines.iter() {
        if let Ok(mut infection) = pops.get_mut(target.0) {
            if medicine.infected {
                infection.level += 10.0;
            }
        }
        commands.entity(med_entity).despawn(); // Consume medicine
    }
}

pub fn sleeper_agent_sabotage_system(
    state: Option<Res<GlobalInfectionState>>,
    mut buildings: Query<&mut Infrastructure>,
) {
    if let Some(state) = state {
        if state.total_infected >= state.critical_mass {
            for mut building in buildings.iter_mut() {
                building.functional = false;
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Introduce a more nuanced global tracker that calculates total infection based on the active `SymbiontInfection` components on `Pop`s, instead of relying solely on an arbitrary manual Resource.
- Allow infrastructure sabotage to target specific critical systems (like airlocks or vents) rather than globally disabling everything at once.

**6. Acceptance Criteria (Testable!)**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops gain `SymbiontInfection` when consuming infected `Medicine`.
- [ ] `Infrastructure` functional state becomes false when the global infection hits critical mass.

**7. Technical Guidance**
- The `Medicine` entity acts as an event payload here. In real implementation, this might be handled via a Bevy Event or an Item Use action in the ECS.

**8. Questions**
*Builder: add questions here if spec is unclear.*

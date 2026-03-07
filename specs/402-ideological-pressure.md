# 402: Ideological Pressure

## 1. Overview
Civilizations on the Galactic map (Layer 3) emit "Cultural Pressure" based on their output of luxury goods, art, and overall prosperity.

If your colony borders a civilization with significantly higher Cultural Pressure and a different Civic Ideology, your Pops slowly start adopting the rival's ethics. If the ideological drift becomes too strong, the colony suffers unrest or may even attempt to secede to join the rival faction.

## 2. Dependencies
- `197-civic-ideology.md` (For base colony ideologies/ethics)
- `094-system-view.md` (For spatial relationships to other empires)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::ideology::{ColonyIdeology, PopEthics, EthicType};

    #[test]
    fn test_ideological_pressure_drifts_pop_ethics() {
        let mut app = App::new();
        app.add_systems(Update, apply_ideological_pressure_system);

        // Player colony is Survivalist
        app.insert_resource(ColonyIdeology { current: EthicType::Survivalist, strength: 10.0 });

        // Rival neighbor is Pacifist with high pressure
        app.insert_resource(NeighborPressure { ethic: EthicType::Pacifist, strength: 50.0 });

        let pop = app.world_mut().spawn((
            Pop,
            PopEthics { current: EthicType::Survivalist, adherence: 100.0 },
        )).id();

        app.update();

        // Pop adherence should decrease, or shift towards Pacifist
        let ethics = app.world().get::<PopEthics>(pop).unwrap();
        assert!(ethics.adherence < 100.0, "Pop adherence to state ideology should weaken under pressure");
    }

    #[test]
    fn test_high_drift_causes_unrest() {
        let mut app = App::new();
        app.add_systems(Update, calculate_ideological_dissent_system);

        app.insert_resource(ColonyIdeology { current: EthicType::Survivalist, strength: 10.0 });

        let pop = app.world_mut().spawn((
            Pop,
            PopEthics { current: EthicType::Pacifist, adherence: 80.0 }, // Opposed to state
            Mood { value: 50.0, ..Default::default() },
        )).id();

        app.update();

        let mood = app.world().get::<Mood>(pop).unwrap();
        assert!(mood.value < 50.0, "Pop with opposed ideology should suffer mood penalties");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, Mood};
use crate::layer1::ideology::{ColonyIdeology, PopEthics, EthicType};

#[derive(Resource)]
pub struct NeighborPressure {
    pub ethic: EthicType,
    pub strength: f32, // 0.0 to 100.0
}

pub fn apply_ideological_pressure_system(
    colony_ideology: Res<ColonyIdeology>,
    neighbor: Res<NeighborPressure>,
    mut pop_query: Query<&mut PopEthics, With<Pop>>,
) {
    if neighbor.strength > colony_ideology.strength {
        let drift_rate = (neighbor.strength - colony_ideology.strength) * 0.01;

        for mut ethics in pop_query.iter_mut() {
            if ethics.current == colony_ideology.current {
                ethics.adherence -= drift_rate;

                if ethics.adherence <= 0.0 {
                    ethics.current = neighbor.ethic;
                    ethics.adherence = 10.0; // Start adopting the new one
                }
            } else if ethics.current == neighbor.ethic {
                ethics.adherence = (ethics.adherence + drift_rate).min(100.0);
            }
        }
    }
}

pub fn calculate_ideological_dissent_system(
    colony_ideology: Res<ColonyIdeology>,
    mut pop_query: Query<(&PopEthics, &mut Mood), With<Pop>>,
) {
    for (ethics, mut mood) in pop_query.iter_mut() {
        if ethics.current != colony_ideology.current {
            // Apply a penalty based on how strongly they believe in the rival ideology
            let penalty = ethics.adherence * 0.05;
            mood.value = (mood.value - penalty).max(0.0);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Mitigation:** Allow players to build "Propaganda Broadcasters" or "Censorship Firewalls" to artificially boost their `ColonyIdeology` strength or block `NeighborPressure`.
- **Trade routes:** Pressure should travel faster along active trade routes or hyperlanes, making isolationism a valid defense against cultural victory.
- **Ethics Enum:** Ensure `EthicType` is defined clearly (e.g. Pacifist vs Militarist, Materialist vs Spiritualist).

## 6. Acceptance Criteria (Testable!)
- [ ] `NeighborPressure` reduces `PopEthics.adherence` if neighbor strength > local strength.
- [ ] Pops convert to the neighbor's ethic if adherence hits 0.
- [ ] Pops with ethics differing from the `ColonyIdeology` suffer mood penalties.
- [ ] Tests pass.

## 7. Technical Guidance
- `NeighborPressure` should ideally be calculated dynamically by a Layer 3 system rather than being a static resource, but a resource is fine for MVP testing.

## 8. Questions
*Builder: add questions here if spec is unclear.*

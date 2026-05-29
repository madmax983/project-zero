#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(Component)]
pub struct BaseSignature {
    pub value: u32,
}

#[derive(Component)]
pub struct SensorSignature {
    pub current: u32,
}

#[derive(Component)]
pub struct EnergyPool {
    pub current: u32,
}

#[derive(Component)]
pub struct DecoyBuoy {
    pub active: bool,
    pub signature_bonus: u32,
    pub energy_cost: u32, // Energy cost per tick or time unit
}

#[derive(Component)]
pub struct SignatureAmplifier {
    pub active: bool,
    pub multiplier: f32,
    pub energy_cost: u32, // Energy cost per tick or time unit
}

pub fn apply_signature_spoofing(
    mut query: Query<(
        &BaseSignature,
        &mut SensorSignature,
        Option<&DecoyBuoy>,
        Option<&SignatureAmplifier>,
    )>,
) {
    for (base, mut sig, decoy, amplifier) in query.iter_mut() {
        let mut new_sig = base.value as f32;

        if let Some(d) = decoy {
            if d.active {
                new_sig += d.signature_bonus as f32;
            }
        }

        if let Some(amp) = amplifier {
            if amp.active {
                new_sig *= amp.multiplier;
            }
        }

        sig.current = new_sig as u32;
    }
}

#[allow(clippy::type_complexity)]
pub fn consume_spoofing_energy(
    mut query: Query<
        (
            &mut EnergyPool,
            Option<&mut DecoyBuoy>,
            Option<&mut SignatureAmplifier>,
        ),
        Or<(With<DecoyBuoy>, With<SignatureAmplifier>)>,
    >,
) {
    for (mut energy, mut decoy, mut amplifier) in query.iter_mut() {
        let mut total_cost = 0;

        if let Some(ref d) = decoy {
            if d.active {
                total_cost += d.energy_cost;
            }
        }

        if let Some(ref amp) = amplifier {
            if amp.active {
                total_cost += amp.energy_cost;
            }
        }

        if total_cost == 0 {
            continue;
        }

        if energy.current >= total_cost {
            energy.current -= total_cost;
        } else {
            // Deactivate systems if out of energy
            if let Some(ref mut d) = decoy {
                d.active = false;
            }
            if let Some(ref mut amp) = amplifier {
                amp.active = false;
            }
            // energy.current = 0; // Or just drain what's left, but leaving it here matches the refactor phase goal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::ship::Ship;
    use crate::layer2::ship::ShipType;
    #[allow(unused_imports)]
    use bevy::prelude::*;

    #[test]
    fn test_decoy_increases_signature() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app
            .world_mut()
            .spawn((
                Ship::new(ShipType::Scout),
                BaseSignature { value: 10 },
                DecoyBuoy {
                    active: true,
                    signature_bonus: 50,
                    energy_cost: 0,
                },
                SensorSignature { current: 10 },
            ))
            .id();

        // Act
        // The system `apply_signature_spoofing` should update `SensorSignature`.
        app.add_systems(Update, apply_signature_spoofing);
        app.update();

        // Assert
        let sig = app.world().get::<SensorSignature>(entity).unwrap();
        assert_eq!(sig.current, 60);
    }

    #[test]
    fn test_decoy_consumes_energy() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app
            .world_mut()
            .spawn((
                Ship::new(ShipType::Scout),
                EnergyPool { current: 100 },
                DecoyBuoy {
                    active: true,
                    signature_bonus: 50,
                    energy_cost: 10,
                },
            ))
            .id();

        // Act
        // The system `consume_spoofing_energy` should deduct energy cost per tick/update.
        app.add_systems(Update, consume_spoofing_energy);
        app.update();

        // Assert
        let energy = app.world().get::<EnergyPool>(entity).unwrap();
        assert_eq!(energy.current, 90);
    }

    #[test]
    fn test_amplifier_scales_signature() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app
            .world_mut()
            .spawn((
                Ship::new(ShipType::Scout),
                BaseSignature { value: 20 },
                SignatureAmplifier {
                    active: true,
                    multiplier: 3.0,
                    energy_cost: 15,
                },
                SensorSignature { current: 20 },
            ))
            .id();

        // Act
        app.add_systems(Update, apply_signature_spoofing);
        app.update();

        // Assert
        let sig = app.world().get::<SensorSignature>(entity).unwrap();
        assert_eq!(sig.current, 60);
    }

    #[test]
    fn test_energy_depletion_deactivates_systems() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app
            .world_mut()
            .spawn((
                Ship::new(ShipType::Scout),
                EnergyPool { current: 5 },
                DecoyBuoy {
                    active: true,
                    signature_bonus: 50,
                    energy_cost: 10,
                },
                SignatureAmplifier {
                    active: true,
                    multiplier: 3.0,
                    energy_cost: 15,
                },
            ))
            .id();

        // Act
        app.add_systems(Update, consume_spoofing_energy);
        app.update();

        // Assert
        let decoy = app.world().get::<DecoyBuoy>(entity).unwrap();
        let amp = app.world().get::<SignatureAmplifier>(entity).unwrap();

        assert!(!decoy.active);
        assert!(!amp.active);
    }
}

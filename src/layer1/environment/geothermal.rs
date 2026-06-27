use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// A tile marking a Geothermal Vent.
#[derive(Component)]
pub struct GeothermalVent {
    pub high_power_duration: u32,
    pub low_power_duration: u32,
    pub current_tick: u32,
    pub is_high_power: bool,
    pub high_power_output: f32,
    pub low_power_output: f32,
}

impl Default for GeothermalVent {
    fn default() -> Self {
        Self {
            high_power_duration: 300,
            low_power_duration: 200,
            current_tick: 0,
            is_high_power: true,
            high_power_output: 1000.0,
            low_power_output: 100.0,
        }
    }
}

/// Global state tracking whether geothermal vents are pulsing.
#[derive(Resource, Default)]
pub struct GeothermalPulseState {
    pub is_pulsing: bool,
}

/// Event to manually or systemically trigger/stop a pulse.
#[derive(Event)]
pub struct GeothermalPulseEvent {
    pub is_active: bool,
}

/// Component to track original power output before it was boosted by a pulse.
#[derive(Component)]
pub struct GeothermalBoosted {
    pub original_output: f32,
}

/// System to process GeothermalPulseEvent and update GeothermalPulseState.
pub fn geothermal_pulse_system(
    mut events: EventReader<GeothermalPulseEvent>,
    mut state: ResMut<GeothermalPulseState>,
) {
    for event in events.read() {
        state.is_pulsing = event.is_active;
    }
}

#[derive(Resource)]
pub struct LavaSurgeChance(pub f32);

impl Default for LavaSurgeChance {
    fn default() -> Self {
        Self(0.01)
    }
}

pub fn process_geothermal_vents(
    mut query: Query<&mut GeothermalVent>,
    mut pulse_events: EventWriter<GeothermalPulseEvent>,
) {
    for mut vent in query.iter_mut() {
        vent.current_tick += 1;

        let phase_duration = if vent.is_high_power { vent.high_power_duration } else { vent.low_power_duration };

        if vent.current_tick >= phase_duration {
            vent.current_tick = 0;
            vent.is_high_power = !vent.is_high_power;
            pulse_events.send(GeothermalPulseEvent {
                is_active: vent.is_high_power,
            });
        }
    }
}

pub fn process_lava_surges(
    vent_query: Query<(&GeothermalVent, &GridPosition)>,
    mut explosion_events: EventWriter<crate::layer1::environment::volatile::ExplosionEvent>,
    surge_chance: Res<LavaSurgeChance>,
) {
    let mut rng = rand::thread_rng();
    for (vent, vent_pos) in vent_query.iter() {
        if vent.is_high_power && rand::Rng::gen::<f32>(&mut rng) < surge_chance.0 {
            explosion_events.send(crate::layer1::environment::volatile::ExplosionEvent {
                center: *vent_pos,
                damage: 25.0,
                radius: 1,
            });
        }
    }
}

/// System to apply or remove the Geothermal Pulse boost to active buildings on vents.
pub fn geothermal_boost_system(
    mut commands: Commands,
    state: Res<GeothermalPulseState>,
    mut active_buildings: Query<(
        Entity,
        &mut PowerSource,
        &GridPosition,
        Option<&GeothermalBoosted>,
    )>,
    vents: Query<&GridPosition, With<GeothermalVent>>,
) {
    // ⚡ Bolt Optimization: Use bevy::utils::HashSet (AHash) instead of std::collections::HashSet to eliminate SipHash overhead on integer keys.
    let vent_positions: bevy::utils::HashSet<_> = vents.iter().map(|p| (p.x, p.y)).collect();

    for (entity, mut power_source, pos, boosted) in active_buildings.iter_mut() {
        let is_on_vent = vent_positions.contains(&(pos.x, pos.y));
        let should_be_boosted = state.is_pulsing && is_on_vent && power_source.active;

        if should_be_boosted && boosted.is_none() {
            // Apply boost
            commands.entity(entity).insert(GeothermalBoosted {
                original_output: power_source.output,
            });
            power_source.output *= 2.0; // E.g., double the output
        } else if !should_be_boosted && boosted.is_some() {
            // Remove boost
            if let Some(boost_data) = boosted {
                power_source.output = boost_data.original_output;
            }
            commands.entity(entity).remove::<GeothermalBoosted>();
        }
    }
}

/// System to apply decay to buildings during a pulse and handle explosions if health reaches zero.
pub fn geothermal_decay_system(
    mut commands: Commands,
    state: Res<GeothermalPulseState>,
    mut buildings: Query<(Entity, &mut Structure, &GridPosition, Option<&PowerSource>)>,
    vents: Query<&GridPosition, With<GeothermalVent>>,
    mut explosion_events: EventWriter<crate::layer1::environment::volatile::ExplosionEvent>,
) {
    if !state.is_pulsing {
        return;
    }

    // ⚡ Bolt Optimization: Use bevy::utils::HashSet (AHash) instead of std::collections::HashSet to eliminate SipHash overhead on integer keys.
    let vent_positions: bevy::utils::HashSet<_> = vents.iter().map(|p| (p.x, p.y)).collect();

    for (entity, mut structure, pos, power_source) in buildings.iter_mut() {
        if vent_positions.contains(&(pos.x, pos.y)) {
            // Only decay if active
            let is_active = power_source.is_none_or(|p| p.active);
            if is_active {
                structure.current_hp -= 5.0; // Magic number decay rate
                if structure.current_hp <= 0.0 {
                    explosion_events.send(crate::layer1::environment::volatile::ExplosionEvent {
                        center: *pos,
                        damage: 50.0, // Magic collateral damage
                        radius: 1,    // Hits adjacent tiles
                    });
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;
    use bevy::prelude::*;

    #[test]
    fn test_geothermal_vent_pulse_cycle() {
        let mut app = App::new();
        app.init_resource::<GeothermalPulseState>();
        app.add_event::<GeothermalPulseEvent>();
        app.add_systems(Update, geothermal_pulse_system);

        // Initial state: Dormant
        let state = app.world().resource::<GeothermalPulseState>();
        assert!(!state.is_pulsing);

        // Advance time or trigger event to start pulse
        app.world_mut()
            .send_event(GeothermalPulseEvent { is_active: true });
        app.update();

        let state = app.world().resource::<GeothermalPulseState>();
        assert!(state.is_pulsing);
    }

    #[test]
    fn test_building_on_vent_receives_boost_and_decay_during_pulse() {
        let mut app = App::new();
        app.init_resource::<GeothermalPulseState>();
        app.add_event::<crate::layer1::environment::volatile::ExplosionEvent>();
        app.world_mut()
            .resource_mut::<GeothermalPulseState>()
            .is_pulsing = true;

        app.add_systems(Update, (geothermal_boost_system, geothermal_decay_system));

        let vent_pos = GridPosition { x: 5, y: 5 };

        // Spawn a vent
        app.world_mut().spawn((GeothermalVent::default(), vent_pos));

        // Spawn an active power source building on the vent
        let building_entity = app
            .world_mut()
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                vent_pos,
            ))
            .id();

        app.update();

        let power = app.world().get::<PowerSource>(building_entity).unwrap();
        let structure = app.world().get::<Structure>(building_entity).unwrap();

        // Verify boost
        assert!(
            power.output > 10.0,
            "Building should receive power output boost"
        );
        // Verify decay
        assert!(
            structure.current_hp < 100.0,
            "Building should suffer rapid decay during pulse"
        );
    }

    #[test]
    fn test_inactive_building_on_vent_ignores_pulse() {
        let mut app = App::new();
        app.init_resource::<GeothermalPulseState>();
        app.add_event::<crate::layer1::environment::volatile::ExplosionEvent>();
        app.world_mut()
            .resource_mut::<GeothermalPulseState>()
            .is_pulsing = true;

        app.add_systems(Update, (geothermal_boost_system, geothermal_decay_system));

        let vent_pos = GridPosition { x: 5, y: 5 };

        app.world_mut().spawn((GeothermalVent::default(), vent_pos));

        let building_entity = app
            .world_mut()
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: false,
                }, // Inactive!
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                vent_pos,
            ))
            .id();

        app.update();

        let power = app.world().get::<PowerSource>(building_entity).unwrap();
        let structure = app.world().get::<Structure>(building_entity).unwrap();

        // Should not be boosted
        assert_eq!(
            power.output, 10.0,
            "Inactive building should not receive power output boost"
        );
        // Should not decay
        assert_eq!(
            structure.current_hp, 100.0,
            "Inactive building should not suffer decay during pulse"
        );
    }

    #[test]
    fn test_building_explodes_when_decay_reaches_zero() {
        let mut app = App::new();
        app.init_resource::<GeothermalPulseState>();
        app.add_event::<crate::layer1::environment::volatile::ExplosionEvent>();
        app.world_mut()
            .resource_mut::<GeothermalPulseState>()
            .is_pulsing = true;

        app.add_systems(Update, geothermal_decay_system);

        let vent_pos = GridPosition { x: 5, y: 5 };
        let adj_pos = GridPosition { x: 5, y: 6 };

        app.world_mut().spawn((GeothermalVent::default(), vent_pos));

        // Building about to explode
        let building_entity = app
            .world_mut()
            .spawn((
                PowerSource {
                    output: 10.0,
                    active: true,
                },
                Structure {
                    current_hp: 1.0,
                    max_hp: 100.0,
                }, // 1 health, will drop below 0
                vent_pos,
            ))
            .id();

        // Adjacent building to take collateral damage
        let _adj_building = app
            .world_mut()
            .spawn((
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                adj_pos,
            ))
            .id();

        app.update();

        // The vent building should be destroyed (or marked for destruction)
        assert!(
            app.world().get::<Structure>(building_entity).is_none(),
            "Building should be destroyed"
        );

        // Instead of testing damage here directly (that's handled by handle_explosion_system now),
        // we test that the ExplosionEvent was fired correctly.
        let events = app
            .world()
            .resource::<Events<crate::layer1::environment::volatile::ExplosionEvent>>();
        let mut reader = events.get_cursor();
        let event = reader.read(events).next();
        assert!(
            event.is_some(),
            "An ExplosionEvent should have been emitted"
        );
        let e = event.unwrap();
        assert_eq!(e.center, vent_pos);
        assert_eq!(e.damage, 50.0);
        assert_eq!(e.radius, 1);
    }

    #[test]
    fn test_geothermal_heartbeat_cycle() {
        let mut app = App::new();
        app.add_event::<GeothermalPulseEvent>();
        app.add_systems(Update, process_geothermal_vents);

        let vent = app.world_mut().spawn((
            GeothermalVent {
                high_power_duration: 300,
                low_power_duration: 200,
                current_tick: 0,
                is_high_power: true,
                high_power_output: 1000.0,
                low_power_output: 100.0,
            },
        )).id();

        app.update();
        let vent_state = app.world().get::<GeothermalVent>(vent).unwrap();
        assert_eq!(vent_state.is_high_power, true);

        let mut vent_mut = app.world_mut().get_mut::<GeothermalVent>(vent).unwrap();
        vent_mut.current_tick = 300;

        app.update();
        let vent_state_low = app.world().get::<GeothermalVent>(vent).unwrap();
        assert_eq!(vent_state_low.is_high_power, false);
    }

    #[test]
    fn test_lava_surge_damage() {
        let mut app = App::new();
        app.add_event::<crate::layer1::environment::volatile::ExplosionEvent>();
        app.add_systems(Update, process_lava_surges);

        app.world_mut().spawn((
            GeothermalVent {
                high_power_duration: 300,
                low_power_duration: 200,
                current_tick: 150,
                is_high_power: true,
                high_power_output: 1000.0,
                low_power_output: 100.0,
            },
            GridPosition { x: 10, y: 10 },
        ));

        app.world_mut().insert_resource(LavaSurgeChance(1.0));
        app.update();

        let events = app.world().resource::<Events<crate::layer1::environment::volatile::ExplosionEvent>>();
        let mut reader = events.get_cursor();
        let event = reader.read(events).next();
        assert!(event.is_some(), "An ExplosionEvent should have been emitted for lava surge");
    }
}

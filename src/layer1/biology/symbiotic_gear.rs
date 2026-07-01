use bevy_ecs::prelude::*;
use crate::layer1::psychology::needs::{Needs, HUNGER_DECAY_PER_TICK};
use crate::layer1::economy::items::Equipment;
use crate::layer1::shields::DamageEvent;

const SYMBIOTE_HUNGER_MULTIPLIER: f32 = 2.0;

pub fn symbiotic_hunger_modifier_system(
    mut query: Query<(&mut Needs, &Equipment)>,
) {
    for (mut needs, equip) in query.iter_mut() {
        if equip.has_symbiote {
            // Apply the additional decay (since decay_needs_system applies base decay)
            // It doubles the decay rate by subtracting an extra base decay amount
            needs.hunger -= HUNGER_DECAY_PER_TICK * (SYMBIOTE_HUNGER_MULTIPLIER - 1.0);
            needs.hunger = needs.hunger.max(0.0);
        }
    }
}

pub fn starving_symbiote_damage_system(
    query: Query<(Entity, &Needs, &Equipment)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (entity, needs, equip) in query.iter() {
        if equip.has_symbiote && needs.hunger <= f32::EPSILON {
            damage_events.send(DamageEvent {
                target: entity,
                amount: 5.0, // Significant damage per tick when starving
                velocity: 0.0, // Symbiote damage is internal
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::health::Health;
    use crate::layer1::shields::DamageEvent;
    use crate::layer1::economy::items::Equipment;

    #[test]
    fn test_symbiotic_gear_increases_hunger_decay() {
        let mut app = App::new();
        // Just run symbiotic_hunger_modifier_system to test the extra decay
        app.add_systems(Update, symbiotic_hunger_modifier_system);

        let pop = app.world_mut().spawn((
            Needs { hunger: 1.0, ..Default::default() },
            Equipment { has_symbiote: true, ..Default::default() },
        )).id();

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        // Since we didn't run decay_needs_system, it should be 1.0 - HUNGER_DECAY_PER_TICK
        assert!(needs.hunger < 1.0, "Symbiotic gear should increase the wearer's hunger decay rate.");
    }

    #[test]
    #[allow(deprecated)]
    fn test_starving_symbiote_damages_wearer() {
        let mut app = App::new();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, starving_symbiote_damage_system);

        let pop = app.world_mut().spawn((
            Health { current: 100.0, max: 100.0, has_rust_lung: false },
            Needs { hunger: 0.0, ..Default::default() }, // Pop is starving
            Equipment { has_symbiote: true, ..Default::default() },
        )).id();

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut cursor = damage_events.get_cursor();
        let mut found = false;
        for event in cursor.read(damage_events) {
            if event.target == pop {
                found = true;
            }
        }

        assert!(found, "A starving symbiote should damage its wearer.");
    }
}

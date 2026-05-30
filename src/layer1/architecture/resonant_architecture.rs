use crate::layer1::entities::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Room {
    pub material: BuildingMaterial,
}

#[derive(PartialEq, Clone, Debug, Default)]
pub enum BuildingMaterial {
    #[default]
    Standard,
    MindStone,
    IronPlating,
}

#[derive(Component)]
pub struct RoomBoundary {
    pub radius: u32,
}

#[derive(Component)]
pub struct AssignedRoom(pub Entity);

#[derive(Component, Default)]
pub struct ResonanceBuff {
    pub material: BuildingMaterial,
    pub ticks_remaining: u32,
}

#[derive(Component)]
pub struct PopResonanceTraits {
    pub research_speed_mult: f32,
    pub stress_gain_mult: f32,
    pub aggression_mult: f32,
}

impl Default for PopResonanceTraits {
    fn default() -> Self {
        Self {
            research_speed_mult: 1.0,
            stress_gain_mult: 1.0,
            aggression_mult: 1.0,
        }
    }
}

type PopResonanceQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PopResonanceTraits,
        Option<&'static AssignedRoom>,
        Option<&'static mut ResonanceBuff>,
        Entity,
    ),
    With<Pop>,
>;

#[allow(clippy::type_complexity)]
pub fn apply_resonant_architecture_system(
    room_query: Query<&Room>,
    mut pop_query: PopResonanceQuery,
    mut commands: Commands,
) {
    for (mut traits, assigned, mut buff, entity) in pop_query.iter_mut() {
        // Handle acquiring buff from assigned room
        if let Some(assigned_room) = assigned {
            if let Ok(room) = room_query.get(assigned_room.0) {
                if room.material != BuildingMaterial::Standard {
                    if let Some(ref mut b) = buff {
                        b.material = room.material.clone();
                        b.ticks_remaining = 100; // 100 ticks of resonance
                    } else {
                        commands.entity(entity).insert(ResonanceBuff {
                            material: room.material.clone(),
                            ticks_remaining: 100,
                        });
                        continue;
                    }
                }
            }
        }

        // Apply buff to traits
        traits.research_speed_mult = 1.0;
        traits.stress_gain_mult = 1.0;
        traits.aggression_mult = 1.0;

        if let Some(ref mut b) = buff {
            if b.ticks_remaining > 0 {
                match b.material {
                    BuildingMaterial::MindStone => {
                        traits.research_speed_mult *= 2.0;
                        traits.stress_gain_mult *= 2.0;
                    }
                    BuildingMaterial::IronPlating => {
                        traits.aggression_mult *= 1.5;
                    }
                    BuildingMaterial::Standard => {}
                }
                b.ticks_remaining -= 1;
            }

            if b.ticks_remaining == 0 {
                commands.entity(entity).remove::<ResonanceBuff>();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_resonant_material_amplifies_trait() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let room_ent = app
            .world_mut()
            .spawn((Room {
                material: BuildingMaterial::MindStone,
            },))
            .id();

        let pop = app
            .world_mut()
            .spawn((Pop, AssignedRoom(room_ent), PopResonanceTraits::default()))
            .id();

        // Run once to add buff
        app.update();
        // Run twice to apply buff
        app.update();

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.research_speed_mult, 2.0);
        assert_eq!(traits.stress_gain_mult, 2.0);
    }

    #[test]
    fn test_pop_outside_room_not_affected() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let _room = app
            .world_mut()
            .spawn((Room {
                material: BuildingMaterial::MindStone,
            },))
            .id();

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                PopResonanceTraits::default(), // No AssignedRoom
            ))
            .id();

        app.update();

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.research_speed_mult, 1.0);
        assert_eq!(traits.stress_gain_mult, 1.0);
    }

    #[test]
    fn test_buff_decays_after_leaving_room() {
        let mut app = App::new();
        app.add_systems(Update, apply_resonant_architecture_system);

        let room_ent = app
            .world_mut()
            .spawn((Room {
                material: BuildingMaterial::IronPlating,
            },))
            .id();

        let pop = app
            .world_mut()
            .spawn((Pop, AssignedRoom(room_ent), PopResonanceTraits::default()))
            .id();

        // Run once to add buff
        app.update();
        // Run twice to apply buff
        app.update();

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.aggression_mult, 1.5);

        // Remove assigned room (Pop leaves the room)
        app.world_mut().entity_mut(pop).remove::<AssignedRoom>();

        for _ in 0..100 {
            app.update();
        }

        let traits = app.world().get::<PopResonanceTraits>(pop).unwrap();
        assert_eq!(traits.aggression_mult, 1.0, "Buff should have decayed");
    }
}

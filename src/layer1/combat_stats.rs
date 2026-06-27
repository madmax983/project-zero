use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct CombatStats {
    pub melee_damage: f32,
    pub damage_types: Vec<DamageType>,
}

#[derive(Component, Default, Debug)]
pub struct Armor {
    pub rating: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DamageType {
    Piercing,
    Venom,
}

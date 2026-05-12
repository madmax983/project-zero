use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SystemQuarantine;

#[derive(Component)]
pub struct TradeHub {
    pub active: bool,
}

#[derive(Component)]
pub struct SystemLocation;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct UnrestLevel(pub u32);

#[derive(Component)]
pub struct ResourceStockpile {
    pub uncontaminated_soil: u32,
}

#[derive(Component)]
pub struct WarlordFaction;

pub fn apply_quarantine_effects(
    mut query: Query<&mut TradeHub, With<SystemQuarantine>>,
) {
    for mut hub in query.iter_mut() {
        hub.active = false;
    }
}

pub fn handle_quarantine_decay(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UnrestLevel), With<SystemQuarantine>>,
) {
    for (entity, mut unrest) in query.iter_mut() {
        unrest.0 += 10;
        if unrest.0 > 150 {
            commands.entity(entity).insert(WarlordFaction);
        }
    }
}

#[cfg(test)]
mod tests;

use super::*;
pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    // Check for Grave before validation
    let mut grave_entity = None;
    if let Some(map) = world.get_resource::<BuildingMap>() {
        if let Some(&entity) = map.0.get(&(x, y)) {
            if world.get::<crate::layer1::funeral::Grave>(entity).is_some() {
                grave_entity = Some(entity);
            }
        }
    }

    if let Err(e) = validate_building_placement(world, x, y) {
        let allow_override = e == PlacementError::Occupied && grave_entity.is_some();
        if !allow_override {
            handle_placement_error(world, e);
            return false;
        }
    }

    // Check Tech requirements
    if !check_tech_requirements(world, building_type) {
        return false;
    }

    // Get material
    let material = if building_type.supports_material() {
        world
            .get_resource::<BuildMode>()
            .map(|m| m.selected_material)
            .unwrap_or_default()
    } else {
        MaterialType::default()
    };

    // Check affordability and deduct cost
    if !deduct_building_cost(world, building_type, material) {
        return false;
    }

    // --- All validation passed, commit to placing the building ---

    // If we are overwriting a grave, handle the sacrilege and destruction now
    if let Some(ge) = grave_entity {
        world.send_event(crate::layer1::ancestral_graves::SacrilegeEvent {
            pos: GridPosition { x, y },
        });
        // Remove grave synchronously
        world.despawn(ge);
    }

    // Spawn building
    let entity = spawn_building(world, x, y, building_type, material);

    apply_post_placement_effects(world, entity, x, y, building_type);

    true
}

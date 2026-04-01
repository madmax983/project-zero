sed -i 's/fn find_path_internal(/fn find_path_internal(\n    world: \&World,\n    start: (i32, i32),\n    end: (i32, i32),\n    can_use_vents: bool,\n    credentials: Option<\&AccessCredentials>,\n    can_eat_rock: bool,\n) -> Option<Vec<(i32, i32)>> {/' src/layer1/pathfinding.rs

sed -i 's/find_path_internal(world, start, end, false, None)/find_path_internal(world, start, end, false, None, false)/g' src/layer1/pathfinding.rs
sed -i 's/find_path_internal(world, start, end, false, credentials.as_ref())/find_path_internal(world, start, end, false, credentials.as_ref(), false)/g' src/layer1/pathfinding.rs
sed -i 's/find_path_internal(world, start, end, can_use_vents, None)/find_path_internal(world, start, end, can_use_vents, None, false)/g' src/layer1/pathfinding.rs

# Also add Lithovore custom path finding signature
cat << 'INNER' >> src/layer1/pathfinding.rs

/// Finds a path for a specific entity type, considering unique capabilities like eating rock.
pub fn find_path_for_lithovore(
    world: &World,
    start: (i32, i32),
    end: (i32, i32),
    can_eat_rock: bool,
) -> Option<Vec<(i32, i32)>> {
    find_path_internal(world, start, end, false, None, can_eat_rock)
}
INNER

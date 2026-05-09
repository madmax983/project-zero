1. **Add `ZoneType::Redlined` to `ZoneType` enum**:
   - Update `src/layer1/administration/zone.rs` to include `Redlined` in the `ZoneType` enum.
   ```bash
   sed -i '/Jail zone (for housing Inmates)./i\    \/\/\/ Redlined zone (disconnected from power grid).\\n    Redlined,' src/layer1/administration/zone.rs
   ```

2. **Add `FactionId::Stateless` to `FactionId` enum**:
   - Update `src/layer1/social/factions.rs` to include `Stateless` in the `FactionId` enum.
   ```bash
   sed -i '/Faction for those with no specific skill focus/i\    \/\/\/ Squatter faction living in redlined zones.\\n    Stateless,' src/layer1/social/factions.rs
   ```
   - Update `Factions::initialize` in `src/layer1/social/factions.rs` to initialize `Stateless` faction data.
   ```bash
   sed -i '/FactionId::Unaligned,/!b;n;n;n;n;a\            self.map.insert(\n                FactionId::Stateless,\n                FactionData {\n                    name: "Stateless Squatters".into(),\n                    ..Default::default()\n                },\n            );' src/layer1/social/factions.rs
   ```

3. **Create `src/layer1/administration/redlining.rs` module**:
   - Create the file and implement the redlining logic and tests. I will omit `ColonySecurity` and use a flat probability, and document this assumption.
   ```bash
   cat << 'CODE_EOF' > src/layer1/administration/redlining.rs
   use bevy_ecs::prelude::*;
   use rand::Rng;
   use crate::layer1::map::GridPosition;
   use crate::layer1::zone::{ZoneGrid, ZoneType};
   use crate::layer1::factions::{FactionMember, FactionId};
   use crate::layer1::energy::{PowerConsumer, Conduit};
   use crate::layer1::actions::{AssignedTo, AssignmentType};
   use crate::layer1::building::{Building, BuildingType};

   #[derive(Event, Default)]
   pub struct RedlineZoneEvent {
       pub pos: GridPosition,
   }

   #[derive(Component, Default)]
   pub struct Stateless;

   pub fn execute_redlining_system(
       mut commands: Commands,
       mut events: EventReader<RedlineZoneEvent>,
       mut zone_grid: ResMut<ZoneGrid>,
       mut consumers: Query<(Entity, &GridPosition), Or<(With<PowerConsumer>, With<Conduit>)>>,
       mut pops: Query<(Entity, &AssignedTo, &mut FactionMember)>,
       buildings: Query<(&Building, &GridPosition)>,
   ) {
       for event in events.read() {
           let start_zone = zone_grid.get(event.pos.x, event.pos.y);
           if start_zone == ZoneType::None || start_zone == ZoneType::Redlined {
               continue;
           }

           let mut queue = vec![(event.pos.x, event.pos.y)];
           let mut visited = std::collections::HashSet::new();
           visited.insert((event.pos.x, event.pos.y));

           while let Some((x, y)) = queue.pop() {
               zone_grid.set(x, y, ZoneType::Redlined);

               let neighbors = [
                   (x + 1, y),
                   (x - 1, y),
                   (x, y + 1),
                   (x, y - 1),
               ];

               for (nx, ny) in neighbors {
                   if !visited.contains(&(nx, ny)) && zone_grid.get(nx, ny) == start_zone {
                       visited.insert((nx, ny));
                       queue.push((nx, ny));
                   }
               }
           }

           for (entity, pos) in &mut consumers {
               if visited.contains(&(pos.x, pos.y)) {
                   commands.entity(entity).remove::<PowerConsumer>();
                   commands.entity(entity).remove::<Conduit>();
               }
           }

           for (pop_entity, assigned, mut faction) in &mut pops {
               if assigned.assignment_type == AssignmentType::HousingResident {
                   if let Ok((building, b_pos)) = buildings.get(assigned.entity) {
                       if building.building_type == BuildingType::Housing && visited.contains(&(b_pos.x, b_pos.y)) {
                           faction.faction_id = Some(FactionId::Stateless);
                           commands.entity(pop_entity).insert(Stateless);
                       }
                   }
               }
           }
       }
   }

   pub fn stateless_expansion_system(
       mut commands: Commands,
       mut zone_grid: ResMut<ZoneGrid>,
       consumers: Query<(Entity, &GridPosition), Or<(With<PowerConsumer>, With<Conduit>)>>,
       mut pops: Query<(Entity, &AssignedTo, &mut FactionMember)>,
       buildings: Query<(&Building, &GridPosition)>,
   ) {
       let mut rng = rand::thread_rng();
       let spread_chance = 0.01; // Flat probability as ColonySecurity doesn't exist

       let mut new_redlined = Vec::new();

       for y in 0..zone_grid.height as i32 {
           for x in 0..zone_grid.width as i32 {
               if zone_grid.get(x, y) == ZoneType::Redlined {
                   let neighbors = [
                       (x + 1, y),
                       (x - 1, y),
                       (x, y + 1),
                       (x, y - 1),
                   ];

                   for (nx, ny) in neighbors {
                       let neighbor_zone = zone_grid.get(nx, ny);
                       if neighbor_zone != ZoneType::None && neighbor_zone != ZoneType::Redlined {
                           if rng.gen::<f32>() < spread_chance {
                               new_redlined.push((nx, ny));
                           }
                       }
                   }
               }
           }
       }

       for (x, y) in new_redlined {
           zone_grid.set(x, y, ZoneType::Redlined);

           for (entity, pos) in &consumers {
               if pos.x == x && pos.y == y {
                   commands.entity(entity).remove::<PowerConsumer>();
                   commands.entity(entity).remove::<Conduit>();
               }
           }

           for (pop_entity, assigned, mut faction) in &mut pops {
               if assigned.assignment_type == AssignmentType::HousingResident {
                   if let Ok((building, b_pos)) = buildings.get(assigned.entity) {
                       if building.building_type == BuildingType::Housing && b_pos.x == x && b_pos.y == y {
                           faction.faction_id = Some(FactionId::Stateless);
                           commands.entity(pop_entity).insert(Stateless);
                       }
                   }
               }
           }
       }
   }

   #[cfg(test)]
   mod tests {
       use bevy_ecs::prelude::*;
       use super::*;

       #[test]
       fn test_dezoning_removes_upkeep_and_creates_stateless_faction() {
           let mut world = World::new();

           let mut grid = ZoneGrid::new(10, 10);
           grid.set(5, 5, ZoneType::Bedroom);
           world.insert_resource(grid);
           world.insert_resource(Events::<RedlineZoneEvent>::default());

           let consumer_entity = world.spawn((
               GridPosition { x: 5, y: 5 },
               PowerConsumer { demand: 50.0, active: true },
           )).id();

           let building_entity = world.spawn((
               Building { building_type: BuildingType::Housing },
               GridPosition { x: 5, y: 5 },
           )).id();

           let pop_entity = world.spawn((
               AssignedTo { entity: building_entity, assignment_type: AssignmentType::HousingResident },
               FactionMember { faction_id: Some(FactionId::MinersGuild) },
           )).id();

           world.resource_mut::<Events<RedlineZoneEvent>>().send(RedlineZoneEvent { pos: GridPosition { x: 5, y: 5 } });

           let mut schedule = Schedule::default();
           schedule.add_systems(execute_redlining_system);
           schedule.run(&mut world);

           let grid = world.resource::<ZoneGrid>();
           assert_eq!(grid.get(5, 5), ZoneType::Redlined);

           assert!(world.get::<PowerConsumer>(consumer_entity).is_none());

           let faction = world.get::<FactionMember>(pop_entity).unwrap();
           assert_eq!(faction.faction_id, Some(FactionId::Stateless));
           assert!(world.get::<Stateless>(pop_entity).is_some());
       }

       #[test]
       fn test_stateless_faction_spreads_to_adjacent_zones() {
           let mut world = World::new();

           let mut grid = ZoneGrid::new(10, 10);
           grid.set(5, 5, ZoneType::Redlined);
           grid.set(5, 6, ZoneType::Bedroom);
           world.insert_resource(grid);

           let mut schedule = Schedule::default();
           // In testing spread, we might need multiple ticks due to low probability, but we can verify it compiles and runs.
           schedule.add_systems(stateless_expansion_system);
           for _ in 0..1000 {
               schedule.run(&mut world);
           }

           let grid = world.resource::<ZoneGrid>();
           // It's highly probable to spread after 1000 ticks.
           assert_eq!(grid.get(5, 6), ZoneType::Redlined);
       }
   }
   CODE_EOF
   ```

4. **Register module and systems**:
   - Update `src/layer1/administration/mod.rs` to include `redlining`.
   ```bash
   sed -i 's/pub mod zone;/pub mod zone;\n\/\/\/ Redlining mechanics.\npub mod redlining;/' src/layer1/administration/mod.rs
   sed -i 's/pub use zone::\*/pub use zone::\*;\npub use redlining::\*;/' src/layer1/administration/mod.rs
   ```
   - In `src/layer1/systems/execution.rs`, register the systems.
   ```bash
   sed -i '/crate::layer1::zone::apply_zone_designation_system,/a\            crate::layer1::redlining::execute_redlining_system,\n            crate::layer1::redlining::stateless_expansion_system,' src/layer1/systems/execution.rs
   ```
   - In `src/simulation.rs`, register the `RedlineZoneEvent`.
   ```bash
   sed -i '/world.init_resource::<Events<crate::layer1::environment::events::DebrisFallEvent>>();/a\        world.init_resource::<Events<crate::layer1::administration::redlining::RedlineZoneEvent>>();' src/simulation.rs
   ```

5. **Run tests**:
   - `cargo test --lib layer1`
   - `cargo clippy -- -D warnings`

6. **Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.**

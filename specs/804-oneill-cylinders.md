# Specification: O'Neill Cylinders

**1. Overview**
Players can construct massive "Artificial Worlds" known as O'Neill Cylinders in Layer 2 (System Map). These megastructures are highly expensive to build but offer fully customizable biomes and sizes, serving as mobile (though slow) colonies in the void. They are vulnerable to hull breaches from orbital debris or combat, which can cause rapid decompression and catastrophic climate failure.

**2. Dependencies**
- `001-architecture-setup`
- `094-system-view` (Layer 2)
- `152-orbital-stations` (Foundation for Layer 2 structures)
- `119-airlock-pressure` (Atmosphere systems for Layer 1, adaptable to Layer 2 structures)

**3. RED Phase: Tests First**
```rust
#[test]
fn test_oneill_cylinder_construction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(OrbitalStructuresPlugin);

    // Arrange: Create a system map and provide massive resources
    let mut resources = ColonyResources::default();
    resources.metal = 50000;
    resources.knowledge = 10000;
    app.world.insert_resource(resources);

    // Act: Issue command to build an O'Neill Cylinder
    let mut commands = app.world.commands();
    let builder_entity = commands.spawn_empty().id();
    commands.entity(builder_entity).insert(ConstructMegastructureCommand {
        structure_type: MegastructureType::ONeillCylinder,
        size_rating: 3,
        target_orbit: OrbitId(1),
    });
    app.update();

    // Assert: O'Neill Cylinder entity is spawned with initial components
    let mut query = app.world.query::<(&Megastructure, &ONeillCylinder, &Atmosphere)>();
    let cylinder_count = query.iter(&app.world).count();
    assert_eq!(cylinder_count, 1, "An O'Neill Cylinder should be constructed");
}

#[test]
fn test_cylinder_biome_customization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Spawn an existing cylinder
    let cylinder_entity = app.world.spawn((
        ONeillCylinder::default(),
        BiomeData { current: BiomeType::Barren },
    )).id();

    // Act: Set custom biome parameters
    app.world.resource_mut::<Events<CustomizeBiomeEvent>>().send(CustomizeBiomeEvent {
        cylinder: cylinder_entity,
        target_biome: BiomeType::TropicalParadise,
    });
    app.update();

    // Assert: Biome data updates
    let biome = app.world.get::<BiomeData>(cylinder_entity).unwrap();
    assert_eq!(biome.current, BiomeType::TropicalParadise, "Cylinder biome should be customized");
}

#[test]
fn test_cylinder_hull_breach_decompression() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Arrange: Cylinder with atmosphere and pressure
    let cylinder_entity = app.world.spawn((
        ONeillCylinder { integrity: 100 },
        Atmosphere { pressure: 1.0, temperature: 25.0 },
    )).id();

    // Act: Asteroid impact causes hull breach
    app.world.resource_mut::<Events<AsteroidImpactEvent>>().send(AsteroidImpactEvent {
        target: cylinder_entity,
        damage: 50,
        causes_breach: true,
    });
    app.update();

    // Systems process the breach
    app.update();

    // Assert: Pressure drops and temperature plummets (explosive decompression)
    let atmosphere = app.world.get::<Atmosphere>(cylinder_entity).unwrap();
    assert!(atmosphere.pressure < 0.2, "Pressure should drop dramatically after breach");
    assert!(atmosphere.temperature < -50.0, "Temperature should freeze instantly upon decompression");
}
```

**4. GREEN Phase: Minimal Implementation**
```rust
// In src/layer2/orbital_megastructures.rs

#[derive(Component)]
pub struct ONeillCylinder {
    pub integrity: u32,
}

impl Default for ONeillCylinder {
    fn default() -> Self {
        Self { integrity: 100 }
    }
}

pub struct ConstructMegastructureCommand {
    pub structure_type: MegastructureType,
    pub size_rating: u32,
    pub target_orbit: OrbitId,
}

pub fn handle_megastructure_construction(
    mut commands: Commands,
    mut resources: ResMut<ColonyResources>,
    query: Query<(Entity, &ConstructMegastructureCommand)>,
) {
    for (entity, cmd) in query.iter() {
        if resources.metal >= 50000 {
            resources.metal -= 50000;
            commands.spawn((
                Megastructure,
                ONeillCylinder::default(),
                Atmosphere { pressure: 1.0, temperature: 20.0 },
                BiomeData { current: BiomeType::Barren },
            ));
            commands.entity(entity).despawn();
        }
    }
}

pub fn process_hull_breaches(
    mut events: EventReader<AsteroidImpactEvent>,
    mut query: Query<(&mut ONeillCylinder, &mut Atmosphere)>,
) {
    for event in events.read() {
        if let Ok((mut cylinder, mut atmosphere)) = query.get_mut(event.target) {
            cylinder.integrity = cylinder.integrity.saturating_sub(event.damage);
            if event.causes_breach {
                atmosphere.pressure = 0.0;
                atmosphere.temperature = -100.0; // Flash freeze
            }
        }
    }
}
```

**5. REFACTOR Phase: Quality & Design**
- Move megastructure definitions to a dedicated `megastructures` module inside Layer 2.
- Integrate the hull breach logic with existing `Atmosphere` calculations from Layer 1, enabling a proper diffusion system rather than an instantaneous hardcoded drop.
- Combine biome customization with existing Terraforming structures if possible.
- Add UI notifications (Chronicle Events) for hull breaches and mass casualty events resulting from decompression.

**6. Acceptance Criteria**
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] O'Neill Cylinders can be built and have a customized biome
- [ ] Hull breaches cause rapid decompression and freezing

**7. Technical Guidance**
- Building an O'Neill Cylinder should be extremely expensive and require late-game technology.
- A cylinder is essentially a portable Layer 1 map within Layer 2. Consider how to handle the data structure (whether it spawns a real `TerrainGrid` or remains abstracted until zoomed in).
- Make sure to add `AsteroidImpactEvent` and register the decompression systems in the `Update` schedule.

**8. Questions**
*Builder: add questions here if spec is unclear.*

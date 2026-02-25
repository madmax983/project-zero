use super::setup_world;
use crate::layer1::cybernetics::{Augmentations, Prosthetic, ProstheticType};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::execution::general_work::{calculate_work_amount, work_execution_system};
use crate::layer1::items::{Equipment, Item, Tool, ToolType};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ForestryProgress, MiningProgress};
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[test]
fn test_work_execution_calls_mine_rock() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    world.entity_mut(designation).insert(MiningProgress {
        current: 0.0,
        max: 1000000.0,
    });

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            Needs::default(),
        ))
        .id();

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation);
    assert!(progress.is_some());
    assert!(progress.unwrap().current > 0.0);

    assert!(world.get_entity(pop).is_ok());
}

#[test]
fn test_work_execution_calls_chop_tree() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Tree;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Chop,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let progress = world.get::<ForestryProgress>(designation);
    assert!(progress.is_some());
    assert!(progress.unwrap().current > 0.0);
}

#[test]
fn test_work_execution_completes_mining() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
            MiningProgress {
                current: 95.0,
                max: 100.0,
            },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    assert!(world.get_entity(designation).is_err());

    let terrain = world.resource::<TerrainGrid>();
    assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

    let items: Vec<_> = world
        .query::<&crate::layer1::resources::ResourceItem>()
        .iter(&world)
        .collect();
    assert!(!items.is_empty());
    assert_eq!(
        items[0].resource_type,
        crate::layer1::resources::ResourceType::Stone
    );
}

#[test]
fn test_work_execution_resets_pop_on_completion() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
            MiningProgress {
                current: 95.0,
                max: 100.0,
            },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            PopAction {
                current: ActionType::Work,
                current_utility: 0.8,
                ticks_committed: 5,
            },
        ))
        .id();

    work_execution_system(&mut world);

    assert!(world.get_entity(designation).is_err());

    assert!(world.get::<MovementTarget>(pop).is_none());
    assert!(world.get::<AtTarget>(pop).is_none());

    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(action.current, ActionType::Idle);
    assert!((action.current_utility - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_work_execution_cleans_up_stale_target() {
    let mut world = setup_world();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(9999),
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            PopAction {
                current: ActionType::Work,
                current_utility: 0.8,
                ticks_committed: 5,
            },
        ))
        .id();

    work_execution_system(&mut world);

    assert!(world.get::<MovementTarget>(pop).is_none());
    assert!(world.get::<AtTarget>(pop).is_none());

    let action = world.get::<PopAction>(pop).unwrap();
    assert_eq!(action.current, ActionType::Idle);
}

#[test]
fn test_work_execution_efficiency_low_morale() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Needs {
                hunger: 0.1,
                rest: 0.1,
                leisure: 0.1,
                hygiene: 0.8,
            },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation).unwrap();
    let is_crit_range = progress.current >= 22.5 && progress.current <= 27.5;
    let is_normal_range = progress.current >= 4.5 && progress.current <= 5.5;

    assert!(
        is_normal_range || is_crit_range,
        "Expected ~5.0 (or ~25.0 crit) progress, got {}",
        progress.current
    );
}

#[test]
fn test_work_execution_efficiency_high_morale() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Needs {
                hunger: 1.0,
                rest: 1.0,
                leisure: 1.0,
                hygiene: 0.8,
            },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation).unwrap();
    let is_normal = progress.current >= 10.8 && progress.current <= 13.2;
    let is_crit = progress.current >= 54.0 && progress.current <= 70.0;
    assert!(
        is_normal || is_crit,
        "Expected ~12.0 progress, got {}",
        progress.current
    );
}

#[test]
fn test_work_execution_skills_mining_efficiency() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let mut skills = Skills::default();
    skills.add_xp(SkillType::Mining, 100.0);

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let _pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            skills,
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation).unwrap();
    let is_normal = progress.current >= 9.9 && progress.current <= 12.1;
    let is_crit = progress.current >= 54.0 && progress.current <= 70.0;
    assert!(
        is_normal || is_crit,
        "Expected ~11.0 (or ~55.0 crit) progress, got {}",
        progress.current
    );
}

#[test]
fn test_work_execution_gains_xp() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Skills::default(),
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
        ))
        .id();

    work_execution_system(&mut world);

    let skills = world.get::<Skills>(pop).unwrap();
    assert!((skills.get_xp(SkillType::Mining) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_work_execution_augmentation_bonus() {
    let mut world = setup_world();
    {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Rock;
    }

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Mine,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    let tool = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let prosthetic = world
        .spawn(Prosthetic {
            prosthetic_type: ProstheticType::BionicArm,
            efficiency_bonus: 0.5,
            social_penalty: 0.0,
            power_consumption: 0.0,
        })
        .id();

    world.spawn((
        Pop,
        GridPosition { x: 5, y: 5 },
        Equipment {
            tool: Some(tool),
            ..Default::default()
        },
        MovementTarget {
            target_entity: designation,
            target_position: GridPosition { x: 5, y: 5 },
            for_action: ActionType::Work,
        },
        AtTarget,
        Augmentations {
            installed: vec![prosthetic],
        },
    ));

    work_execution_system(&mut world);

    let progress = world.get::<MiningProgress>(designation).unwrap();
    let is_normal = progress.current >= 13.5 && progress.current <= 16.5;
    let is_crit = progress.current >= 67.5 && progress.current <= 82.5;

    assert!(
        is_normal || is_crit,
        "Expected ~15.0 (or crit), got {}",
        progress.current
    );
}

#[test]
fn test_calculate_work_amount_cap() {
    let mut world = setup_world();
    world.insert_resource(crate::layer1::admin::AdminStats {
        efficiency: 1.0,
        ..Default::default()
    });

    let pop = world.spawn(Pop).id();
    let tool_entity = world
        .spawn((
            Item::default(),
            Tool {
                tool_type: ToolType::Pickaxe,
                durability: 100.0,
                max_durability: 100.0,
            },
        ))
        .id();

    let prosthetic = world
        .spawn(Prosthetic {
            prosthetic_type: ProstheticType::BionicArm,
            efficiency_bonus: 10000.0,
            social_penalty: 0.0,
            power_consumption: 0.0,
        })
        .id();
    world.entity_mut(pop).insert(Augmentations {
        installed: vec![prosthetic],
    });

    let work_amount = calculate_work_amount(
        &world,
        pop,
        DesignationType::Mine,
        Some(tool_entity),
        1.0,
        1.0,
        0.5,
    );

    assert!(
        (work_amount - 1000.0).abs() < 0.001,
        "Work amount should be capped at 1000.0, got {}",
        work_amount
    );
}

# 1365 Title Inflation

## 1. Overview
Pops demand career progression, leading to a proliferation of meaningless management titles. A Pop with a fancy title ("Senior Executive Miner") gains a Mood boost but costs Administration points to maintain, without any actual authority or output increase. If left unchecked, the colony becomes filled with "Vice Presidents of Hauling" who refuse to do menial labor.

## 2. Dependencies
- `Pop` component
- `Mood` component
- `Administration` resource (or equivalent points/currency)
- `Job` assignment system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_grant_title_increases_mood_and_costs_admin() {
    let mut app = App::new();
    // Arrange: Setup basic app with a Pop and Administration resource
    app.insert_resource(Administration::new(10));
    let pop_id = app.world_mut().spawn((Pop, Mood::new(50))).id();

    // Act: Grant a fancy title
    app.world_mut().resource_mut::<Events<GrantTitleEvent>>().send(GrantTitleEvent {
        target: pop_id,
        title: "Senior Executive Miner".to_string(),
        admin_cost: 2,
    });
    app.update();

    // Assert: Mood is boosted and Administration is spent
    let mood = app.world().get::<Mood>(pop_id).unwrap();
    assert!(mood.value > 50);
    assert_eq!(app.world().resource::<Administration>().current, 8);
    assert!(app.world().get::<FancyTitle>(pop_id).is_some());
}

#[test]
fn test_pops_with_titles_refuse_menial_labor() {
    let mut app = App::new();
    // Arrange: A Pop with a fancy title
    let pop_id = app.world_mut().spawn((Pop, FancyTitle("Vice President of Hauling".into()))).id();

    // Act: Attempt to assign them to a menial job (e.g. Hauler)
    app.world_mut().resource_mut::<Events<AssignJobEvent>>().send(AssignJobEvent {
        target: pop_id,
        job_type: JobType::Menial(MenialJob::Hauler),
    });
    app.update();

    // Assert: The Pop refuses the assignment (job remains empty or unassigned)
    assert!(app.world().get::<Job>(pop_id).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood { pub value: i32 }
impl Mood { pub fn new(value: i32) -> Self { Self { value } } }

#[derive(Component)]
pub struct FancyTitle(pub String);

#[derive(Resource)]
pub struct Administration { pub current: i32 }
impl Administration { pub fn new(current: i32) -> Self { Self { current } } }

#[derive(Event)]
pub struct GrantTitleEvent {
    pub target: Entity,
    pub title: String,
    pub admin_cost: i32,
}

#[derive(Event)]
pub struct AssignJobEvent {
    pub target: Entity,
    pub job_type: JobType,
}

pub enum JobType {
    Menial(MenialJob),
    Management,
}

pub enum MenialJob {
    Hauler,
}

#[derive(Component)]
pub struct Job(pub JobType);

pub fn grant_title_system(
    mut events: EventReader<GrantTitleEvent>,
    mut admin: ResMut<Administration>,
    mut query: Query<&mut Mood>,
    mut commands: Commands,
) {
    for event in events.read() {
        if admin.current >= event.admin_cost {
            admin.current -= event.admin_cost;
            if let Ok(mut mood) = query.get_mut(event.target) {
                mood.value += 10;
            }
            commands.entity(event.target).insert(FancyTitle(event.title.clone()));
        }
    }
}

pub fn assign_job_system(
    mut events: EventReader<AssignJobEvent>,
    query: Query<Option<&FancyTitle>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(title) = query.get(event.target) {
            // Refuse menial jobs if the pop has a fancy title
            if title.is_some() && matches!(event.job_type, JobType::Menial(_)) {
                continue;
            }
            commands.entity(event.target).insert(Job(event.job_type)); // simplified for GREEN
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The job assignment logic is simplified; it should probably integrate with a more robust task queuing system rather than an instant event.
- **Performance**: `grant_title_system` and `assign_job_system` are simple queries. As the number of Pops grows, filtering for eligible assignment candidates beforehand is better than trying to assign and failing.
- **API Improvements**: `JobType` needs a clearer hierarchy if "Management" versus "Menial" becomes a major game mechanic.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops with fancy titles successfully refuse menial assignments

## 7. Technical Guidance
- **Gotchas**: Don't let the player infinite-loop title grants to max out Mood. There should be a cooldown or a stacking admin cost per title.
- **Integration Points**: Tie this into the UI so the player can actually click a Pop and hit "Grant Promotion", seeing the exact cost.

## 8. Questions
*Builder: add questions here if spec is unclear.*

# 655: University of the Stars

## 1. Overview
**Layer:** 3 -> 1
**Fantasy:** Knowledge is power, and you are the librarian.
**Mechanic:** High-level Education buildings attract "Foreign Students" (Pops from other Civs). They pay tuition (Credits/Diplomacy) but bring their home Civ's Ethics (Influence pressure).
**Emergence:** Your colony becomes a pacifist democracy because you hosted too many students from the "Galactic Republic", toppling your own military junta.
**Tension:** Profit/Science vs. Cultural contamination.

## 2. Dependencies
- Layer 1 Education Building Systems (e.g., `University` or `Academy` components)
- Layer 1 Pop Systems (`Pop`, `PopIdeology`, `PopNeeds`)
- Layer 3 Diplomatic Systems (`DiplomaticState`, `FactionData`)
- Integration: Add event for student arrival/departure.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_university_attracts_foreign_students() {
        // Arrange
        let mut app = App::new();
        app.add_event::<ForeignStudentArrivalEvent>();
        app.insert_resource(DiplomaticState {
            factions: vec![FactionData {
                id: "galactic_republic".to_string(),
                primary_ideology: "Pacifist".to_string(),
                diplomatic_standing: 50.0,
            }],
        });
        app.add_systems(Update, process_foreign_student_attraction);

        let university_entity = app.world_mut().spawn((
            University { level: 3, capacity: 100, current_students: 0 },
            GlobalTransform::default(),
        )).id();

        // Act
        app.update();

        // Assert
        let events = app.world().resource::<Events<ForeignStudentArrivalEvent>>();
        let mut reader = events.get_reader();
        let received_events: Vec<_> = reader.read(events).collect();

        assert!(!received_events.is_empty(), "High-level university should attract students");
        assert_eq!(received_events[0].faction_id, "galactic_republic");
    }

    #[test]
    fn test_foreign_students_pay_tuition() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(ColonyResources { credits: 1000.0, ..default() });
        app.add_systems(Update, process_foreign_student_tuition);

        app.world_mut().spawn((
            Pop,
            ForeignStudent { faction_id: "galactic_republic".to_string(), tuition_paid: false },
        ));

        // Act
        app.update();

        // Assert
        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.credits > 1000.0, "Colony should receive credits from tuition");
    }

    #[test]
    fn test_foreign_students_exert_ideological_pressure() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_student_ideological_pressure);

        let native_pop = app.world_mut().spawn((
            Pop,
            PopIdeology { current_ideology: "Militarist".to_string(), shift_pressure: 0.0 },
        )).id();

        app.world_mut().spawn((
            Pop,
            ForeignStudent { faction_id: "galactic_republic".to_string(), tuition_paid: true },
            PopIdeology { current_ideology: "Pacifist".to_string(), shift_pressure: 0.0 },
        ));

        // Act
        // Run multiple updates to simulate time passing and pressure accumulating
        for _ in 0..10 {
            app.update();
        }

        // Assert
        let native_ideology = app.world().get::<PopIdeology>(native_pop).unwrap();
        assert!(native_ideology.shift_pressure > 0.0, "Native pop should experience ideological shift pressure from foreign students");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct University {
    pub level: u32,
    pub capacity: u32,
    pub current_students: u32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct ForeignStudent {
    pub faction_id: String,
    pub tuition_paid: bool,
}

#[derive(Component)]
pub struct PopIdeology {
    pub current_ideology: String,
    pub shift_pressure: f32,
}

#[derive(Event)]
pub struct ForeignStudentArrivalEvent {
    pub faction_id: String,
    pub ideology: String,
}

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub credits: f32,
}

#[derive(Clone)]
pub struct FactionData {
    pub id: String,
    pub primary_ideology: String,
    pub diplomatic_standing: f32,
}

#[derive(Resource, Default)]
pub struct DiplomaticState {
    pub factions: Vec<FactionData>,
}

pub fn process_foreign_student_attraction(
    mut events: EventWriter<ForeignStudentArrivalEvent>,
    query: Query<&University>,
    diplomacy: Option<Res<DiplomaticState>>,
) {
    if let Some(diplomacy) = diplomacy {
        for university in query.iter() {
            if university.level >= 3 && university.current_students < university.capacity {
                if let Some(faction) = diplomacy.factions.first() {
                    events.send(ForeignStudentArrivalEvent {
                        faction_id: faction.id.clone(),
                        ideology: faction.primary_ideology.clone(),
                    });
                }
            }
        }
    }
}

pub fn process_foreign_student_tuition(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<&mut ForeignStudent>,
) {
    for mut student in query.iter_mut() {
        if !student.tuition_paid {
            resources.credits += 100.0;
            student.tuition_paid = true;
        }
    }
}

pub fn apply_student_ideological_pressure(
    mut native_query: Query<&mut PopIdeology, (With<Pop>, Without<ForeignStudent>)>,
    student_query: Query<&PopIdeology, With<ForeignStudent>>,
) {
    let student_count = student_query.iter().count();
    if student_count > 0 {
        // Find dominant student ideology for simplicity in minimal implementation
        if let Some(dominant_student_ideology) = student_query.iter().next() {
             for mut native in native_query.iter_mut() {
                 if native.current_ideology != dominant_student_ideology.current_ideology {
                     native.shift_pressure += 0.1 * (student_count as f32);
                 }
             }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The `apply_student_ideological_pressure` minimal implementation just picks the first student's ideology. It needs to calculate the aggregate pressure of all student ideologies and apply it proportionally to nearby native pops, not globally.
- **Performance**: Ideological pressure calculations should be spatial (e.g., within the same university or residential block) rather than iterating over all native pops against all student pops.
- **Design Improvements**: Link `ForeignStudentArrivalEvent` to actual Pop spawning logic via an integration layer. Add diplomatic reputation modifiers based on how many students you are hosting.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Universities level 3+ emit `ForeignStudentArrivalEvent`.
- [ ] Foreign students add credits to `ColonyResources`.
- [ ] Foreign students increase ideological shift pressure on native pops.

## 7. Technical Guidance
- Ensure `ForeignStudent` acts as a marker component and is added alongside `Pop`.
- Bevy Events should bridge the Layer 3 (Diplomacy) data stream to Layer 1 (Colony) pop spawning.
- Consider adding a "Tuition Value" configuration per faction based on diplomatic standing.

## 8. Questions
*Builder: add questions here if spec is unclear.*

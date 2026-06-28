use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Desensitization {
    pub level: f32,
}

#[derive(Event)]
pub struct AtrocityEvent {
    pub severity: f32,
}

#[derive(Event)]
pub struct MoraleBuffEvent {
    pub target: Entity,
    pub amount: f32,
}

#[derive(Event)]
pub struct StressPenaltyEvent {
    pub target: Entity,
    pub amount: f32,
}

pub fn process_atrocities(
    mut events: EventReader<AtrocityEvent>,
    mut desens: ResMut<Desensitization>,
) {
    for ev in events.read() {
        desens.level = (desens.level + (ev.severity * 0.01)).min(1.0);
    }
}

pub fn apply_morale_buffs(
    mut events: EventReader<MoraleBuffEvent>,
    mut query: Query<&mut Morale>,
    desens: Res<Desensitization>,
) {
    for ev in events.read() {
        if let Ok(mut morale) = query.get_mut(ev.target) {
            let actual_buff = ev.amount * (1.0 - desens.level);
            morale.value += actual_buff;
        }
    }
}

pub fn apply_stress_penalties(
    mut events: EventReader<StressPenaltyEvent>,
    mut query: Query<&mut crate::layer1::needs::Needs>,
    desens: Res<Desensitization>,
) {
    for ev in events.read() {
        if let Ok(mut needs) = query.get_mut(ev.target) {
            // Stress penalty typically reduces leisure. Higher desens means lower stress penalty.
            let actual_penalty = ev.amount * (1.0 - desens.level);
            needs.leisure = (needs.leisure - actual_penalty).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy::prelude::*;

    #[test]
    fn test_atrocity_increases_desensitization() {
        let mut app = App::new();
        app.add_systems(Update, process_atrocities);

        app.world_mut()
            .insert_resource(Desensitization { level: 0.0 });
        app.world_mut()
            .insert_resource(Events::<AtrocityEvent>::default());

        // Send an atrocity
        app.world_mut().send_event(AtrocityEvent { severity: 10.0 });

        app.update();

        let d_level = app.world().resource::<Desensitization>().level;
        assert!(d_level > 0.0);
    }

    #[test]
    fn test_high_desensitization_dampens_morale_buffs() {
        let mut app = App::new();
        app.add_systems(Update, apply_morale_buffs);

        // Colony is numb
        app.world_mut()
            .insert_resource(Desensitization { level: 1.0 }); // 100% numb
        app.world_mut()
            .insert_resource(Events::<MoraleBuffEvent>::default());

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Morale {
                    value: 50.0,
                    ..Default::default()
                },
            ))
            .id();

        // Send a massive buff
        app.world_mut().send_event(MoraleBuffEvent {
            target: pop,
            amount: 50.0,
        });

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        // The buff should be heavily dampened
        assert!(morale.value < 100.0);
        assert_eq!(morale.value, 50.0); // 100% dampening means no effect
    }

    #[test]
    fn test_high_desensitization_dampens_stress_penalties() {
        let mut app = App::new();
        app.add_systems(Update, apply_stress_penalties);

        // Colony is numb
        app.world_mut()
            .insert_resource(Desensitization { level: 1.0 }); // 100% numb
        app.world_mut()
            .insert_resource(Events::<StressPenaltyEvent>::default());

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Needs {
                    leisure: 100.0,
                    ..Default::default()
                },
            ))
            .id();

        // Send a massive stress penalty
        app.world_mut().send_event(StressPenaltyEvent {
            target: pop,
            amount: 50.0,
        });

        app.update();

        let needs = app.world().get::<Needs>(pop).unwrap();
        // The penalty should be heavily dampened
        assert_eq!(needs.leisure, 100.0); // 100% dampening means no stress effect
    }
}
